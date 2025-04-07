//! process

use crate::common::string;
use crate::recording::Recorder;
use crate::types::AsyncMatchFn;
use crate::{Execute, ExecuteState, Item, Pipe, PipeManger, PipeState};
use bytes::Bytes;
use genesis_common::{EventSubscription, NotifyEnum, TargetSSHOptions, TaskStatusEnum};
use genesis_ssh::start_ssh_connect;
use std::collections::HashMap;
use std::io::Write;
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;
use tokio::{
    select,
    sync::{mpsc::unbounded_channel, watch, Mutex, RwLock},
};
use tracing::{debug, error};
use uuid::Uuid;

// pub type MatchFnType = Arc<Mutex<dyn Fn(&str) -> bool + Send>>;
#[derive(Clone)]
pub struct ExecuteFns {
    // pub fns: Vec<MatchFnType>,
    pub fns: Vec<AsyncMatchFn>,
    pub execute: Arc<Mutex<Execute>>,
}

pub struct ProcessManger {
    abort_sc: watch::Sender<bool>,
    recorder: Arc<Mutex<Option<Recorder>>>,
    pub uniq_id: String,
    pub ssh_cmd_wait_times: u8,
    pub execute: Arc<Mutex<Execute>>,
    pub abort_rc: watch::Receiver<bool>,
    pub broadcast_sender: broadcast::Sender<ExecuteState>,
    pub broadcast_receiver: broadcast::Receiver<ExecuteState>,

    // run time param
    execute_info: Arc<Mutex<Option<String>>>,
    cmd_expire_time: Arc<Mutex<Option<Instant>>>,
    global_params: Arc<RwLock<HashMap<String, String>>>,
}

