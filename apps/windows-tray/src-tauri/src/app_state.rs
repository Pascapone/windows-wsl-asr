use std::{path::PathBuf, sync::Arc};

use serde::Serialize;
use tokio::sync::Mutex;

use crate::config::AppConfig;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendStatus {
    Stopped,
    Starting,
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DictationStatus {
    Idle,
    BackendStarting,
    BackendReady,
    Recording,
    Finalizing,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub config: AppConfig,
    pub backend_status: BackendStatus,
    pub dictation_status: DictationStatus,
    pub backend_owned: bool,
    pub backend_model_loaded: bool,
    pub partial_text: String,
    pub last_transcript: Option<String>,
    pub error_message: Option<String>,
}

impl AppSnapshot {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            backend_status: BackendStatus::Stopped,
            dictation_status: DictationStatus::Idle,
            backend_owned: false,
            backend_model_loaded: false,
            partial_text: String::new(),
            last_transcript: None,
            error_message: None,
        }
    }
}

pub struct RuntimeState {
    pub snapshot: AppSnapshot,
}

#[derive(Clone)]
pub struct StateStore {
    inner: Arc<Mutex<RuntimeState>>,
    config_path: PathBuf,
    log_path: PathBuf,
}

impl StateStore {
    pub fn new(snapshot: AppSnapshot, config_path: PathBuf, log_path: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeState { snapshot })),
            config_path,
            log_path,
        }
    }

    pub async fn snapshot(&self) -> AppSnapshot {
        self.inner.lock().await.snapshot.clone()
    }

    pub async fn update<F>(&self, update: F) -> AppSnapshot
    where
        F: FnOnce(&mut AppSnapshot),
    {
        let mut guard = self.inner.lock().await;
        update(&mut guard.snapshot);
        guard.snapshot.clone()
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}
