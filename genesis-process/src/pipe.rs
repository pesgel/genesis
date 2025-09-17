use bytes::{Bytes, BytesMut};
use std::iter::once;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch::{self, Receiver};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

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
    pub ps1: Arc<RwLock<String>>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub state: Arc<RwLock<PipeState>>,
    pub out_buf: Arc<Mutex<vt100::Parser>>,
    pub input_buf: Arc<Mutex<vt100::Parser>>,
    pub wait_times: u8,
    pub uniq_id: String,
    alternate_mode: Arc<RwLock<bool>>,
    ps1_char: Arc<Vec<char>>,
    counter: Arc<std::sync::atomic::AtomicUsize>,
}

impl PipeManger {
    pub fn with_ps1_char(&mut self, chars: Vec<char>) -> &mut Self {
        self.ps1_char = Arc::new(chars);
        self
    }
    pub fn new(wait_times: u8, uniq_id: String) -> Self {
        Self {
            wait_times,
            uniq_id,
            ps1: Arc::new(Default::default()),
            parser: Arc::new(Default::default()),
            state: Arc::new(Default::default()),
            out_buf: Arc::new(Default::default()),
            input_buf: Arc::new(Default::default()),
            alternate_mode: Arc::new(RwLock::new(false)),
            ps1_char: Arc::new(vec!['#', '$', '>']),
            counter: Arc::new(Default::default()),
        }
    }

    pub async fn do_process_in(
        &self,
        ctx: CancellationToken,
        out_io_sender: UnboundedSender<Bytes>,
        mut in_io_reader: UnboundedReceiver<Bytes>,
    ) {
        let ps1 = self.ps1.clone();
        let state = self.state.clone();
        let wait_times = self.wait_times;
        let alternate_mode = self.alternate_mode.clone();
        loop {
            select! {
                _ = ctx.cancelled() => {
                    debug!(session_id=%self.uniq_id,"do_process_in receive abort signal");
                    return ;
                }
                rb = in_io_reader.recv() => match rb {
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
                                    let _ = out_io_sender.send(before);
                                }
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                {
                                    *state.write().await = PipeState::Out;
                                    self.counter.fetch_add(1, Ordering::SeqCst);
                                    let _ = out_io_sender.send(Bytes::from_static(b"\r"));
                                }
                            }else{
                                *state.write().await = PipeState::Out;
                                self.counter.fetch_add(1, Ordering::SeqCst);
                                let _ = out_io_sender.send(Bytes::from_static(b"\r"));
                            }
                        } else {
                            // 正常处理
                            *state.write().await = PipeState::In;

                            if !self.is_cursor_position_report(&data) {
                                self.counter.fetch_add(1, Ordering::SeqCst);
                            }
                            let _ = out_io_sender.send(data);
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

    pub async fn do_process_out(
        &self,
        ctx: CancellationToken,
        in_io_sender: UnboundedSender<Bytes>,
        mut out_io_reader: UnboundedReceiver<Bytes>,
        state_sender: broadcast::Sender<ExecuteState>,
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
                rb = out_io_reader.recv() => match rb {
                    Some(data) => {
                        self.counter.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                            if x > 0 {
                                Some(x - 1)
                            } else {
                                None
                            }
                        }).ok();
                        // step1. send ssh server data to broadcast
                        let _ = state_sender.send(ExecuteState::ExecutedBytes(data.clone()));
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
                                    output.lock().await.process(b"\x1b[2J");
                                    continue;
                                }else{
                                    input.process(b"\x1b[2J");
                                    let mut output = output.lock().await;
                                    let ps1_value = ps1.read().await;
                                    let cmd_out = output.screen().contents().replace(ps1_value.as_str(), "").trim().to_string();
                                    output.process(b"\x1b[2J");
                                    let _ = state_sender.send(ExecuteState::ExecutedCmd(PipeCmd{
                                        input: cmd_input,
                                        output: cmd_out,
                                    }));
                                }
                            }
                        }
                        // 处理完毕,发送数据
                        let mut x = alternate_mode.write().await;
                        *x = false;
                        let _ = in_io_sender.send(data);
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
            let mut parser = vt100::Parser::default();
            parser.process(line);
            if let Some(c) = parser.screen().contents().trim().chars().next_back() {
                if self.ps1_char.contains(&c) {
                    return Some(line);
                }
            }
        }
        None
    }
    pub async fn do_interactive(
        self: Arc<Self>,
        in_io: Pipe,
        out_io: Pipe,
        state_sender: broadcast::Sender<ExecuteState>,
        ctx: CancellationToken,
    ) -> Result<(), String> {
        // step1. receive in data
        let in_self = self.clone();
        let dpi_ctx = ctx.clone();
        tokio::spawn(async move {
            in_self
                .do_process_in(dpi_ctx, out_io.sender, in_io.reader)
                .await;
        });
        // step2. process ssh server response data
        let out_self = self.clone();
        let dpo_ctx = ctx.clone();
        tokio::spawn(async move {
            out_self
                .do_process_out(dpo_ctx, in_io.sender, out_io.reader, state_sender)
                .await;
        });
        Ok(())
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
