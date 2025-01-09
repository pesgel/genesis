//! process

use std::iter::once;
use std::{sync::Arc, time::Duration};

use bytes::{Bytes, BytesMut};
use genesis_common::{NotifyEnum, SshTargetPasswordAuth, TargetSSHOptions};
use genesis_ssh::start_ssh_connect;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::{
    select,
    sync::{mpsc::unbounded_channel, watch, Mutex, RwLock},
};
use tracing::info;
use uuid::Uuid;

use crate::{Execute, PreMatchTypeEnum};

#[derive(Clone, Default)]
enum PipeState {
    #[default]
    In,
    Out,
}
#[derive(Clone, Debug, Default)]
pub struct PipeCmd {
    pub input: String,
    pub output: String,
}

#[derive(Debug)]
pub struct Pipe {
    pub sender: UnboundedSender<Bytes>,
    pub reader: UnboundedReceiver<Bytes>,
}

impl Pipe {
    pub fn new(sender: UnboundedSender<Bytes>, reader: UnboundedReceiver<Bytes>) -> Self {
        Self { sender, reader }
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct PipeManger {
    ps1: Arc<RwLock<String>>,
    parser: Arc<Mutex<vt100::Parser>>,
    state: Arc<RwLock<PipeState>>,
    out_buf: Arc<Mutex<vt100::Parser>>,
    input_buf: Arc<Mutex<vt100::Parser>>,
    wait_times: u8,
}

impl PipeManger {
    pub async fn do_interactive(
        &mut self,
        mut in_io: Pipe,
        mut out_io: Pipe,
        cmd_sender: Option<UnboundedSender<PipeCmd>>,
        abort_rc: &watch::Receiver<bool>,
    ) -> Result<(), String> {
        let mut abort_in_io: watch::Receiver<bool> = abort_rc.clone();
        tokio::spawn({
            let ps1 = self.ps1.clone();
            let state = self.state.clone();
            let wait_times = self.wait_times;
            async move {
                loop {
                    select! {
                        flag = abort_in_io.changed() => match flag {
                            Ok(_) => {
                                if *abort_in_io.borrow() {
                                    info!("interactive receive abort signal");
                                    return ;
                                }
                            },
                            Err(e) => {
                                info!("interactive receive abort signal error: {:?}",e);
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
                                let mut now_wait_time = 0;
                                loop {
                                    match *state.read().await {
                                        PipeState::In => break,
                                        PipeState::Out => {
                                            tokio::time::sleep(Duration::from_millis(20)).await;
                                            now_wait_time += 1;
                                            if now_wait_time >= wait_times {
                                                info!("time out, break");
                                                break;
                                            }
                                        },
                                    }
                                }
                                let _ = out_io.sender.send(data);
                            },
                            None => {
                                info!("receive none");
                                break
                            },
                        }
                    }
                }
            }
        });

        // 接受返回数据,并发送到channel
        tokio::spawn({
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
                                        *(stat.write().await) = PipeState::Out;
                                    }
                                    // 判断当前接受状态,若是输入状态,则写到input
                                    // 若是输出状态,则写入到output
                                    if !ps1.read().await.is_empty() {
                                        match *(stat.read().await) {
                                            PipeState::In => {
                                                input.lock().await.process(data);
                                            },
                                            PipeState::Out => {
                                                output.lock().await.process(data);
                                            },
                                        }
                                    }
                                    buffer.extend_from_slice(data);
                                    info!("data: {:?}",buffer);
                                    if let Some(p1) = extract_command_after_bell(&buffer){
                                        *(ps1.write().await) = String::from_utf8_lossy(p1).to_string();
                                        // 打印ps1,并清空
                                        *(stat.write().await) = PipeState::In;
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
                                            let pipe_cmd = PipeCmd{
                                                input: cmd_input,
                                                output: cmd_out,
                                            };
                                            info!("send:{:?}",pipe_cmd);
                                            let _ = sender.send(pipe_cmd);
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
                    }
                }
            }
        });
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
        let (hub, sender, _) = start_ssh_connect(uuid, option).await.unwrap();

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
                                            PipeState::In => break,
                                            PipeState::Out => {},
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
                                        *(stat.write().await) = PipeState::Out;
                                    }
                                    // 判断当前接受状态,若是输入状态,则写到input
                                    // 若是输出状态,则写入到output
                                    if !ps1.read().await.is_empty() {
                                        match *(stat.read().await) {
                                            PipeState::In => {
                                                input.lock().await.process(data);
                                            },
                                            PipeState::Out => {
                                                output.lock().await.process(data);
                                            },
                                        }
                                    }
                                    buffer.extend_from_slice(data);
                                    if let Some(p1) = extract_command_after_bell(&buffer){
                                        *(ps1.write().await) = String::from_utf8_lossy(p1).to_string();
                                        // 打印ps1,并清空
                                        *(stat.write().await) = PipeState::In;
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
                    }
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

