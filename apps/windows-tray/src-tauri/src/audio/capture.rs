use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, StreamConfig,
};
use serde::Serialize;
use tokio::sync::mpsc::{self, error::TrySendError};

use super::resample::LinearResampler;

const TARGET_RATE: u32 = 16_000;
const TARGET_CHUNK_SAMPLES: usize = 3_200;
const AUDIO_QUEUE_CAPACITY: usize = 16;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub struct AudioCaptureHandle {
    _stream: Stream,
}

impl AudioCaptureHandle {
    pub fn stop(self) {}
}

struct ChunkPipeline {
    channels: usize,
    resampler: LinearResampler,
    pending: Vec<f32>,
    sender: mpsc::Sender<Vec<f32>>,
    dropped_chunks: usize,
}

impl ChunkPipeline {
    fn new(sample_rate: u32, channels: usize, sender: mpsc::Sender<Vec<f32>>) -> Self {
        Self {
            channels,
            resampler: LinearResampler::new(sample_rate, TARGET_RATE),
            pending: Vec::new(),
            sender,
            dropped_chunks: 0,
        }
    }

    fn push_f32(&mut self, data: &[f32]) {
        let mono = downmix_to_mono(data, self.channels, |sample| sample);
        self.flush_mono(&mono);
    }

    fn push_i16(&mut self, data: &[i16]) {
        let mono = downmix_to_mono(data, self.channels, |sample| sample as f32 / 32768.0);
        self.flush_mono(&mono);
    }

    fn push_u16(&mut self, data: &[u16]) {
        let mono = downmix_to_mono(data, self.channels, |sample| (sample as f32 / 65535.0) * 2.0 - 1.0);
        self.flush_mono(&mono);
    }

    fn flush_mono(&mut self, mono: &[f32]) {
        let resampled = self.resampler.push(mono);
        if resampled.is_empty() {
            return;
        }

        self.pending.extend(resampled);
        while self.pending.len() >= TARGET_CHUNK_SAMPLES {
            let chunk = self.pending.drain(..TARGET_CHUNK_SAMPLES).collect::<Vec<_>>();
            match self.sender.try_send(chunk) {
                Ok(()) => {}
                Err(TrySendError::Full(_chunk)) => {
                    self.dropped_chunks += 1;
                    if self.dropped_chunks == 1 || self.dropped_chunks % 10 == 0 {
                        log::warn!(
                            "Audio queue full; dropped {} chunk(s) while backend was slower than capture",
                            self.dropped_chunks
                        );
                    }
                }
                Err(TrySendError::Closed(_chunk)) => {
                    return;
                }
            }
        }
    }
}

fn downmix_to_mono<T>(data: &[T], channels: usize, convert: impl Fn(T) -> f32 + Copy) -> Vec<f32>
where
    T: Copy,
{
    if channels == 0 {
        return Vec::new();
    }

    let mut mono = Vec::with_capacity(data.len() / channels.max(1));
    for frame in data.chunks(channels) {
        let sum = frame.iter().copied().map(convert).sum::<f32>();
        mono.push(sum / channels as f32);
    }
    mono
}

fn host() -> cpal::Host {
    cpal::default_host()
}

fn enumerate_devices() -> anyhow::Result<Vec<(String, Device, bool)>> {
    let host = host();
    let default_name = host.default_input_device().and_then(|device| device.name().ok());
    let mut items = Vec::new();
    for (index, device) in host.input_devices()?.enumerate() {
        let name = device.name().unwrap_or_else(|_| format!("Input {}", index + 1));
        let id = format!("{index}:{name}");
        let is_default = default_name.as_ref().map(|value| value == &name).unwrap_or(false);
        items.push((id, device, is_default));
    }
    Ok(items)
}

pub fn list_input_devices() -> anyhow::Result<Vec<AudioDeviceInfo>> {
    Ok(enumerate_devices()?
        .into_iter()
        .map(|(id, device, is_default)| AudioDeviceInfo {
            id,
            name: device.name().unwrap_or_else(|_| "Unknown input".into()),
            is_default,
        })
        .collect())
}

fn resolve_device(device_id: Option<&str>) -> anyhow::Result<Device> {
    let host = host();
    if let Some(device_id) = device_id {
        let devices = enumerate_devices()?;
        if let Some((_, device, _)) = devices.into_iter().find(|(id, _, _)| id == device_id) {
            return Ok(device);
        }
    }

    host.default_input_device()
        .ok_or_else(|| anyhow!("No input device available"))
}

pub fn start_capture(
    device_id: Option<&str>,
) -> anyhow::Result<(AudioCaptureHandle, mpsc::Receiver<Vec<f32>>)> {
    let device = resolve_device(device_id)?;
    let supported = device.default_input_config().context("failed to query input config")?;
    let config: StreamConfig = supported.clone().into();
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate.0;
    let (sender, receiver) = mpsc::channel(AUDIO_QUEUE_CAPACITY);
    let pipeline = Arc::new(Mutex::new(ChunkPipeline::new(sample_rate, channels, sender)));
    let error_callback = |error| {
        log::error!("Audio capture error: {error}");
    };

    let stream = match supported.sample_format() {
        SampleFormat::F32 => {
            let pipeline = Arc::clone(&pipeline);
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    if let Ok(mut pipeline) = pipeline.lock() {
                        pipeline.push_f32(data);
                    }
                },
                error_callback,
                None,
            )?
        }
        SampleFormat::I16 => {
            let pipeline = Arc::clone(&pipeline);
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    if let Ok(mut pipeline) = pipeline.lock() {
                        pipeline.push_i16(data);
                    }
                },
                error_callback,
                None,
            )?
        }
        SampleFormat::U16 => {
            let pipeline = Arc::clone(&pipeline);
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    if let Ok(mut pipeline) = pipeline.lock() {
                        pipeline.push_u16(data);
                    }
                },
                error_callback,
                None,
            )?
        }
        other => return Err(anyhow!("Unsupported audio sample format: {other:?}")),
    };

    stream.play().context("failed to start input stream")?;
    Ok((AudioCaptureHandle { _stream: stream }, receiver))
}
