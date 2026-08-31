use crate::audio::AudioCapture;
use crate::config::Config;
use crate::paste::{normalize_transcription, CommandPaste};
use crate::worker::{TranscriptionRequest, WorkerClient};
use anyhow::Result;
pub struct Daemon;
impl Daemon {
    pub fn run(config: Config) -> Result<()> {
        config.validate()?;
        let mut worker = WorkerClient::start(&config)?;
        let _capture = AudioCapture::start(&config.audio_device)?;
        let _paste = CommandPaste::detect();
        tracing::info!(model=%config.model,"speak daemon ready");
        let _ = (
            &mut worker,
            normalize_transcription(""),
            TranscriptionRequest {
                id: String::new(),
                audio_path: String::new(),
                model: config.model,
                device: "auto".into(),
                compute_type: "auto".into(),
                language: None,
                beam_size: config.beam_size,
            },
        );
        Ok(())
    }
}