#[allow(dead_code)]
fn extract_command_after_bell_back(data: &[u8]) -> Option<&[u8]> {
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
fn extract_command_after_bell(data: &[u8]) -> Option<&[u8]> {
    // 将字节流按行拆分
    let lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();

    // 从最后一行开始向前查找，找到包含 \x1b 的行
    for line in lines.iter().rev() {
        if line.contains(&0x1b) {
            // 查找是否包含 \x1b
            return Some(*line);
        }
    }
    None // 如果没有找到包含 \x1b 的行
}

pub struct ProcessManger {
    pub execute: Arc<Mutex<Execute>>,
}

pub type MatchFnType = Arc<Mutex<dyn Fn(&str) -> bool + Send>>;
#[derive(Clone)]
pub struct ExecuteFns {
    pub fns: Vec<MatchFnType>,        // 使用 Arc<Mutex> 确保闭包线程安全
    pub execute: Arc<Mutex<Execute>>, // 存储 Execute 的 Arc<Mutex>
}

impl ProcessManger {
    #[allow(dead_code)]
    pub async fn run(&mut self, uuid: Uuid, ssh_option: TargetSSHOptions) -> anyhow::Result<()> {
        let (abort_sc, abort_rc) = watch::channel(false);
        let (hub, sender, mut notify) = start_ssh_connect(uuid, ssh_option).await?;
        //
        match notify.changed().await {
            Ok(_) => match *notify.borrow() {
                NotifyEnum::ERROR(ref e) => {
                    info!("connect error: {}", e);
                    anyhow::bail!(e.clone());
                }
                _ => {
                    info!("connect success")
                }
            },
            Err(e) => {
                info!("handler_ssh receive abort signal error: {:?}", e);
                anyhow::bail!(e.to_string());
            }
        };

        let receiver = hub.subscribe(|_| true).await;
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        let (psc, _) = unbounded_channel::<Bytes>();

        let in_pipe = Pipe::new(psc, in_rc);
        let out_pipe = Pipe::new(sender, receiver.unbox());

        let mut manager = PipeManger {
            wait_times: 100,
            ..Default::default()
        };
        let (cmd_sc, mut cmd_rc) = unbounded_channel();
        let _ = manager
            .do_interactive(in_pipe, out_pipe, Some(cmd_sc), &abort_rc)
            .await;
        let res = manager.out_buf.clone();
        let state = manager.state.clone();
        let mut abort_execute_cmd = abort_rc.clone();

        let (cmd_sender, mut cmd_executor) = unbounded_channel();

        cmd_sender.send(self.execute.clone())?;

        let a: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            loop {
                select! {
                    flag = abort_execute_cmd.changed() => match flag {
                        Ok(_) => {
                            if *abort_execute_cmd.borrow() {
                                info!("loop receive abort signal");
                                return;
                            }
                        },
                        Err(e) => {
                            info!("loop receive abort signal error: {:?}",e);
                            return
                        }, // 如果接收到错误，也退出
                    },
                    ma = cmd_executor.recv() => match ma {
                            Some(execute) => {
                            let exe = execute.lock().await.clone();
                            let mut cmd = exe.node.core.cmd.clone();
                            if !cmd.ends_with('\r') {
                                cmd.push('\r');
                            }
                            // 发送命令到远程执行
                            info!("send node:{} cmd:{}", exe.node.id, cmd);
                            let _ = sc.send(cmd.into());
                            // 执行完毕,根据子节点配置pre数据,判断需要走哪条分支
                            if exe.children.is_empty() {
                                return;
                            }
                            // children存在,组装出ExecuteFns
                            let execute_fns = RwLock::new(Vec::new());
                            // 整理子节点
                            for children_node_arc in exe.children {
                                let children_node = children_node_arc.lock().await.clone();
                                let mut mm_fn: Vec<MatchFnType> = Vec::new();
                                match children_node.node.pre {
                                    Some(pre) => {
                                        // pre存在,则获取其中的值
                                        for pre_item in pre.list {
                                            match pre_item.match_type {
                                                PreMatchTypeEnum::Eq => {
                                                    mm_fn.push(Arc::new(Mutex::new(move |s: &str| {
                                                        info!("input eq:{}", s);
                                                        s == pre_item.value
                                                    })));
                                                }
                                                PreMatchTypeEnum::Reg => {
                                                    mm_fn.push(Arc::new(Mutex::new(move |s: &str| {
                                                        s == pre_item.value
                                                    })));
                                                }
                                                PreMatchTypeEnum::Contains => {
                                                    mm_fn.push(Arc::new(Mutex::new(move |s: &str| {
                                                        s.to_lowercase()
                                                            .contains(&pre_item.value.to_lowercase())
                                                    })));
                                                }
                                                PreMatchTypeEnum::NotContains => {
                                                    mm_fn.push(Arc::new(Mutex::new(move |s: &str| {
                                                        !s.to_lowercase()
                                                            .contains(&pre_item.value.to_lowercase())
                                                    })));
                                                }
                                            }
                                        }
                                    }
                                    None => continue,
                                }
                                execute_fns.write().await.push(ExecuteFns {
                                    fns: mm_fn,
                                    execute: children_node_arc,
                                });
                            }
                            //存在子节点,等待子节点匹配
                            loop {
                                select! {
                                    flag = abort_execute_cmd.changed() => match flag {
                                        Ok(_) => {
                                            if *abort_execute_cmd.borrow() {
                                                info!("cmd execute loop receive abort signal");
                                                return; // 如果收到中止命令，退出
                                            }
                                        },
                                        Err(e) => {
                                            info!("cmd execute loop receive abort signal error: {:?}",e);
                                            return
                                        },
                                    },
                                    _ = tokio::time::sleep(Duration::from_secs(3)) => {
                                        let content = &res.lock().await.screen().contents(); // 获取屏幕内容
                                        info!("receive content: {}", content);
                                        match process_execute_fns(&execute_fns, content, &cmd_sender, &state).await{
                                            Ok(_) => {
                                                info!("time stop loop");
                                                break;
                                            },
                                            Err(_) => {
                                                info!("time all not match content:{}\n",content);
                                            },
                                        }
                                    },
                                    ma = cmd_rc.recv() => match ma {
                                        Some(md) => {
                                            let content = md.output.clone();
                                            info!("receive cmd: {}", content);
                                            match process_execute_fns(&execute_fns, &content, &cmd_sender, &state).await{
                                                Ok(_) => {
                                                    info!("cmd stop loop");
                                                    break;
                                                },
                                                Err(_) => {
                                                    info!("cmd all not match content:{}\n",content);
                                                },
                                            }
                                        },
                                        None => break, // 如果没有命令，退出循环
                                    }
                                }
                            }
                        }
                        None => {
                            info!("stop execute cmd");
                            break;
                        }
                    }
                }
            }
        });
        let _ = tokio::join!(a);
        let _ = abort_sc.send(true);
        info!("end cmd");
        anyhow::Ok(())
    }
}

