use anyhow::Result;
pub fn available() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}
pub fn check_uinput() -> Result<()> {
    if std::path::Path::new("/dev/uinput").exists() {
        Ok(())
    } else {
        anyhow::bail!("/dev/uinput unavailable; grant uinput permissions")
    }
}
