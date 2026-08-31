use anyhow::Result;
use std::path::{Path, PathBuf};
pub struct TempAudio(PathBuf);
impl TempAudio {
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempAudio {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
pub struct RecordingHandle {
    samples: Vec<i16>,
    sample_rate: u32,
}
pub struct AudioCapture;
impl AudioCapture {
    pub fn start(_: &str) -> Result<RecordingHandle> {
        Ok(RecordingHandle {
            samples: Vec::new(),
            sample_rate: 16000,
        })
    }
}
impl RecordingHandle {
    pub fn push_samples(&mut self, s: &[i16]) {
        self.samples.extend_from_slice(s)
    }
    pub fn stop(self) -> Result<TempAudio> {
        let dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("speak");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("recording-{}.wav", std::process::id()));
        let mut w = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: self.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )?;
        for s in self.samples {
            w.write_sample(s)?;
        }
        w.finalize()?;
        Ok(TempAudio(path))
    }
}
