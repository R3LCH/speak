use anyhow::Result;
pub fn available() -> bool {
    std::env::var("DISPLAY").is_ok()
}
pub fn paste_command() -> Result<()> {
    Ok(())
}
