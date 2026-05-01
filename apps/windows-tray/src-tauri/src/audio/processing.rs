use serde::Serialize;

use crate::config::AudioProcessingConfig;

const SAMPLE_RATE_HZ: f32 = 16_000.0;
const MIN_DB: f32 = -120.0;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioProcessingMetrics {
    pub input_rms_db: f32,
    pub output_rms_db: f32,
    pub input_peak_db: f32,
    pub output_peak_db: f32,
    pub current_gain_db: f32,
    pub gain_reduction_db: f32,
    pub clip_count: u64,
    pub limit_count: u64,
    pub dropped_chunks: u64,
}

#[derive(Debug, Clone)]
pub struct AudioProcessor {
    config: AudioProcessingConfig,
    high_pass_prev_x: f32,
    high_pass_prev_y: f32,
    auto_gain_db: f32,
    compressor_gain_db: f32,
    total_clip_count: u64,
    total_limit_count: u64,
}

impl AudioProcessor {
    pub fn new(config: AudioProcessingConfig) -> Self {
        Self {
            config,
            high_pass_prev_x: 0.0,
            high_pass_prev_y: 0.0,
            auto_gain_db: 0.0,
            compressor_gain_db: 0.0,
            total_clip_count: 0,
            total_limit_count: 0,
        }
    }

    pub fn process(
        &mut self,
        samples: &mut [f32],
        dropped_chunks: u64,
    ) -> Option<AudioProcessingMetrics> {
        let input_stats = self.config.metering_enabled.then(|| signal_stats(samples));
        let clipped_this_chunk = if self.config.metering_enabled {
            samples.iter().filter(|sample| sample.abs() >= 1.0).count() as u64
        } else {
            0
        };
        self.total_clip_count = self.total_clip_count.saturating_add(clipped_this_chunk);

        if self.config.high_pass_enabled {
            self.apply_high_pass(samples);
        }

        if self.config.auto_gain_enabled {
            self.apply_auto_gain(samples);
        }

        if self.config.compressor_enabled {
            self.apply_compressor(samples);
        }

        if self.config.output_gain_enabled {
            let gain = db_to_linear(self.config.output_gain_db);
            for sample in samples.iter_mut() {
                *sample *= gain;
            }
        }

        let limited_this_chunk = if self.config.limiter_enabled {
            self.apply_limiter(samples)
        } else {
            0
        };
        self.total_limit_count = self.total_limit_count.saturating_add(limited_this_chunk);

        if !self.config.metering_enabled {
            return None;
        }

        let (input_rms_db, input_peak_db) = input_stats.unwrap_or((MIN_DB, MIN_DB));
        let (output_rms_db, output_peak_db) = signal_stats(samples);
        Some(AudioProcessingMetrics {
            input_rms_db,
            output_rms_db,
            input_peak_db,
            output_peak_db,
            current_gain_db: self.auto_gain_db,
            gain_reduction_db: -self.compressor_gain_db.min(0.0),
            clip_count: self.total_clip_count,
            limit_count: self.total_limit_count,
            dropped_chunks,
        })
    }

    fn apply_high_pass(&mut self, samples: &mut [f32]) {
        let cutoff = self
            .config
            .high_pass_cutoff_hz
            .clamp(10.0, SAMPLE_RATE_HZ * 0.45);
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / SAMPLE_RATE_HZ;
        let alpha = rc / (rc + dt);

        for sample in samples.iter_mut() {
            let input = *sample;
            let output = alpha * (self.high_pass_prev_y + input - self.high_pass_prev_x);
            self.high_pass_prev_x = input;
            self.high_pass_prev_y = output;
            *sample = output;
        }
    }

    fn apply_auto_gain(&mut self, samples: &mut [f32]) {
        let (rms_db, _) = signal_stats(samples);
        let target_gain_db = (self.config.target_rms_db - rms_db)
            .clamp(self.config.auto_gain_min_db, self.config.auto_gain_max_db);
        let attack = smoothing_coeff(self.config.auto_gain_attack_ms);
        let release = smoothing_coeff(self.config.auto_gain_release_ms);

        for sample in samples.iter_mut() {
            let coeff = if target_gain_db < self.auto_gain_db {
                attack
            } else {
                release
            };
            self.auto_gain_db += (target_gain_db - self.auto_gain_db) * coeff;
            *sample *= db_to_linear(self.auto_gain_db);
        }
    }

