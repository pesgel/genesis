use anyhow::{Context, Result};
use bytes::Bytes;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

pub const RECORDING_CAST: &str = "recording.cast";
pub const SSH_KIND: &str = "ssh";

const DEFAULT_SHELL: &str = "/bin/bash";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Env {
    #[serde(rename = "SHELL")]
    shell: String,
    #[serde(rename = "TERM")]
    term: String,
}

impl Env {
    fn new(shell: impl Into<String>, term: impl Into<String>) -> Self {
        Self {
            shell: shell.into(),
            term: term.into(),
        }
    }
}

#[derive(Debug, Serialize, Builder, Deserialize)]
#[builder(setter(into))]
struct Header {
    env: Env,
    width: u32,
    height: u32,
    version: u8,
    #[serde(rename = "Timestamp")]
    timestamp: i64,
}

#[derive(Debug, Builder)]
#[builder(build_fn(private, name = "private_build"))]
#[builder(setter(into))]
pub struct Recorder {
    uniq: String,
    path: String,
    term: String,
    height: u32,
    width: u32,
    #[builder(setter(skip))]
    timestamp: i64,
    #[builder(setter(skip))]
    file: Option<File>,
}
impl RecorderBuilder {
    pub fn build(&mut self) -> Result<Recorder> {
        let mut s = self.private_build()?;
        s.start()?;
        Ok(s)
    }
}
impl Recorder {
    pub fn start(&mut self) -> Result<&mut Self> {
        let path = PathBuf::from(self.path.as_str())
            .join(SSH_KIND)
            .join(self.uniq.as_str())
            .join(RECORDING_CAST);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system time before UNIX epoch")?
            .as_secs() as i64;
        self.timestamp = timestamp;
        self.file = Some(Self::create_file(&path).context("failed to create recording file")?);
        // 添加头数据
        let header = HeaderBuilder::default()
            .version(2)
            .height(self.height)
            .width(self.width)
            .env(Env::new(DEFAULT_SHELL, self.term.as_str()))
            .timestamp(timestamp)
            .build()?;
        self.write_header(&header)?;
        Ok(self)
    }
    fn write_header(&mut self, header: &Header) -> Result<()> {
        let json = serde_json::to_vec(header)?;
        if let Some(file) = &mut self.file {
            file.write_all(&json)?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn write_data(&mut self, data: &str) -> Result<()> {
        let now_nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as i64;
        let delta = (now_nanos - self.timestamp * 1_000_000_000) as f64 / 1_000_000_000.0;

        let row = vec![
            serde_json::Value::from(delta),
            serde_json::Value::from("o"),
            serde_json::Value::from(data),
        ];

        let json = serde_json::to_vec(&row)?;
        if let Some(file) = &mut self.file {
            file.write_all(&json)?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn close(&mut self) {
        if self.file.is_some() {
            let _ = self.write_data("end session");
            if let Some(file) = &mut self.file {
                let _ = file.sync_all();
            }
            self.file = None;
        }
    }
    fn create_file(path: &Path) -> Result<File> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("Invalid path"))?;

        if parent.exists() {
            return Err(
                io::Error::new(io::ErrorKind::AlreadyExists, "Directory already exists").into(),
            );
        }
        std::fs::create_dir_all(parent)?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(Into::into)
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        self.close();
    }
}

impl Write for Recorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let data_str = String::from_utf8(buf.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if data_str.len() > self.uniq.len() && data_str.starts_with(&self.uniq) {
            return Ok(buf.len());
        }

        self.write_data(&data_str).map_err(io::Error::other)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_create() {
        let mut recorder = RecorderBuilder::default()
            .uniq("123")
            .path("/tmp/rust")
            .term("xterm-256color")
            .height(80u32)
            .width(24u32)
            .build()
            .unwrap();
        let data = "test data";
        recorder.write_data(data).unwrap();
        recorder.close();
    }
}
