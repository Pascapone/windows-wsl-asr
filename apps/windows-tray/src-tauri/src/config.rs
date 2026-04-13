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
            fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
        }

        if path.exists() {
            let content = fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
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
            fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
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
