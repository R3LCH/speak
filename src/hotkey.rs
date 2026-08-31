use anyhow::Result;
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub alt: bool,
    pub pressed: bool,
    pub timestamp_ms: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Activation;
pub struct DoubleTapDetector {
    last: u64,
    waiting: bool,
    window: u64,
}
impl DoubleTapDetector {
    pub fn new(window: u64) -> Self {
        Self {
            last: 0,
            waiting: false,
            window,
        }
    }
    pub fn feed(&mut self, e: KeyEvent) -> Option<Activation> {
        if !e.alt || !e.pressed {
            return None;
        }
        if self.waiting && e.timestamp_ms - self.last <= self.window {
            self.waiting = false;
            Some(Activation)
        } else {
            self.waiting = true;
            self.last = e.timestamp_ms;
            None
        }
    }
}
pub trait HotkeyBackend {
    fn next_activation(&mut self) -> Result<Activation>;
}
