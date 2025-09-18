use bytes::{Bytes, BytesMut};
use derive_builder::Builder;
use std::iter::once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use vt100::Parser;

const CLEAN_SCREEN: &[u8] = b"\x1b[2J";
/// cmd execute state enum
#[derive(Clone)]
pub enum ExecuteState {
    /// start with executeId
    Start(String),
    /// end with executeId
    End(String),
    /// executed with instructId
    Executed((String, String)),
    /// Binary data returned by the SSH channel
    ExecutedBytes(Bytes),
    /// executed cmd with in/out
    ExecutedCmd(PipeCmd),
}

#[derive(Clone, Default)]
pub enum PipeState {
    In,
    #[default]
    Out,
}
#[derive(Clone, Debug, Default)]
pub struct PipeCmd {
    pub input: String,
    pub output: String,
}

#[allow(dead_code)]
#[derive(Default, Builder)]
#[builder(build_fn(private, name = "private_build"))]
#[builder(setter(into), default)]
pub struct PipeManger {
    wait_times: u8,
    uniq_id: String,
    ctx: CancellationToken,
    ps1: Arc<RwLock<String>>,
    parser: Arc<Mutex<Parser>>,
    state: Arc<RwLock<PipeState>>,
    out_buf: Arc<Mutex<Parser>>,
    input_buf: Arc<Mutex<Parser>>,
    alternate_mode: Arc<RwLock<bool>>,
    ps1_char: Arc<Vec<char>>,
    counter: Arc<AtomicUsize>,
}

impl PipeMangerBuilder {
    pub fn build(&mut self) -> anyhow::Result<PipeManger> {
        if self.ps1_char.is_none() {
            self.ps1_char = Some(Arc::new(vec!['#', '$', '>']))
        }
        Ok(self.private_build()?)
    }
}

impl PipeManger {
    /// # 启用ssh处理通道
    /// 返回发送数据的sender,以及接受返回数据的receiver
    pub fn do_interactive(
        self: Arc<Self>,
        sender: UnboundedSender<Bytes>,
        receiver: UnboundedReceiver<Bytes>,
    ) -> anyhow::Result<(UnboundedSender<Bytes>, Receiver<ExecuteState>)> {
        let in_self = self.clone();
        let in_ctx = self.ctx.clone();
        let (sc, in_rc) = unbounded_channel::<Bytes>();
        tokio::spawn(async move {
            in_self.do_process_in(in_ctx, sender, in_rc).await;
        });
        // step2. process ssh server response data
        let out_self = self.clone();
        let dpo_ctx = self.ctx.clone();
        let (bs, br) = broadcast::channel::<ExecuteState>(1024);
        tokio::spawn(async move {
            out_self.do_process_out(dpo_ctx, receiver, bs).await;
        });
        Ok((sc, br))
    }

