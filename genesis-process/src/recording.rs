use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const RECORDING_CAST: &str = "recording.cast";
pub const SSH_KIND: &str = "ssh";

#[derive(Debug, Serialize, Deserialize)]
struct Env {
    #[serde(rename = "SHELL")]
    shell: String,
    #[serde(rename = "TERM")]
    term: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    title: String,
    version: i32,
    height: i32,
    width: i32,
    env: Env,
    #[serde(rename = "Timestamp")]
    timestamp: i64,
}

pub struct Recorder {
    file: Option<File>,
    timestamp: i64,
    uniq_id: String,
}

impl Recorder {
    pub fn new(
        uniq_id: &str,
        root_path: &str,
        term: &str,
        height: i32,
        width: i32,
    ) -> Result<Self> {
        let path = PathBuf::from(root_path)
            .join(SSH_KIND)
            .join(uniq_id)
            .join(RECORDING_CAST);

        let file = create_file(&path).context("Failed to create recording file")?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time before UNIX epoch")?
            .as_secs() as i64;

        let header = Header {
            title: String::new(),
            version: 2,
            height,
            width,
            env: Env {
                shell: "/bin/bash".to_string(),
                term: term.to_string(),
            },
            timestamp,
        };

        let mut recorder = Recorder {
            file: Some(file),
            timestamp,
            uniq_id: uniq_id.to_string(),
        };

        recorder.write_header(&header)?;
        Ok(recorder)
    }

    fn write_header(&mut self, header: &Header) -> Result<()> {
        let json = serde_json::to_vec(header)?;
        if let Some(file) = &mut self.file {
            file.write_all(&json)?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    fn write_data(&mut self, data: &str) -> Result<()> {
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
}

impl io::Write for Recorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let data_str = String::from_utf8(buf.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if data_str.len() > self.uniq_id.len() && data_str.starts_with(&self.uniq_id) {
            return Ok(buf.len());
        }

        self.write_data(&data_str)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn create_file(path: &Path) -> Result<File> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid path"))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn basic_functionality() -> Result<()> {
        let mut recorder = Recorder::new("test_id", "./", "xterm-256color", 80, 24)?;

        // Test writing data
        let data = "test data";
        Write::write_all(&mut recorder, data.as_bytes())?;

        recorder.close();
        Ok(())
    }
}
