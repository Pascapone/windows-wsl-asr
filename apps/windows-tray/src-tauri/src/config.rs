use std::{fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub start_on_login: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfig {
    pub wsl_distro: String,
    pub host: String,
    pub port: u16,
    pub model_name: String,
    pub gpu_memory_utilization: f32,
    pub chunk_size_sec: f32,
    pub auto_start_backend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureConfig {
    pub hotkey: String,
    pub input_device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioProcessingConfig {
    pub high_pass_enabled: bool,
    pub auto_gain_enabled: bool,
    pub compressor_enabled: bool,
    pub output_gain_enabled: bool,
    pub limiter_enabled: bool,
    pub metering_enabled: bool,
    pub high_pass_cutoff_hz: f32,
    pub target_rms_db: f32,
    pub auto_gain_min_db: f32,
    pub auto_gain_max_db: f32,
    pub auto_gain_attack_ms: f32,
    pub auto_gain_release_ms: f32,
    pub compressor_threshold_db: f32,
    pub compressor_ratio: f32,
    pub compressor_attack_ms: f32,
    pub compressor_release_ms: f32,
    pub output_gain_db: f32,
    pub limiter_ceiling_db: f32,
}

impl Default for AudioProcessingConfig {
    fn default() -> Self {
        Self {
            high_pass_enabled: true,
            auto_gain_enabled: true,
            compressor_enabled: true,
            output_gain_enabled: true,
            limiter_enabled: true,
            metering_enabled: true,
            high_pass_cutoff_hz: 80.0,
            target_rms_db: -20.0,
            auto_gain_min_db: -6.0,
            auto_gain_max_db: 18.0,
            auto_gain_attack_ms: 80.0,
            auto_gain_release_ms: 900.0,
            compressor_threshold_db: -18.0,
            compressor_ratio: 3.0,
            compressor_attack_ms: 8.0,
            compressor_release_ms: 180.0,
            output_gain_db: 0.0,
            limiter_ceiling_db: -1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayConfig {
    pub enabled: bool,
    pub anchor: String,
    pub offset_x: i32,
    pub offset_y: i32,
    pub max_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationConfig {
    pub language_hint: Option<String>,
    pub auto_paste: bool,
    pub restore_clipboard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryConfig {
    pub terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub backend: BackendConfig,
    pub capture: CaptureConfig,
    #[serde(default)]
    pub audio_processing: AudioProcessingConfig,
    pub overlay: OverlayConfig,
    pub dictation: DictationConfig,
    pub dictionary: DictionaryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                start_on_login: false,
            },
            backend: BackendConfig {
                wsl_distro: "Ubuntu".into(),
                host: "127.0.0.1".into(),
                port: 8765,
                model_name: "Qwen/Qwen3-ASR-1.7B".into(),
                gpu_memory_utilization: 0.85,
                chunk_size_sec: 0.5,
                auto_start_backend: true,
            },
            capture: CaptureConfig {
                hotkey: "Ctrl+Shift+Space".into(),
                input_device_id: None,
            },
            audio_processing: AudioProcessingConfig::default(),
            overlay: OverlayConfig {
                enabled: true,
                anchor: "center".into(),
                offset_x: 0,
                offset_y: 0,
                max_width: 420,
            },
            dictation: DictationConfig {
                language_hint: Some("German".into()),
                auto_paste: true,
                restore_clipboard: true,
            },
            dictionary: DictionaryConfig {
                terms: vec!["Pibo".into(), "OpenClaw".into(), "Pascal".into()],
            },
        }
    }
}

impl AppConfig {
    pub fn config_path() -> anyhow::Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("Roaming AppData directory not available")?
            .join("PiboLocalAsrTray");
        Ok(dir.join("config.json"))
    }

    pub fn log_path() -> anyhow::Result<PathBuf> {
        let dir = dirs::data_local_dir()
            .context("LocalAppData directory not available")?
            .join("PiboLocalAsrTray")
            .join("logs");
        Ok(dir.join("app.log"))
    }

    pub fn load_or_create() -> anyhow::Result<(Self, PathBuf)> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let mut config = serde_json::from_str::<Self>(&content)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            let mut changed = false;
            if (config.backend.gpu_memory_utilization - 0.82).abs() < f32::EPSILON {
                config.backend.gpu_memory_utilization = 0.85;
                changed = true;
            }
            if config.dictation.language_hint.as_deref() == Some("de") {
                config.dictation.language_hint = Some("German".into());
                changed = true;
            }
            if config.overlay.anchor == "mouse" {
                config.overlay.anchor = "center".into();
                config.overlay.offset_x = 0;
                config.overlay.offset_y = 0;
                changed = true;
            }
            if config.overlay.max_width == 520 {
                config.overlay.max_width = 420;
                changed = true;
            }
            if changed {
                config.save_to(&path)?;
            }
            Ok((config, path))
        } else {
            let config = Self::default();
            config.save_to(&path)?;
            Ok((config, path))
        }
    }

    pub fn save_to(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn dictionary_context(&self) -> String {
        let mut terms: Vec<String> = self
            .dictionary
            .terms
            .iter()
            .map(|term| term.trim())
            .filter(|term| !term.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        terms.sort();
        terms.dedup();
        terms.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_config_json() -> &'static str {
        r#"{
          "general": { "startOnLogin": false },
          "backend": {
            "wslDistro": "Ubuntu",
            "host": "127.0.0.1",
            "port": 8765,
            "modelName": "Qwen/Qwen3-ASR-1.7B",
            "gpuMemoryUtilization": 0.85,
            "chunkSizeSec": 0.5,
            "autoStartBackend": true
          },
          "capture": {
            "hotkey": "Ctrl+Shift+Space",
            "inputDeviceId": null
          },
          "overlay": {
            "enabled": true,
            "anchor": "center",
            "offsetX": 0,
            "offsetY": 0,
            "maxWidth": 420
          },
          "dictation": {
            "languageHint": "German",
            "autoPaste": true,
            "restoreClipboard": true
          },
          "dictionary": { "terms": ["Pibo"] }
        }"#
    }

    #[test]
    fn legacy_config_without_audio_processing_uses_defaults() {
        let config = serde_json::from_str::<AppConfig>(legacy_config_json())
            .expect("legacy config should parse");

        assert!(config.audio_processing.high_pass_enabled);
        assert!(config.audio_processing.auto_gain_enabled);
        assert!(config.audio_processing.compressor_enabled);
        assert!(config.audio_processing.output_gain_enabled);
        assert!(config.audio_processing.limiter_enabled);
        assert!(config.audio_processing.metering_enabled);
        assert_eq!(config.audio_processing.high_pass_cutoff_hz, 80.0);
        assert_eq!(config.audio_processing.target_rms_db, -20.0);
        assert_eq!(config.audio_processing.limiter_ceiling_db, -1.0);
    }

    #[test]
    fn save_and_reload_preserves_audio_processing_fields() {
        let mut config = serde_json::from_str::<AppConfig>(legacy_config_json())
            .expect("legacy config should parse");
        config.audio_processing.high_pass_enabled = false;
        config.audio_processing.target_rms_db = -24.5;
        config.audio_processing.output_gain_db = 2.0;

        let path = std::env::temp_dir().join(format!(
            "pibo-audio-processing-config-{}.json",
            std::process::id()
        ));
        config.save_to(&path).expect("config should save");
        let content = fs::read_to_string(&path).expect("saved config should be readable");
        let reloaded =
            serde_json::from_str::<AppConfig>(&content).expect("saved config should parse");
        let _ = fs::remove_file(path);

        assert!(!reloaded.audio_processing.high_pass_enabled);
        assert_eq!(reloaded.audio_processing.target_rms_db, -24.5);
        assert_eq!(reloaded.audio_processing.output_gain_db, 2.0);
    }
}
