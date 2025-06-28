use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::fmt;
use std::io::{self};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub const DELIMITER: u8 = b';';
pub const VERSION: &str = "1.0.0"; // mock version
pub const TUNNEL_CLOSED: i32 = 1000; // mock error code

// --- Configuration struct ---
#[derive(Debug, Clone)]
pub struct Configuration {
    pub connection_id: String,
    pub protocol: String,
    pub parameters: HashMap<String, String>,
}

impl Configuration {
    pub fn new(pt: &str) -> Self {
        Self {
            connection_id: String::new(),
            protocol: pt.to_string(),
            parameters: HashMap::new(),
        }
    }

    pub fn set_read_only_mode(&mut self) {
        self.parameters.insert("read-only".into(), "true".into());
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.parameters.insert(name.to_string(), value.to_string());
    }

    pub fn with(mut self, name: &str, value: &str) -> Self {
        self.parameters.insert(name.to_string(), value.to_string());
        self
    }

    pub fn unset(&mut self, name: &str) {
        self.parameters.remove(name);
    }

    pub fn get(&self, name: &str) -> String {
        self.parameters.get(name).cloned().unwrap_or_default()
    }
}

// --- Instruction struct ---
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: String,
    pub args: Vec<String>,
    pub protocol_form: Option<String>,
}

impl Instruction {
    pub fn new(opcode: &str, args: Vec<&str>) -> Self {
        Self {
            opcode: opcode.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
            protocol_form: None,
        }
    }

    pub fn new_error(msg: &str) -> Self {
        Self {
            opcode: "error".into(),
            args: vec![
                general_purpose::STANDARD.encode(msg),
                TUNNEL_CLOSED.to_string(),
            ],
            protocol_form: None,
        }
    }

    pub fn new_msg(code: &str, msg: &str) -> Self {
        Self {
            opcode: "msg".into(),
            args: vec![code.to_string(), general_purpose::STANDARD.encode(msg)],
            protocol_form: None,
        }
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim_end_matches(';');
        let parts: Vec<&str> = s.split(',').collect();
        let mut args = Vec::new();

        for part in parts {
            let spl: Vec<&str> = part.splitn(2, '.').collect();
            if spl.len() != 2 {
                return None;
            }
            args.push(spl[1].to_string());
        }

        let opcode = args.first()?.clone();
        Some(Instruction::new(
            &opcode,
            args[1..].iter().map(|s| s.as_str()).collect(),
        ))
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!("{}.{}", self.opcode.len(), self.opcode);
        for arg in &self.args {
            output += &format!(",{}.{}", arg.len(), arg);
        }
        output += ";";
        write!(f, "{}", output)
    }
}

// --- Tunnel struct ---
pub struct Tunnel {
    pub uuid: Option<String>,
    pub is_open: Arc<AtomicBool>,
    pub reader: BufReader<TcpStream>,
    config: Configuration,
}

impl Tunnel {
    pub async fn connect(addr: &str, config: Configuration) -> io::Result<Self> {
        let conn = TcpStream::connect(addr).await?;
        let reader = BufReader::new(conn);
        let mut tunnel = Tunnel {
            uuid: None,
            is_open: Arc::new(AtomicBool::new(false)),
            reader,
            config,
        };

        let select_arg = if tunnel.config.connection_id.is_empty() {
            tunnel.config.protocol.clone()
        } else {
            tunnel.config.connection_id.clone()
        };

        tunnel
            .write_instruction(Instruction::new("select", vec![&select_arg]))
            .await?;
        let args_instr = tunnel.expect("args").await?;

        let width = tunnel.config.get("width");
        let height = tunnel.config.get("height");
        let dpi = tunnel.config.get("dpi");

        tunnel
            .write_instruction(Instruction::new("size", vec![&width, &height, &dpi]))
            .await?;
        tunnel
            .write_instruction(Instruction::new("audio", vec!["audio/L8", "audio/L16"]))
            .await?;
        tunnel
            .write_instruction(Instruction::new("video", vec![]))
            .await?;
        tunnel
            .write_instruction(Instruction::new(
                "image",
                vec!["image/jpeg", "image/png", "image/webp"],
            ))
            .await?;
        tunnel
            .write_instruction(Instruction::new("timezone", vec!["Asia/Shanghai"]))
            .await?;

        let mut parameters = vec![];
        for arg in &args_instr.args {
            if arg.contains("VERSION") {
                parameters.push(VERSION.to_string());
            } else {
                parameters.push(tunnel.config.get(arg));
            }
        }

        let param_refs: Vec<&str> = parameters.iter().map(|s| s.as_str()).collect();
        tunnel
            .write_instruction(Instruction::new("connect", param_refs))
            .await?;

        let ready_instr = tunnel.expect("ready").await?;
        if ready_instr.args.is_empty() {
            return Err(io::Error::other("no connection id received"));
        }

        tunnel.uuid = Some(ready_instr.args[0].clone());
        tunnel.is_open.store(true, Ordering::Relaxed);

        Ok(tunnel)
    }

    pub async fn close(&mut self) {
        self.is_open.store(false, Ordering::Relaxed);
        let _ = self.reader.get_mut().shutdown().await;
    }

    pub async fn write_instruction(&mut self, mut instr: Instruction) -> io::Result<()> {
        let msg = instr.to_bytes();
        self.reader.get_mut().write_all(&msg).await
    }

    pub async fn read_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.reader.read_until(DELIMITER, &mut buf).await?;
        Ok(buf)
    }

    pub async fn read_instruction(&mut self) -> io::Result<Instruction> {
        let bytes = self.read_bytes().await?;
        let s = String::from_utf8_lossy(&bytes);
        Instruction::parse(&s)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "parse failed"))
    }

    pub async fn expect(&mut self, opcode: &str) -> io::Result<Instruction> {
        let instr = self.read_instruction().await?;
        if instr.opcode != opcode {
            let msg = format!("expected '{}' but received '{}'", opcode, instr.opcode);
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }
        Ok(instr)
    }
}
