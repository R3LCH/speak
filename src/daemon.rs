use crate::audio::AudioCapture;
use crate::config::Config;
use crate::hotkey::{GlobalHotkey, HotkeyBackend};
use crate::paste::PasteBackend;
use crate::paste::{normalize_transcription, CommandPaste};
use crate::worker::{TranscriptionRequest, WorkerClient};
use anyhow::{Context, Result};
pub struct Daemon;
impl Daemon {
    pub fn run(config: Config) -> Result<()> {
        config.validate()?;
        ctrlc::set_handler(|| std::process::exit(0)).context("install signal handler")?;
        let mut worker = WorkerClient::start(&config)?;
        let mut hotkey = GlobalHotkey::start(config.double_tap_ms);
        let mut paste = CommandPaste::detect();
        tracing::info!(model=%config.model,"speak daemon ready");
        let mut recording = None;
        loop {
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
