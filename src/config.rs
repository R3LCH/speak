use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DevicePolicy {
    #[default]
    Auto,
    Cpu,
    Cuda,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComputeType {
    #[default]
    Auto,
    Int8,
    Int8Float16,
    Float16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub device: DevicePolicy,
    #[serde(default)]
    pub compute_type: ComputeType,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_beam")]
    pub beam_size: u32,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_tap_ms")]
    pub double_tap_ms: u64,
    #[serde(default = "default_audio")]
    pub audio_device: String,
    #[serde(default = "default_worker")]
    pub worker_command: String,
    #[serde(default)]
    pub model_cache: Option<PathBuf>,
}
fn default_model() -> String {
    "small".into()
}
fn default_language() -> String {
    "auto".into()
}
fn default_beam() -> u32 {
    1
}
fn default_hotkey() -> String {
    "alt-double-tap".into()
}
fn default_tap_ms() -> u64 {
    300
}
fn default_audio() -> String {
    "default".into()
}
fn default_worker() -> String {
    "python3 -m speak_worker".into()
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: default_model(),
            device: Default::default(),
            compute_type: Default::default(),
            language: default_language(),
            beam_size: 1,
            hotkey: default_hotkey(),
            double_tap_ms: 300,
            audio_device: default_audio(),
            worker_command: default_worker(),
            model_cache: None,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.model.trim().is_empty() {
            bail!("model must not be empty")
        }
        if self.beam_size == 0 {
            bail!("beam_size must be greater than zero")
        }
        if self.double_tap_ms == 0 || self.double_tap_ms > 5000 {
            bail!("double_tap_ms must be between 1 and 5000")
        }
        if self.hotkey != "alt-double-tap" && self.hotkey.trim().is_empty() {
            bail!("hotkey must not be empty")
        }
        Ok(())
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        let s = fs::read_to_string(p).with_context(|| format!("read config {}", p.display()))?;
        let c: Self = toml::from_str(&s).context("parse TOML config")?;
        c.validate()?;
        Ok(c)
    }
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("speak/config.toml")
    }
}
