use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::BackendConfig;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(3);
const START_TIMEOUT: Duration = Duration::from_secs(15);
const CHUNK_TIMEOUT: Duration = Duration::from_secs(45);
const FINISH_TIMEOUT: Duration = Duration::from_secs(45);
const CANCEL_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct BackendClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub model_loaded: bool,
    pub model_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartResponse {
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptResponse {
    pub language: Option<String>,
    pub text: String,
    pub chunk_index: Option<u32>,
    pub audio_seconds: Option<f32>,
    pub processing_ms: Option<f32>,
    pub text_length: Option<usize>,
}

#[derive(Debug, Serialize)]
struct SessionMeta {
    client: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
struct StartRequest {
    context: String,
    language: Option<String>,
    session_meta: SessionMeta,
}

impl BackendClient {
    pub fn new(config: &BackendConfig) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .build()
                .expect("failed to build reqwest client"),
            base_url: format!("http://{}:{}", config.host, config.port),
        }
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        self.client
            .get(format!("{}/healthz", self.base_url))
            .timeout(HEALTH_TIMEOUT)
            .send()
            .await
            .context("health request failed")?
            .error_for_status()
            .context("health request returned an error")?
            .json()
            .await
            .context("failed to decode health response")
    }

    pub async fn start_session(&self, context: String, language: Option<String>) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/api/start", self.base_url))
            .json(&StartRequest {
                context,
                language,
                session_meta: SessionMeta {
                    client: "windows-tray",
                    version: "0.1.0",
                },
            })
            .timeout(START_TIMEOUT)
            .send()
            .await
            .context("start request failed")?
            .error_for_status()
            .context("start request returned an error")?
            .json::<StartResponse>()
            .await
            .context("failed to decode start response")?;

        Ok(response.session_id)
    }

    pub async fn push_chunk(&self, session_id: &str, chunk: &[f32]) -> Result<TranscriptResponse> {
        let bytes = chunk
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<_>>();
        self.client
            .post(format!("{}/api/chunk", self.base_url))
            .query(&[("session_id", session_id)])
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .timeout(CHUNK_TIMEOUT)
            .send()
            .await
            .context("chunk request failed")?
            .error_for_status()
            .context("chunk request returned an error")?
            .json()
            .await
            .context("failed to decode chunk response")
    }

    pub async fn finish_session(&self, session_id: &str) -> Result<TranscriptResponse> {
        self.client
            .post(format!("{}/api/finish", self.base_url))
            .query(&[("session_id", session_id)])
            .timeout(FINISH_TIMEOUT)
            .send()
            .await
            .context("finish request failed")?
            .error_for_status()
            .context("finish request returned an error")?
            .json()
            .await
            .context("failed to decode finish response")
    }

    pub async fn cancel_session(&self, session_id: &str) -> Result<()> {
        self.client
            .post(format!("{}/api/cancel", self.base_url))
            .query(&[("session_id", session_id)])
            .timeout(CANCEL_TIMEOUT)
            .send()
            .await
            .context("cancel request failed")?
            .error_for_status()
            .context("cancel request returned an error")?;
        Ok(())
    }
}
