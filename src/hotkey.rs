use anyhow::Result;
use std::sync::mpsc::{self, Receiver};
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

pub struct GlobalHotkey {
    rx: Receiver<KeyEvent>,
    detector: DoubleTapDetector,
}
impl GlobalHotkey {
    pub fn start(window: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            if let Err(err) = listen_evdev(tx.clone()) {
                tracing::error!(error=?err, "evdev listener stopped; check /dev/input permissions");
            }
            if let Err(err) = rdev::listen(move |e| {
                use rdev::{EventType, Key};
                let (alt, pressed) = match e.event_type {
                    EventType::KeyPress(Key::Alt | Key::AltGr) => (true, true),
                    EventType::KeyRelease(Key::Alt | Key::AltGr) => (true, false),
                    _ => (false, false),
                };
                if alt {
                    let _ = tx.send(KeyEvent {
                        alt,
                        pressed,
                        timestamp_ms: e
                            .time
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    });
                }
            }) {
                tracing::error!(error=?err, "global key listener stopped; grant input-device access or run an X11-compatible session");
            }
        });
        Self {
            rx,
            detector: DoubleTapDetector::new(window),
        }
    }
}

fn listen_evdev(tx: mpsc::Sender<KeyEvent>) -> anyhow::Result<()> {
    use evdev::{Device, InputEventKind, Key};
    let paths = std::fs::read_dir("/dev/input")?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("event"))
                .unwrap_or(false)
        });
    let mut devices = Vec::new();
    for path in paths {
        if let Ok(d) = Device::open(&path) {
            let keyboard_name = d
                .name()
                .map(|n| n.to_ascii_lowercase().contains("keyboard"))
                .unwrap_or(false);
            if keyboard_name
                || d.supported_keys()
                    .map(|k| k.contains(Key::KEY_LEFTALT) || k.contains(Key::KEY_RIGHTALT))
                    .unwrap_or(false)
            {
                devices.push(d);
            }
        }
    }
    if devices.is_empty() {
        anyhow::bail!("no keyboard input device with Alt key found")
    }
    loop {
        for d in devices.iter_mut() {
            for ev in d.fetch_events()? {
                if let InputEventKind::Key(Key::KEY_LEFTALT | Key::KEY_RIGHTALT) = ev.kind() {
                    let _ = tx.send(KeyEvent {
                        alt: true,
                        pressed: ev.value() != 0,
                        timestamp_ms: ev
                            .timestamp()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    });
                }
            }
        }
    }
}
impl HotkeyBackend for GlobalHotkey {
    fn next_activation(&mut self) -> Result<Activation> {
        loop {
            let e = self.rx.recv()?;
            if let Some(a) = self.detector.feed(e) {
                return Ok(a);
            }
        }
    }
}
