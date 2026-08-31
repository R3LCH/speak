use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionRequest {
    pub id: String,
    pub audio_path: String,
    pub model: String,
    pub device: String,
    pub compute_type: String,
    pub language: Option<String>,
    pub beam_size: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub id: String,
    pub ok: bool,
    pub text: Option<String>,
    pub error: Option<String>,
    pub device_used: String,
}
pub struct WorkerClient {
    child: Child,
}
impl WorkerClient {
    pub fn start(config: &Config) -> Result<Self> {
        let mut p = config.worker_command.split_whitespace();
        let exe = p.next().context("empty worker command")?;
        let child = Command::new(exe)
            .args(p)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("start worker")?;
        Ok(Self { child })
    }
    pub fn transcribe(&mut self, req: TranscriptionRequest) -> Result<TranscriptionResponse> {
        let line = serde_json::to_string(&req)?;
        self.child
            .stdin
            .as_mut()
            .context("worker stdin")?
            .write_all(format!("{}\n", line).as_bytes())?;
        self.child.stdin.as_mut().unwrap().flush()?;
        let stdout = self.child.stdout.as_mut().context("worker stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut out = String::new();
        reader.read_line(&mut out)?;
        serde_json::from_str(out.trim()).context("parse worker response")
    }
    pub fn restart(&mut self) -> Result<()> {
        self.child.kill().ok();
        Ok(())
    }
}
