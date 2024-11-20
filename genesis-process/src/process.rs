//! process

use std::iter::once;
use std::{sync::Arc, time::Duration};

use bytes::{Bytes, BytesMut};
use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
use genesis_ssh::start_ssh_connect;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::{
    select,
    sync::{mpsc::unbounded_channel, watch, Mutex, RwLock},
};
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Default)]
enum ProcessState {
    #[default]
    In,
    Out,
}
#[derive(Clone, Debug, Default)]
pub struct ProcessCmd {
    pub input: String,
    pub output: String,
}

#[derive(Debug)]
pub struct ProcessPipe {
    pub sender: UnboundedSender<Bytes>,
    pub reader: UnboundedReceiver<Bytes>,
}

impl ProcessPipe {
    pub fn new(sender: UnboundedSender<Bytes>, reader: UnboundedReceiver<Bytes>) -> Self {
        Self { sender, reader }
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct ProcessManger {
    ps1: Arc<RwLock<String>>,
    parser: Arc<Mutex<vt100::Parser>>,
    state: Arc<RwLock<ProcessState>>,
    out_buf: Arc<Mutex<vt100::Parser>>,
    input_buf: Arc<Mutex<vt100::Parser>>,
}

impl ProcessManger {
    pub async fn do_interactive(
        &mut self,
        mut in_io: ProcessPipe,
        mut out_io: ProcessPipe,
        cmd_sender: Option<UnboundedSender<ProcessCmd>>,
        abort_rc: &watch::Receiver<bool>,
    ) -> Result<(), String> {
        let mut abort_in_io: watch::Receiver<bool> = abort_rc.clone();
        let a = tokio::spawn({
            let ps1 = self.ps1.clone();
            let state = self.state.clone();
            async move {
                loop {
                    select! {
                        flag = abort_in_io.changed() => match flag {
                            Ok(_) => {
                                if *abort_in_io.borrow() {
                                    return ;
                                }
                            },
                            Err(_) => {
                                return ;
                            },
                        },
                        rb = in_io.reader.recv() => match rb {
                            Some(data) => {
                                // 未设置ps1 不允许输入
                                while ps1.read().await.is_empty() {
                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                }
                                // 判断输入状态是否是允许输入
                                loop {
                                    select! {
                                        _ = tokio::time::sleep(Duration::from_millis(200))=>{
                                            info!("wait sate overtime break");
                                            break;
                                        },
                                        sa = state.read()=> match *sa {
                                            ProcessState::In => break,
                                            ProcessState::Out => {},
                                        }
                                    }
                                }
                                let _ = out_io.sender.send(data);
                            },
                            None => break,
                        }
                    }
                }
            }
        });

        // 接受返回数据,并发送到channel
        let h = tokio::spawn({
            let parser = self.parser.clone();
            let ps1 = self.ps1.clone();
            let stat = self.state.clone();
            let input = self.input_buf.clone();
            let output = self.out_buf.clone();
            let mut abort_sc: watch::Receiver<bool> = abort_rc.clone();
            async move {
                let mut buffer = BytesMut::new();
                'ro: loop {
                    select! {
                        rb = out_io.reader.recv() => match rb {
                            Some(data) => {
                                let parts: Vec<Bytes> = data
                                    .split(|&byte| byte == b'\r')
                                    .flat_map(|part| once(Bytes::copy_from_slice(part)).chain(once(Bytes::from_static(b"\r"))))
                                    // TODO 性能影响
                                    .take(data.split(|&byte| byte == b'\r').count() * 2 - 1) // 去掉最后一个多余的 \r
                                    .filter(|e| !e.is_empty())
                                    .collect();
                                // loop
                                for data in parts.iter() {
                                    let mut par = parser.lock().await;
                                    par.process(data);
                                    if par.screen().alternate_screen() {
                                        //VIM等界面
                                        continue 'ro;
                                    }
                                    if data.len() ==1 && data[0] == b'\r'{
                                        *(stat.write().await) = ProcessState::Out;
                                    }
                                    // 判断当前接受状态,若是输入状态,则写到input
                                    // 若是输出状态,则写入到output
                                    if !ps1.read().await.is_empty() {
                                        match *(stat.read().await) {
                                            ProcessState::In => {
                                                input.lock().await.process(data);
                                            },
                                            ProcessState::Out => {
                                                output.lock().await.process(data);
                                            },
                                        }
                                    }
                                    buffer.extend_from_slice(data);
                                    if let Some(p1) = extract_command_after_bell(&buffer){
                                        *(ps1.write().await) = String::from_utf8_lossy(p1).to_string();
                                        // 打印ps1,并清空
                                        *(stat.write().await) = ProcessState::In;
                                        buffer.clear();

                                        let mut input = input.lock().await;
                                        let cmd_input = input.screen().contents().trim().to_string();
                                        if cmd_input.is_empty() {
                                            continue;
                                        }
                                        input.process(b"\x1b[2J");
                                        let mut output = output.lock().await;
                                        let ps1_value = ps1.read().await;
                                        let cmd_out = output.screen().contents().replace(ps1_value.as_str(), "").trim().to_string();
                                        output.process(b"\x1b[2J");
                                        if let Some(sender) = cmd_sender.clone() {
                                            let _ = sender.send(ProcessCmd{
                                                input: cmd_input,
                                                output: cmd_out,
                                            });
                                        }
                                    }
                                }
                                // 处理完毕,发送数据
                                let _ = in_io.sender.send(data);
                            },
                            None => {return ;},
                        },
                        flag = abort_sc.changed() => match flag {
                            Ok(_) => {
                                if *abort_sc.borrow() {
                                    return ;
                                }
                            },
                            Err(_) => {
                                return ;
                            },
                        }
                    };
                }
            }
        });
        let _ = tokio::join!(a, h);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn do_interactive_test(&mut self) {
        info!("do_interactive");
        // step1. 建立到远程服务的连接并且初始化事件处理器
        let uuid = Uuid::new_v4();
        let option = TargetSSHOptions {
            host: "10.0.1.52".into(),
            port: 22,
            username: "root".into(),
            allow_insecure_algos: Some(true),
            auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
                password: "1qaz2wsx".into(),
            }),
        };
        // 退出通知
        let (abort_sc, mut abort_rc) = watch::channel(false);
        // remote connect
        let (hub, sender) = start_ssh_connect(uuid, option).await.unwrap();

        let mut receiver = hub.subscribe(|_| true).await;
        let (sc, mut rc) = unbounded_channel::<Bytes>();

        let a = tokio::spawn(async move {
            let _ = tokio::time::sleep(Duration::from_secs(5)).await;
            let _ = sc.send(Bytes::from_static(b"p"));
            let _ = sc.send(Bytes::from_static(b"w"));
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;
            let _ = sc.send(Bytes::from_static(b"d"));
            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = sc.send(Bytes::from_static(b"\r"));
            info!("send pwd");
            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = sc.send(Bytes::from_static(b"cd /home\r"));
            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = sc.send(Bytes::from_static(b"pwwd\r"));
            let _ = tokio::time::sleep(Duration::from_secs(5)).await;

            // TODO 多行测试
            // let _ = tokio::time::sleep(Duration::from_secs(5)).await;
            // let _ = sc.send(Bytes::from_static(b"pw\\"));
            // let _ = sc.send(Bytes::from_static(b"\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"d"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"cd /home\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"pwwd\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;

            // info!("send vim");
            // let _ = sender.send(Bytes::from_static(b"vim test.txt\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(20)).await;
            abort_sc.send(true)
        });

        tokio::spawn({
            let ps1 = self.ps1.clone();
            let state = self.state.clone();
            async move {
                loop {
                    select! {
                        rb = rc.recv() => match rb {
                            Some(data) => {
                                // 未设置ps1 不允许输入
                                while ps1.read().await.is_empty() {
                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                }
                                // 判断输入状态是否是允许输入
                                loop {
                                    select! {
                                        _ = tokio::time::sleep(Duration::from_millis(200))=>{
                                            info!("wait sate overtime break");
                                            break;
                                        },
                                        sa = state.read()=> match *sa {
                                            ProcessState::In => break,
                                            ProcessState::Out => {},
                                        }
                                    }
                                }
                                let _ = sender.send(data);
                            },
                            None => break,
                        }
                    }
                }
            }
        });
        // 接受返回数据,并发送到channel
        let h = tokio::spawn({
            let parser = self.parser.clone();
            let ps1 = self.ps1.clone();
            let stat = self.state.clone();
            let input = self.input_buf.clone();
            let output = self.out_buf.clone();
            async move {
                let mut buffer = BytesMut::new();
                'ro: loop {
                    select! {
                        rb = receiver.recv() => match rb {
                            Some(data) => {
                                let parts: Vec<Bytes> = data
                                    .split(|&byte| byte == b'\r')
                                    .flat_map(|part| once(Bytes::copy_from_slice(part)).chain(once(Bytes::from_static(b"\r"))))
                                    // TODO 性能影响
                                    .take(data.split(|&byte| byte == b'\r').count() * 2 - 1) // 去掉最后一个多余的 \r
                                    .filter(|e| !e.is_empty())
                                    .collect();
                                // loop
                                for data in parts.iter() {
                                    let mut par = parser.lock().await;
                                    par.process(data);
                                    if par.screen().alternate_screen() {
                                        //VIM等界面
                                        continue 'ro;
                                    }
                                    if data.len() ==1 && data[0] == b'\r'{
                                        *(stat.write().await) = ProcessState::Out;
                                    }
                                    // 判断当前接受状态,若是输入状态,则写到input
                                    // 若是输出状态,则写入到output
                                    if !ps1.read().await.is_empty() {
                                        match *(stat.read().await) {
                                            ProcessState::In => {
                                                input.lock().await.process(data);
                                            },
                                            ProcessState::Out => {
                                                output.lock().await.process(data);
                                            },
                                        }
                                    }
                                    buffer.extend_from_slice(data);
                                    if let Some(p1) = extract_command_after_bell(&buffer){
                                        *(ps1.write().await) = String::from_utf8_lossy(p1).to_string();
                                        // 打印ps1,并清空
                                        *(stat.write().await) = ProcessState::In;
                                        buffer.clear();

                                        let input = input.lock().await;
                                        info!("input data: {}",input.screen().contents());
                                        //input.clear();
                                        let output = output.lock().await;
                                        info!("output data: {}",output.screen().contents());
                                        //output.clear();
                                        // // 设置初始光标位置
                                        // let mut pos = position.lock().await;
                                        // let new_pos = par.screen().cursor_position();
                                        // info!("contents:\n{:?}\n",par.screen().contents_between(pos.0, pos.1, new_pos.0, new_pos.1));
                                        // *pos = new_pos;
                                    }
                                }
                            },
                            None => {return ;},
                        },
                        flag = abort_rc.changed() => match flag {
                            Ok(_) => {
                                if *abort_rc.borrow() {
                                    return ;
                                }
                            },
                            Err(_) => {
                                return ;
                            },
                        }
                    };
                }
            }
        });
        let _ = tokio::join!(a, h);
        info!(
            "parser screen:\n{}",
            self.parser.lock().await.screen().contents()
        )
    }
}