    fn apply_compressor(&mut self, samples: &mut [f32]) {
        let threshold = self.config.compressor_threshold_db;
        let ratio = self.config.compressor_ratio.max(1.0);
        let attack = smoothing_coeff(self.config.compressor_attack_ms);
        let release = smoothing_coeff(self.config.compressor_release_ms);

        for sample in samples.iter_mut() {
            let level_db = amplitude_to_db(sample.abs());
            let target_gain_db = if level_db > threshold {
                let compressed_db = threshold + (level_db - threshold) / ratio;
                compressed_db - level_db
            } else {
                0.0
            };
            let coeff = if target_gain_db < self.compressor_gain_db {
                attack
            } else {
                release
            };
            self.compressor_gain_db += (target_gain_db - self.compressor_gain_db) * coeff;
            *sample *= db_to_linear(self.compressor_gain_db);
        }
    }

    fn apply_limiter(&mut self, samples: &mut [f32]) -> u64 {
        let ceiling = db_to_linear(self.config.limiter_ceiling_db.min(0.0));
        let mut limited = 0u64;
        for sample in samples.iter_mut() {
            if sample.abs() > ceiling {
                *sample = sample.signum() * ceiling;
                limited += 1;
            }
        }
        limited
    }
}

fn signal_stats(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (MIN_DB, MIN_DB);
    }

    let mut sum_squares = 0.0f64;
    let mut peak = 0.0f32;
    for sample in samples {
        let value = sample.abs();
        sum_squares += f64::from(*sample) * f64::from(*sample);
        peak = peak.max(value);
    }

    let rms = (sum_squares / samples.len() as f64).sqrt() as f32;
    (amplitude_to_db(rms), amplitude_to_db(peak))
}

fn amplitude_to_db(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        MIN_DB
    } else {
        (20.0 * amplitude.log10()).max(MIN_DB)
    }
}

fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

fn smoothing_coeff(time_ms: f32) -> f32 {
    if time_ms <= 0.0 {
        1.0
    } else {
        1.0 - (-1.0 / (SAMPLE_RATE_HZ * time_ms / 1000.0)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_config() -> AudioProcessingConfig {
        AudioProcessingConfig {
            high_pass_enabled: false,
            auto_gain_enabled: false,
            compressor_enabled: false,
            output_gain_enabled: false,
            limiter_enabled: false,
            metering_enabled: false,
            ..AudioProcessingConfig::default()
        }
    }

    #[test]
    fn disabled_pipeline_is_pass_through() {
        let mut processor = AudioProcessor::new(disabled_config());
        let mut samples = vec![0.0, 0.1, -0.2, 0.4, -0.7, 0.25];
        let original = samples.clone();

        let metrics = processor.process(&mut samples, 0);

        assert!(metrics.is_none());
        assert_eq!(samples, original);
    }

    #[test]
    fn high_pass_reduces_dc_component() {
        let mut config = disabled_config();
        config.high_pass_enabled = true;
        config.high_pass_cutoff_hz = 80.0;
        let mut processor = AudioProcessor::new(config);
        let mut samples = vec![0.5; 16_000];

        processor.process(&mut samples, 0);

        let average = samples.iter().copied().sum::<f32>() / samples.len() as f32;
        assert!(average.abs() < 0.02, "average was {average}");
    }

    #[test]
    fn auto_gain_raises_quiet_signal_and_respects_max() {
        let mut config = disabled_config();
        config.auto_gain_enabled = true;
        config.auto_gain_attack_ms = 0.0;
        config.auto_gain_release_ms = 0.0;
        config.target_rms_db = -20.0;
        config.auto_gain_min_db = -6.0;
        config.auto_gain_max_db = 12.0;
        let mut processor = AudioProcessor::new(config);
        let mut samples = vec![0.01; 3200];

        processor.process(&mut samples, 0);

        let gain = samples[0] / 0.01;
        assert!(gain > 3.9, "gain was {gain}");
        assert!(gain <= db_to_linear(12.0) + 0.001, "gain was {gain}");
    }

    #[test]
    fn compressor_reduces_signal_above_threshold() {
        let mut config = disabled_config();
        config.compressor_enabled = true;
        config.compressor_attack_ms = 0.0;
        config.compressor_release_ms = 0.0;
        config.compressor_threshold_db = -18.0;
        config.compressor_ratio = 3.0;
        let mut processor = AudioProcessor::new(config);
        let mut samples = vec![0.5; 3200];

        processor.process(&mut samples, 0);

        assert!(samples[0] < 0.5, "sample was {}", samples[0]);
    }

    #[test]
    fn limiter_never_exceeds_ceiling() {
        let mut config = disabled_config();
        config.limiter_enabled = true;
        config.limiter_ceiling_db = -1.0;
        let mut processor = AudioProcessor::new(config);
        let mut samples = vec![-1.5, -1.0, 0.0, 1.0, 1.5];

        processor.process(&mut samples, 0);

        let ceiling = db_to_linear(-1.0);
        assert!(samples
            .iter()
            .all(|sample| sample.abs() <= ceiling + 0.000_001));
    }
}