    /// # 处理用户输入
    /// ## 入参
    /// - sender: 发送到服务器
    /// - reader: 读取用户输入的数据
    async fn do_process_in(
        &self,
        ctx: CancellationToken,
        sender: UnboundedSender<Bytes>,
        mut reader: UnboundedReceiver<Bytes>,
    ) {
        let ps1 = self.ps1.clone();
        let state = self.state.clone();
        let alternate_mode = self.alternate_mode.clone();
        let wait_times = self.wait_times;
        loop {
            select! {
                _ = ctx.cancelled() => {
                    debug!(session_id=%self.uniq_id,"do_process_in receive abort signal");
                    return ;
                }
                rb = reader.recv() => match rb {
                    Some(data) => {
                        // 未设置ps1 不允许输入
                        while ps1.read().await.is_empty() {
                            tokio::time::sleep(Duration::from_millis(20)).await;
                        }
                        // 判断输入状态是否是允许输入
                        let mut now_wait_time = 0;
                        // 不在vim等交互模式
                        if !*alternate_mode.read().await {
                            loop {
                                match *state.read().await {
                                    PipeState::In => break,
                                    PipeState::Out => {
                                        tokio::time::sleep(Duration::from_millis(20)).await;
                                        now_wait_time += 1;
                                        if now_wait_time >= wait_times {
                                            debug!(session_id=%self.uniq_id,"do_process_in time out, break");
                                            now_wait_time = 0;
                                            break;
                                        }
                                    },
                                }
                            }
                        }
                        // 输入数据最后一个字符是\r
                        if data.last() == Some(&b'\r'){
                            // 等待命令前字符
                            while self.counter.load(Ordering::SeqCst) !=0{
                                tokio::time::sleep(Duration::from_millis(20)).await;
                                now_wait_time += 1;
                                if now_wait_time >= wait_times {
                                    debug!(session_id=%self.uniq_id,"do_process_in counter time out, break");
                                    break;
                                }
                            }
                            if data.len() > 1{
                                let split_index = data.len() - 1;
                                {
                                    let before = data.slice(0..split_index);
                                    *state.write().await = PipeState::In;
                                    self.counter.fetch_add(1, Ordering::SeqCst);
                                    let _ = sender.send(before);
                                }
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                {
                                    *state.write().await = PipeState::Out;
                                    self.counter.fetch_add(1, Ordering::SeqCst);
                                    let _ = sender.send(Bytes::from_static(b"\r"));
                                }
                            }else{
                                *state.write().await = PipeState::Out;
                                self.counter.fetch_add(1, Ordering::SeqCst);
                                let _ = sender.send(Bytes::from_static(b"\r"));
                            }
                        } else {
                            // 正常处理
                            *state.write().await = PipeState::In;

                            if !self.is_cursor_position_report(&data) {
                                self.counter.fetch_add(1, Ordering::SeqCst);
                            }
                            let _ = sender.send(data);
                        }
                    },
                    None => {
                        debug!(session_id=%self.uniq_id,"do_process_in receive none");
                        break
                    },
                }
            }
        }
    }
    // 判断是否是上报光标位置二进制
    fn is_cursor_position_report(&self, buf: &[u8]) -> bool {
        // 以 ESC + '[' 开头，且以 'R' 结尾
        if buf.len() < 5 {
            return false;
        }
        if buf[0] != 0x1b || buf[1] != b'[' || buf[buf.len() - 1] != b'R' {
            return false;
        }

        // 中间必须包含一个 ';' 且全是 ASCII 数字和分号
        let middle = &buf[2..buf.len() - 1];
        let mut has_semicolon = false;

        for &b in middle {
            match b {
                b'0'..=b'9' => continue,
                b';' => {
                    if has_semicolon {
                        // 多个 ';' 不符合
                        return false;
                    }
                    has_semicolon = true;
                }
                _ => return false,
            }
        }

        has_semicolon
    }
    /// # 处理服务端返回数据
    /// 此方法处理ssh服务端返回数据,原始二进制数据通过ExecutedBytes返回,命令数据通过ExecutedCmd返回
    /// ## 入参数
    /// - reader: 从服务端读取数据
    async fn do_process_out(
        &self,
        ctx: CancellationToken,
        mut reader: UnboundedReceiver<Bytes>,
        broadcast: Sender<ExecuteState>,
    ) {
        let parser = self.parser.clone();
        let ps1 = self.ps1.clone();
        let stat = self.state.clone();
        let input = self.input_buf.clone();
        let output = self.out_buf.clone();
        let alternate_mode = self.alternate_mode.clone();
        let mut buffer = BytesMut::new();
        'ro: loop {
            select! {
                rb = reader.recv() => match rb {
                    Some(data) => {
                        self.counter.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                            if x > 0 {
                                Some(x - 1)
                            } else {
                                None
                            }
                        }).ok();
                        // step1. send ssh server data to broadcast
                        let _ = broadcast.send(ExecuteState::ExecutedBytes(data.clone()));
                        // step2. remove redundancy data
                        let parts: Vec<Bytes> = data
                            .split(|&byte| byte == b'\r')
                            .flat_map(|part| once(Bytes::copy_from_slice(part)).chain(once(Bytes::from_static(b"\r"))))
                            .take(data.split(|&byte| byte == b'\r').count() * 2 - 1) // 去掉最后一个多余的 \r
                            .filter(|e| !e.is_empty())
                            .collect();
                        // loop
                        for data in parts.iter() {
                            let mut par = parser.lock().await;
                            par.process(data);
                            if par.screen().alternate_screen() {
                                //VIM等界面
                                let mut x = alternate_mode.write().await;
                                *x = true;
                                continue 'ro;
                            }
                            // 判断当前接受状态,若是输入状态,则写到input
                            // 若是输出状态,则写入到output
                            match *(stat.read().await) {
                                PipeState::In => {
                                    input.lock().await.process(data);
                                },
                                PipeState::Out => {
                                    output.lock().await.process(data);
                                },
                            }
                            buffer.extend_from_slice(data);
                            if let Some(p1) = self.extract_command_after_bell(&buffer){
                                self.counter.store(0,Ordering::SeqCst);
                                *(ps1.write().await) = String::from_utf8_lossy(p1).to_string();
                                // 打印ps1,并清空
                                *(stat.write().await) = PipeState::In;
                                buffer.clear();

                                let mut input = input.lock().await;
                                let cmd_input = input.screen().contents().trim().to_string();
                                if cmd_input.is_empty() {
                                    output.lock().await.process(CLEAN_SCREEN);
                                    continue;
                                }else{
                                    input.process(CLEAN_SCREEN);
                                    let mut output = output.lock().await;
                                    let ps1_value = ps1.read().await;
                                    let cmd_out = output.screen().contents().replace(ps1_value.as_str(), "").trim().to_string();
                                    output.process(CLEAN_SCREEN);
                                    let _ = broadcast.send(ExecuteState::ExecutedCmd(PipeCmd{
                                        input: cmd_input,
                                        output: cmd_out,
                                    }));
                                }
                            }
                        }
                        // 处理完毕,发送数据
                        let mut x = alternate_mode.write().await;
                        *x = false;
                    },
                    None => {return ;},
                },
                _ = ctx.cancelled() => {
                    debug!(session_id=%self.uniq_id,"do_process_out receive abort signal");
                    return ;
                }
            }
        }
    }
    fn extract_command_after_bell<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        let lines = data.split(|&b| b == b'\n');

        for line in lines.rev() {
            if line.is_empty() || !line.contains(&0x1b) {
                continue;
            }
            let mut parser = Parser::default();
            parser.process(line);
            if let Some(c) = parser.screen().contents().trim().chars().next_back() {
                if self.ps1_char.contains(&c) {
                    return Some(line);
                }
            }
        }
        None
    }
}
