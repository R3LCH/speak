use crate::config::Config;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
pub fn socket_path() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("speak.sock")
}
pub fn serve(stop: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Result<()> {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0; 64];
                let n = stream.read(&mut buf).unwrap_or(0);
                let cmd = String::from_utf8_lossy(&buf[..n]);
                let reply = if cmd.trim() == "stop" {
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                    "stopping\n"
                } else if cmd.trim() == "status" {
                    "running\n"
                } else {
                    "error: unknown command\n"
                };
                let _ = stream.write_all(reply.as_bytes());
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100))
            }
            Err(e) => return Err(e.into()),
        }
    }
    let _ = std::fs::remove_file(path);
    Ok(())
}
pub fn client_command(command: &str) -> Result<()> {
    let mut stream =
        UnixStream::connect(socket_path()).context("connect to speak daemon; is it running?")?;
    stream.write_all(command.as_bytes())?;
    let mut out = String::new();
    stream.read_to_string(&mut out)?;
    print!("{out}");
    Ok(())
}
pub fn doctor(_: &Config) -> Result<()> {
    let s = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        "wayland"
    } else if std::env::var("DISPLAY").is_ok() {
        "x11"
    } else {
        "none"
    };
    println!("session: {s}");
    if s == "none" {
        anyhow::bail!("no display session")
    }
    let input_access = std::fs::read_dir("/dev/input")
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .any(|e| std::fs::File::open(e.path()).is_ok())
        })
        .unwrap_or(false);
    println!(
        "/dev/input readable: {}",
        if input_access {
            "yes"
        } else {
            "no (add user to input group or udev rule)"
        }
    );
    for cmd in if s == "wayland" {
        vec!["wl-copy", "ydotool"]
    } else {
        vec!["xclip", "xdotool"]
    } {
        let ok = std::process::Command::new("sh")
            .args(["-c", &format!("command -v {cmd}")])
            .status()
            .map(|v| v.success())
            .unwrap_or(false);
        println!("{cmd}: {}", if ok { "ok" } else { "missing" });
    }
    if s == "wayland" {
        println!(
            "/dev/uinput: {}",
            if std::path::Path::new("/dev/uinput").exists() {
                "present"
            } else {
                "missing"
            }
        );
    }
    let python = Config::load(Config::default_path())
        .ok()
        .and_then(|c| {
            c.worker_command
                .split_whitespace()
                .next()
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "python3".into());
    let py = std::process::Command::new(&python)
        .args(["-c", "import faster_whisper"])
        .status()
        .map(|v| v.success())
        .unwrap_or(false);
    println!(
        "faster-whisper: {}",
        if py {
            "ok"
        } else {
            "missing (install worker/requirements.txt)"
        }
    );
    Ok(())
}
