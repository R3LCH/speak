use crate::config::Config;
use anyhow::Result;
pub fn doctor(_: &Config) -> Result<()> {
    let session = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        "wayland"
    } else if std::env::var("DISPLAY").is_ok() {
        "x11"
    } else {
        "none"
    };
    println!("session: {}", session);
    if session == "none" {
        anyhow::bail!("no X11 or Wayland session detected")
    }
    Ok(())
}
pub fn client_command(_: &str) -> Result<()> {
    Ok(())
}