impl ProcessManger {
    pub fn new(uniq_id: String, execute: Arc<Mutex<Execute>>) -> anyhow::Result<ProcessManger> {
        let (abort_sc, abort_rc) = watch::channel(false);
        let (broadcast_sender, broadcast_receiver) = broadcast::channel::<ExecuteState>(2048);
        anyhow::Ok(Self {
            uniq_id,
            execute,
            abort_rc,
            broadcast_sender,
            broadcast_receiver,
            abort_sc,
            ssh_cmd_wait_times: 100,
            recorder: Arc::new(Mutex::new(None)),
            cmd_expire_time: Arc::new(Mutex::new(None)),
            execute_info: Arc::new(Mutex::new(None)),
            global_params: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn insert_global_params(&self, key: String, value: String) {
        self.global_params.write().await.insert(key, value);
    }

    pub fn with_default_recorder(mut self) -> anyhow::Result<Self> {
        self.recorder = Arc::new(Mutex::new(Some(Recorder::new(
            &self.uniq_id,
            "./",
            "xterm",
            80,
            40,
        )?)));
        anyhow::Ok(self)
    }
    pub fn with_recorder_param(
        mut self,
        save_path: &str,
        term: &str,
        height: u8,
        width: u8,
    ) -> anyhow::Result<Self> {
        self.recorder = Arc::new(Mutex::new(Some(Recorder::new(
            &self.uniq_id,
            save_path,
            term,
            height,
            width,
        )?)));
        anyhow::Ok(self)
    }

    pub fn with_recorder(&mut self, recorder: Recorder) -> &mut Self {
        self.recorder = Arc::new(Mutex::new(Some(recorder)));
        self
    }
    pub fn with_ssh_cmd_wait_times(&mut self, times: u8) -> &mut Self {
        self.ssh_cmd_wait_times = times;
        self
    }
    pub fn register_state_watcher(&self) -> broadcast::Receiver<ExecuteState> {
        self.broadcast_receiver.resubscribe()
    }

    pub fn get_abort_sc(&self) -> watch::Sender<bool> {
        self.abort_sc.clone()
    }

    pub fn stop_process(&self) {
        match self.abort_sc.send(true) {
            Ok(_) => {}
            Err(e) => {
                error!(session_id=%self.uniq_id,"execute:{} abort signal error:{}", self.uniq_id, e);
            }
        }
    }

    async fn set_cmd_expire_time(&self, expire_secs: u64) {
        let mut expire_time = self.cmd_expire_time.lock().await;
        *expire_time = Some(Instant::now() + Duration::from_secs(expire_secs))
    }

    async fn check_cmd_expire_time(&self) -> bool {
        self.cmd_expire_time
            .lock()
            .await
            .map(|t| Instant::now().ge(&t))
            .unwrap_or(false)
    }

    async fn set_execute_info(&self, info: String) {
        *self.execute_info.lock().await = Some(info);
    }

    async fn get_execute_info(&self) -> Option<String> {
        self.execute_info.lock().await.clone()
    }

    async fn wait_ssh_state(&self, mut notify: watch::Receiver<NotifyEnum>) -> anyhow::Result<()> {
        match notify.changed().await {
            Ok(_) => match *notify.borrow() {
                NotifyEnum::ERROR(ref e) => {
                    debug!(session_id=%self.uniq_id,"connect error: {}", e);
                    anyhow::bail!(e.clone());
                }
                _ => {
                    debug!(session_id=%self.uniq_id,"connect success")
                }
            },
            Err(e) => {
                error!(session_id=%self.uniq_id,"handler_ssh receive abort signal error: {:?}", e);
                anyhow::bail!(e.to_string());
            }
        };
        anyhow::Ok(())
    }
}

impl ProcessManger {
    pub async fn do_recording(&self, recv: EventSubscription<Bytes>) {
        let mut abort_tx: watch::Receiver<bool> = self.abort_rc.clone();
        let mut receiver = recv.unbox();
        if let Some(mut recorder) = self.recorder.lock().await.take() {
            loop {
                select! {
                    flag = abort_tx.changed() => match flag {
                            Ok(_) => {
                                if *abort_tx.borrow() {
                                    debug!(session_id=%self.uniq_id,"do_recording receive abort signal");
                                    break ;
                                }
                            },
                            Err(e) => {
                                error!(session_id=%self.uniq_id,"do_recording receive abort signal error: {:?}",e);
                                break ;
                            },
                        },
                    _ = tokio::time::sleep(Duration::from_secs(3)) => {
                        match recorder.flush() {
                                Ok(_) => {},
                                Err(e) => {
                                    error!(session_id=%self.uniq_id,"do_recording flush error: {:?}",e);
                                    break;
                                }
                        }
                    }
                    rb = receiver.recv() => match rb {
                        None  => {
                            self.stop_process();
                        },
                        Some(bytes)=> {
                            match recorder.write_all(bytes.as_ref()) {
                                Ok(_) => {},
                                Err(e) => {
                                    error!(session_id=%self.uniq_id,"do_recording write error: {:?}",e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            recorder.close();
        }
        debug!(session_id=%self.uniq_id,"do_recording end");
    }
    #[allow(dead_code)]
    pub async fn run(
        &mut self,
        uuid: Uuid,
        ssh_option: TargetSSHOptions,
    ) -> anyhow::Result<genesis_common::TaskStatusEnum> {
        let (hub, sender, notify) = start_ssh_connect(uuid, ssh_option).await?;
        // step1. wait until ssh connected
        self.wait_ssh_state(notify).await?;
        // step2. Two-way binary stream copy
        let receiver = hub.subscribe(|_| true).await;
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        let (psc, _) = unbounded_channel::<Bytes>();
        let in_pipe = Pipe::new(psc, in_rc);
        let out_pipe = Pipe::new(sender, receiver.unbox());
        let manager = PipeManger::new(self.ssh_cmd_wait_times, self.uniq_id.clone());
        // step3.start interactive
        let new_manager = Arc::new(manager);
        let _ = new_manager
            .clone()
            .do_interactive(
                in_pipe,
                out_pipe,
                self.broadcast_sender.clone(),
                self.abort_rc.clone(),
                self.get_abort_sc(),
            )
            .await;
        // step5. cmd & recording process
        let _ = tokio::join!(
            self.do_cmd_process(sc, new_manager.out_buf.clone(), new_manager.state.clone()),
            self.do_recording(hub.subscribe(|_| true).await)
        );
        // step6. stop type check
        let old = self.abort_rc.clone();
        if *old.borrow() {
            self.get_execute_info()
                .await
                .map(|s| anyhow::bail!(s))
                .unwrap_or(anyhow::Ok(TaskStatusEnum::ManualStop))
        } else {
            self.stop_process();
            debug!(session_id=%self.uniq_id,"end execute:{}", self.uniq_id);
            anyhow::Ok(TaskStatusEnum::Success)
        }
    }

    async fn do_cmd_process(
        &self,
        sc: UnboundedSender<Bytes>,
        res: Arc<Mutex<vt100::Parser>>,
        state: Arc<RwLock<PipeState>>,
    ) {
        let (cmd_sender, mut cmd_executor) = unbounded_channel();
        // 初始数据发送
        cmd_sender.send(self.execute.clone()).unwrap();
        let mut abort_execute_cmd = self.abort_rc.clone();
        loop {
            select! {
                flag = abort_execute_cmd.changed() => match flag {
                    Ok(_) => {
                        if *abort_execute_cmd.borrow() {
                            debug!(session_id=%self.uniq_id,"loop receive abort signal");
                            return;
                        }
                    },
                    Err(e) => {
                        debug!(session_id=%self.uniq_id,"loop receive abort signal error: {:?}",e);
                        return
                    }, // 如果接收到错误，也退出
                },
                ma = cmd_executor.recv() => match ma {
                        Some(execute) => {
                        let exe = execute.lock().await.clone();
                        let execute_node_info = format!("node[id:{} des:{}]",exe.node.id,exe.node.core.des);
                        let mut cmd = exe.node.core.cmd.clone();
                        let node_id = exe.node.id.clone();
                        if !cmd.ends_with('\r') {
                            cmd.push('\r');
                        }
                        // 添加执行参数
                        self.insert_global_params(format!("node-{}-cmd-input", node_id),cmd.clone()).await;
                        // 发送命令到远程执行
                        debug!(session_id=%self.uniq_id,"send node:{} cmd:{}", node_id, cmd);
                        let _ = sc.send(cmd.into());
                        // 超时配置校验
                        if exe.node.core.expire > 0 {
                            self.set_cmd_expire_time(exe.node.core.expire).await;
                        }
                        // 执行完毕,根据子节点配置pre数据,判断需要走哪条分支
                        if exe.children.is_empty() {
                            return;
                        }
                        let execute_fns = self.do_next_match(exe).await;
                        //存在子节点,等待子节点匹配
                        if self.cmd_wait_loop(&node_id,res.clone(),state.clone(),&execute_fns,&cmd_sender).await{
                            // 超时记录
                            self.set_execute_info(format!("execute expired for node:{}",execute_node_info)).await;
                            // 发送停止信号
                            self.stop_process();
                            break;
                        }
                    }
                    None => {
                        self.stop_process();
                        debug!(session_id=%self.uniq_id,"do_cmd_process cmd_executor receive none");
                        break;
                    }
                }
            }
        }
        debug!(session_id=%self.uniq_id,"do_cmd_process end");
    }

    async fn cmd_wait_loop(
        &self,
        node_id: &str,
        res: Arc<Mutex<vt100::Parser>>,
        state: Arc<RwLock<PipeState>>,
        execute_fns: &RwLock<Vec<ExecuteFns>>,
        cmd_sender: &UnboundedSender<Arc<Mutex<Execute>>>,
    ) -> bool {
        let mut abort_execute_cmd = self.abort_rc.clone();
        let mut subscribe = self.register_state_watcher();
        let mut expired = false;
        loop {
            select! {
                flag = abort_execute_cmd.changed() => match flag {
                    Ok(_) => {
                        if *abort_execute_cmd.borrow() {
                            debug!(session_id=%self.uniq_id,"cmd execute loop receive abort signal");
                            break; // 如果收到中止命令，退出
                        }
                    },
                    Err(e) => {
                        error!(session_id=%self.uniq_id,"cmd execute loop receive abort signal error: {:?}",e);
                        break
                    },
                },
                _ = tokio::time::sleep(Duration::from_secs(3)) => {
                    if self.check_cmd_expire_time().await {
                        expired = true;
                        break;
                    }
                    let content = &res.lock().await.screen().contents(); // 获取屏幕内容
                    debug!(session_id=%self.uniq_id,"receive content: {}", content);
                    self.insert_global_params(
                                format!("node-{}-cmd-output",node_id),content.clone()).await;
                    match self.process_execute_fns(execute_fns, cmd_sender, &state).await{
                        Ok(_) => {
                            debug!(session_id=%self.uniq_id,"time stop loop");
                            break;
                        },
                        Err(_) => {
                            debug!(session_id=%self.uniq_id,"time all not match content:{}\n",content);
                        },
                    }
                },
                ma = subscribe.recv() => match ma {
                    Ok(execute_state) => match execute_state{
                         ExecuteState::ExecutedCmd(md) => {
                            if self.check_cmd_expire_time().await {
                                expired = true;
                                break;
                            }
                            let content = md.output.clone();
                            debug!(session_id=%self.uniq_id,"receive cmd: {:?}", md);
                            self.insert_global_params(
                                format!("node-{}-cmd-output",node_id),content.clone()).await;
                            match self.process_execute_fns(execute_fns, cmd_sender, &state).await{
                                Ok(_) => {
                                    debug!(session_id=%self.uniq_id,"cmd stop loop");
                                    break;
                                },
                                Err(_) => {
                                    error!(session_id=%self.uniq_id,"cmd all not match content:{}\n",content);
                                },
                            }
                        }
                        ExecuteState::End(_)=> {
                            debug!(session_id=%self.uniq_id,"receive execute end signal stop.");
                            break
                        }
                        _ => {}
                        },
                    Err(e)=>{
                       error!(session_id=%self.uniq_id,"receive execute end signal err: {:?}",e);
                    }
                }
            }
        }
        expired
    }

    async fn process_execute_fns(
        &self,
        execute_fns: &RwLock<Vec<ExecuteFns>>,
        cmd_sender: &UnboundedSender<Arc<Mutex<Execute>>>,
        state: &RwLock<PipeState>,
    ) -> anyhow::Result<()> {
        let efn = execute_fns.read().await;
        if efn.is_empty() {
            return anyhow::Ok(());
        }
        for fnn in efn.iter() {
            if self.check_conditions(&fnn.fns).await {
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

    fn item_build(&self, node_id: &str, item: Item) -> AsyncMatchFn {
        string::cmd_string_match(item.match_type, node_id.to_string(), item.value.clone())
    }

    async fn do_next_match(&self, exe: Execute) -> RwLock<Vec<ExecuteFns>> {
        // children存在,组装出ExecuteFns
        let mut execute_fns = Vec::new();
        // 整理子节点
        for children_node_arc in exe.children {
            let mm_fn = children_node_arc
                .lock()
                .await
                .clone()
                .node
                .pre
                .map(|pre| {
                    pre.list
                        .into_iter()
                        .map(|d| self.item_build(&exe.node.id, d))
                        .collect()
                })
                .unwrap_or(vec![]);
            execute_fns.push(ExecuteFns {
                fns: mm_fn,
                execute: children_node_arc,
            });
        }
        RwLock::new(execute_fns)
    }

    async fn check_conditions(&self, fns: &[AsyncMatchFn]) -> bool {
        for ef in fns {
            let x = ef;
            match (*x)(self.global_params.clone()).await {
                Ok(v) => match v {
                    true => continue,
                    false => return false,
                },
                Err(e) => {
                    error!(session_id=%self.uniq_id,"match item err: {:?}",e);
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::{iter::once, time::Duration};

    use bytes::Bytes;
    use genesis_common::TargetSSHOptions;
    use genesis_ssh::start_ssh_connect;
    use tokio::sync::{mpsc::unbounded_channel, watch};
    use tracing::info;
    use uuid::Uuid;

    use super::PipeManger;
    use crate::{Core, Edge, Item, Node, Pipe, Position, Pre};

    use super::*;
    use crate::common::em::PreMatchTypeEnum;
    use crate::{Graph, InData};
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
                        expire: 0,
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
                        expire: 0,
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
                        expire: 0,
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
                        expire: 0,
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
                        expire: 0,
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
                        expire: 0,
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
                        expire: 0,
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

        let mut pm = ProcessManger::new("".to_string(), execute).unwrap();

        let option = TargetSSHOptions::default();
        let _ = pm.run(Uuid::new_v4(), option).await;
    }
    #[tokio::test]
    #[ignore]
    async fn test_parser_manager() {
        tracing_subscriber::fmt().init();
        let uuid = Uuid::new_v4();
        let option = TargetSSHOptions::default();
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

        let manager = PipeManger::default();
        let (abort_sc, abort_rc) = watch::channel(false);
        let (scc, _rc) = broadcast::channel(16);
        let new_manager = Arc::new(manager);
        let ma = new_manager.do_interactive(
            in_pipe,
            out_pipe,
            scc.clone(),
            abort_rc.clone(),
            abort_sc.clone(),
        );

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
            let _ = abort_sc.send(true);
            info!("end pwd");
        });
        // let b = tokio::spawn(async move {
        //     while let Some(cmd) = cmd_rc.recv().await {
        //         info!("receive cmd info:{:?}", cmd);
        //     }
        // });
        let _ = tokio::join!(ma, a);
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