// 拆分判断所有条件是否满足的函数
async fn check_conditions(fns: &[MatchFnType], input: &str) -> bool {
    for ef in fns {
        let x = ef.lock().await;
        if !(*x)(input) {
            return false;
        }
    }
    true
}

// 拆分匹配并执行的逻辑
async fn process_execute_fns(
    execute_fns: &RwLock<Vec<ExecuteFns>>,
    input: &str,
    cmd_sender: &UnboundedSender<Arc<Mutex<Execute>>>,
    state: &RwLock<PipeState>,
) -> anyhow::Result<()> {
    let efn = execute_fns.read().await;
    if efn.is_empty() {
        return anyhow::Ok(());
    }
    for fnn in efn.iter() {
        if check_conditions(&fnn.fns, input).await {
            // 发送命令
            cmd_sender.send(fnn.execute.clone())?;
            // 更新状态
            *state.write().await = PipeState::In;
            // 如果找到匹配的条件，跳出循环
            return anyhow::Ok(());
        }
    }
    anyhow::bail!("not match any branch")
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

    use super::PipeManger;
    use crate::{Core, Edge, Item, Node, Pipe, Position, Pre};

    use crate::{Graph, InData};

    use super::*;
    #[tokio::test]
    #[ignore]
    async fn test_graph_building() {
        let new_password = "%]m73MmQ";
        //let old_password = "#wR61V(s";
        let old_password = "#wR61V(s";
        tracing_subscriber::fmt().init();
        // 创建一些测试数据
        let in_data = InData {
            nodes: vec![
                Node {
                    id: "1".to_string(),
                    pre: None,
                    core: Core {
                        des: "判断是否是home目录".to_string(),
                        cmd: "pwd".to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
                Node {
                    id: "2".to_string(),
                    pre: Some(Pre {
                        list: vec![Item {
                            value: "home/yangping".to_string(),
                            match_type: PreMatchTypeEnum::Contains,
                        }],
                    }),
                    core: Core {
                        des: "home目录执行密码变更".to_string(),
                        cmd: "passwd".to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
                Node {
                    id: "3".to_string(),
                    pre: Some(Pre {
                        list: vec![Item {
                            value: "/root".to_string(),
                            match_type: PreMatchTypeEnum::Contains,
                        }],
                    }),
                    core: Core {
                        des: "root目录直接退出".to_string(),
                        cmd: "exit".to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
                Node {
                    id: "4".to_string(),
                    pre: Some(Pre {
                        list: vec![Item {
                            value: "current".to_string(),
                            match_type: PreMatchTypeEnum::Contains,
                        }],
                    }),
                    core: Core {
                        des: "输入当前密码".to_string(),
                        cmd: old_password.to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
                Node {
                    id: "5".to_string(),
                    pre: Some(Pre {
                        list: vec![Item {
                            value: "New password".to_string(),
                            match_type: PreMatchTypeEnum::Contains,
                        }],
                    }),
                    core: Core {
                        des: "输入新密码".to_string(),
                        cmd: new_password.to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
                Node {
                    id: "6".to_string(),
                    pre: Some(Pre {
                        list: vec![Item {
                            value: "Retype new password".to_string(),
                            match_type: PreMatchTypeEnum::Contains,
                        }],
                    }),
                    core: Core {
                        des: "确认新密码".to_string(),
                        cmd: new_password.to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
                Node {
                    id: "7".to_string(),
                    pre: Some(Pre {
                        list: vec![Item {
                            value: "success".to_string(),
                            match_type: PreMatchTypeEnum::Contains,
                        }],
                    }),
                    core: Core {
                        des: "退出".to_string(),
                        cmd: "exit".to_string(),
                    },
                    post: None,
                    position: Position::default(),
                },
            ],
            edges: vec![
                Edge {
                    source: "1".to_string(),
                    target: "2".to_string(),
                },
                Edge {
                    source: "1".to_string(),
                    target: "3".to_string(),
                },
                Edge {
                    source: "2".to_string(),
                    target: "4".to_string(),
                },
                Edge {
                    source: "4".to_string(),
                    target: "5".to_string(),
                },
                Edge {
                    source: "5".to_string(),
                    target: "6".to_string(),
                },
                Edge {
                    source: "6".to_string(),
                    target: "7".to_string(),
                },
            ],
        };
        let mut graph = Graph::new();
        graph.build_from_edges(in_data).await;
        // 打印图的结构
        // graph.print_graph().await;
        let execute = graph.start_node().await.unwrap();

        let mut pm = ProcessManger { execute };

        let option = TargetSSHOptions {
            host: "10.0.1.52".into(),
            port: 22,
            username: "yangping".into(),
            allow_insecure_algos: Some(true),
            auth: genesis_common::SSHTargetAuth::Password(SshTargetPasswordAuth {
                password: old_password.to_string(),
            }),
        };
        let _ = pm.run(Uuid::new_v4(), option).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_parser_manager_test() {
        tracing_subscriber::fmt().init();
        let mut pm = PipeManger::default();
        pm.do_interactive_test().await;
        info!("end")
    }

    #[tokio::test]
    #[ignore]
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
        let (hub, sender, _) = start_ssh_connect(uuid, option).await.unwrap();
        let receiver = hub.subscribe(|_| true).await;
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        let (psc, _) = unbounded_channel::<Bytes>();

        let in_pipe = Pipe::new(psc, in_rc);
        let out_pipe = Pipe::new(sender, receiver.unbox());

        let mut manager = PipeManger::default();
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

    #[tokio::test]
    async fn test_continue() {}
}
