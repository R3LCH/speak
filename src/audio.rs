use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
    samples: Arc<Mutex<Vec<f32>>>,
    channels: usize,
    sample_rate: u32,
    stream: Option<cpal::Stream>,
}
pub struct AudioCapture;
impl AudioCapture {
    pub fn start(name: &str) -> Result<RecordingHandle> {
        let host = cpal::default_host();
        let device = if name == "default" {
            host.default_input_device()
        } else {
            host.input_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
        }
        .ok_or_else(|| anyhow!("input device '{}' not found", name))?;
        let supported = device
            .default_input_config()
            .context("query default input config")?;
        let channels = supported.channels() as usize;
        let sample_rate = supported.sample_rate().0;
        let samples = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&samples);
        let err = |e| tracing::error!(error=%e, "audio stream error");
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &supported.config(),
                move |d: &[f32], _| sink.lock().unwrap().extend_from_slice(d),
                err,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &supported.config(),
                move |d: &[i16], _| {
                    let mut s = sink.lock().unwrap();
                    s.extend(d.iter().map(|v| *v as f32 / 32768.0));
                },
                err,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &supported.config(),
                move |d: &[u16], _| {
                    let mut s = sink.lock().unwrap();
                    s.extend(d.iter().map(|v| *v as f32 / 32768.0 - 1.0));
                },
                err,
                None,
            )?,
            f => return Err(anyhow!("unsupported sample format: {f:?}")),
        };
        stream.play().context("start audio stream")?;
        Ok(RecordingHandle {
            samples,
            channels,
            sample_rate,
            stream: Some(stream),
        })
    }
}
impl RecordingHandle {
    pub fn stop(mut self) -> Result<TempAudio> {
        self.stream.take();
        let raw = std::mem::take(&mut *self.samples.lock().unwrap());
        let dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("speak");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!(
            "recording-{}-{}.wav",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        let mut writer = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 16_000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )?;
        let ratio = self.sample_rate as f64 / 16_000.0;
        let frames = raw.len() / self.channels.max(1);
        for i in 0..((frames as f64 / ratio).ceil() as usize) {
            let src = ((i as f64 * ratio) as usize).min(frames.saturating_sub(1));
            let mut mono = 0.0;
            for c in 0..self.channels {
                mono += raw[src * self.channels + c];
            }
            writer.write_sample(
                (mono / self.channels.max(1) as f32 * 32767.0).clamp(-32768.0, 32767.0) as i16,
            )?;
        }
        writer.finalize()?;
        Ok(TempAudio(path))
    }
}
