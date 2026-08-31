use crate::audio::AudioCapture;
use crate::config::Config;
use crate::hotkey::{GlobalHotkey, HotkeyBackend};
use crate::paste::PasteBackend;
use crate::paste::{normalize_transcription, CommandPaste};
use crate::worker::{TranscriptionRequest, WorkerClient};
use anyhow::Result;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
pub struct Daemon;
impl Daemon {
    pub fn run(config: Config) -> Result<()> {
        config.validate()?;
        let stop = Arc::new(AtomicBool::new(false));
        let ss = Arc::clone(&stop);
        std::thread::spawn(move || {
            let _ = crate::control::serve(ss);
        });
        let mut worker = WorkerClient::start(&config)?;
        let mut hotkey = GlobalHotkey::start(config.double_tap_ms);
        let mut paste = CommandPaste::detect();
        tracing::info!(model=%config.model,"speak daemon ready");
        let mut recording = None;
        loop {
            if stop.load(Ordering::Relaxed) {
                break Ok(());
            }
            hotkey.next_activation()?;
            if recording.is_none() {
                recording = Some(AudioCapture::start(&config.audio_device)?);
                tracing::info!("recording started");
            } else {
                let audio = recording.take().unwrap().stop()?;
                tracing::info!("recording stopped; transcribing");
                let response = worker.transcribe(TranscriptionRequest {
                    id: format!("{}", std::process::id()),
                    audio_path: audio.path().display().to_string(),
                    model: config.model.clone(),
                    device: "auto".into(),
                    compute_type: "auto".into(),
                    language: (config.language != "auto").then(|| config.language.clone()),
                    beam_size: config.beam_size,
                })?;
                tracing::info!(ok=response.ok, device=%response.device_used, text=?response.text, "transcription response");
                if response.ok {
                    if let Some(text) = response.text.and_then(|t| normalize_transcription(&t)) {
                        paste.paste(&text)?;
                        tracing::info!("text pasted");
                    }
                }
            }
        }
    }
}