fn extract_command_after_bell(data: &[u8]) -> Option<&[u8]> {
    // 查找最后一个 \x1b] 的位置
    let start_pos = data.windows(2).rposition(|w| w == [0x1b, b']']);
    // 查找最后一个 \x07 的位置
    let end_pos = data.iter().rposition(|&b| b == 0x07);
    if let (Some(start), Some(end)) = (start_pos, end_pos) {
        // 确保 \x07 出现在 \x1b] 之后
        if end > start {
            // 提取 \x07 后面的部分
            return Some(&data[end + 1..]);
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use std::{iter::once, time::Duration};

    use bytes::Bytes;
    use genesis_common::{SshTargetPasswordAuth, TargetSSHOptions};
    use genesis_ssh::start_ssh_connect;
    use tokio::sync::{mpsc::unbounded_channel, watch};
    use tracing::info;
    use uuid::Uuid;

    use crate::ProcessPipe;

    use super::ProcessManger;

    #[tokio::test]
    async fn test_parser_manager_test() {
        tracing_subscriber::fmt().init();
        let mut pm = ProcessManger::default();
        pm.do_interactive_test().await;
        info!("end")
    }

    #[tokio::test]
    async fn test_parser_manager() {
        tracing_subscriber::fmt().init();
        let uuid = Uuid::new_v4();
        let option = TargetSSHOptions {
            host: "10.0.1.52".into(),
            port: 22,
            username: "root".into(),
            allow_insecure_algos: Some(true),
            auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
                password: "1qaz2wsx".into(),
            }),
        };
        // In-Reader ------> -------------------> Out-Sender
        //    I                                       I
        // In-Writer <-------------------<------ Out-Reader
        // remote connect
        let (hub, sender) = start_ssh_connect(uuid, option).await.unwrap();
        let receiver = hub.subscribe(|_| true).await;
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        let (psc, _) = unbounded_channel::<Bytes>();

        let in_pipe = ProcessPipe::new(psc, in_rc);
        let out_pipe = ProcessPipe::new(sender, receiver.unbox());

        let mut manager = ProcessManger::default();
        let (abort_sc, abort_rc) = watch::channel(false);
        let (cmd_sc, mut cmd_rc) = unbounded_channel();
        let ma = manager.do_interactive(in_pipe, out_pipe, Some(cmd_sc), &abort_rc);

        let a = tokio::spawn(async move {
            let _ = tokio::time::sleep(Duration::from_secs(5)).await;
            let _ = sc.send(Bytes::from_static(b"p"));
            let _ = sc.send(Bytes::from_static(b"w"));
            let _ = tokio::time::sleep(Duration::from_secs(1)).await;
            let _ = sc.send(Bytes::from_static(b"d"));
            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = sc.send(Bytes::from_static(b"\r"));
            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = sc.send(Bytes::from_static(b"cd /home\r"));
            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = sc.send(Bytes::from_static(b"pwwd\r"));
            let _ = tokio::time::sleep(Duration::from_secs(5)).await;
            // TODO 多行测试
            // let _ = tokio::time::sleep(Duration::from_secs(5)).await;
            // let _ = sc.send(Bytes::from_static(b"pw\\"));
            // let _ = sc.send(Bytes::from_static(b"\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"d"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"cd /home\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // let _ = sc.send(Bytes::from_static(b"pwwd\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(2)).await;
            // info!("send vim");
            // let _ = sender.send(Bytes::from_static(b"vim test.txt\r"));
            // let _ = tokio::time::sleep(Duration::from_secs(20)).await;
            let _ = abort_sc.send(true);
            info!("end pwd");
        });
        let b = tokio::spawn(async move {
            while let Some(cmd) = cmd_rc.recv().await {
                info!("receive cmd:{:?}", cmd);
            }
        });
        let _ = tokio::join!(ma, a, b);
    }

    #[test]
    fn test_split() {
        let data = Bytes::from("111\r11");
        let parts: Vec<Bytes> = data
            .split(|&byte| byte == b'\r')
            .flat_map(|part| {
                once(Bytes::copy_from_slice(part)).chain(once(Bytes::from_static(b"\r")))
            })
            .take(data.split(|&byte| byte == b'\r').count() * 2 - 1)
            .filter(|e| !e.is_empty()) // 去掉最后一个多余的 \r
            .collect();

        for part in parts {
            //println!("{:?}", String::from_utf8_lossy(part));
            println!("{:?},{}", part, part.is_empty());
        }
    }
}
