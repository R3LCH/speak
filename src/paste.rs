use anyhow::Result;
pub trait PasteBackend {
    fn paste(&mut self, text: &str) -> Result<()>;
}
pub fn normalize_transcription(text: &str) -> Option<String> {
    let s = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
pub struct CommandPaste {
    wayland: bool,
}
impl CommandPaste {
    pub fn detect() -> Self {
        Self {
            wayland: std::env::var("WAYLAND_DISPLAY").is_ok(),
        }
    }
}
impl PasteBackend for CommandPaste {
    fn paste(&mut self, text: &str) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        if self.wayland {
            let mut c = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;
            c.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            c.wait()?;
            Command::new("ydotool")
                .args(["key", "29:1", "47:1", "47:0", "29:0"])
                .status()?;
        } else {
            let mut c = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()?;
            c.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            c.wait()?;
            Command::new("xdotool").args(["key", "ctrl+v"]).status()?;
        }
        Ok(())
    }
}
