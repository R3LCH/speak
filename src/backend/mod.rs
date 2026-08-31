pub mod wayland;
pub mod x11;
use crate::config::Config;
use anyhow::Result;
pub struct BackendSet;
pub struct BackendFactory;
impl BackendFactory {
    pub fn detect(_: &Config) -> Result<BackendSet> {
        Ok(BackendSet)
    }
}
