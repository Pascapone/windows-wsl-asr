use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    app_state::{BackendStatus, DictationStatus, StateStore},
    audio::capture::{start_capture, AudioCaptureHandle},
    backend_manager::BackendManager,
    dictation::{
        backend_client::BackendClient,
        paste::{copy_text, paste_text},
    },
    emit_snapshot,
    overlay::window::{hide_overlay, show_overlay},
};

const MAX_SEGMENT_CHUNKS: usize = 48;
const MAX_SEGMENT_AUDIO_SECONDS: f32 = 24.0;
const MAX_CHUNK_PROCESSING_MS: f32 = 3_500.0;

struct LiveSessionState {
    session_id: String,
    committed_text: String,
    segment_index: usize,
}

struct ActiveSession {
    capture: AudioCaptureHandle,
    send_task: JoinHandle<anyhow::Result<()>>,
    live: Arc<Mutex<LiveSessionState>>,
}

#[derive(Clone, Default)]
pub struct SessionController {
    active: Arc<Mutex<Option<ActiveSession>>>,
}

impl SessionController {
    pub async fn is_recording(&self) -> bool {
        self.active.lock().await.is_some()
    }

    pub async fn start_recording(
        &self,
        app: &tauri::AppHandle,
        state: &StateStore,
        backend_manager: &BackendManager,
    ) -> anyhow::Result<()> {
        if self.active.lock().await.is_some() {
            return Ok(());
        }

        let snapshot = state.snapshot().await;
        let config = snapshot.config.clone();
        if snapshot.backend_status != BackendStatus::Ready {
            if !config.backend.auto_start_backend {
                return Err(anyhow!("Backend is not ready. Start it first or enable auto-start."));
            }

            state
                .update(|snapshot| {
                    snapshot.backend_status = BackendStatus::Starting;
                    snapshot.dictation_status = DictationStatus::BackendStarting;
                    snapshot.error_message = None;
                })
                .await;
            emit_snapshot(app, state).await?;
            if config.overlay.enabled {
                show_overlay(app, state).await?;
            }
            backend_manager.start(app, state).await?;
            backend_manager
                .wait_until_ready(app, state, Duration::from_secs(90))
                .await?;
        }

        let current = state.snapshot().await;
        let backend_client = BackendClient::new(&current.config.backend);
        let dictionary_context = current.config.dictionary_context();
        let language_hint = current.config.dictation.language_hint.clone();
        let session_id = backend_client
            .start_session(
                dictionary_context.clone(),
                language_hint.clone(),
            )
            .await
            .context("failed to start backend session")?;
        let (capture, mut receiver) = start_capture(current.config.capture.input_device_id.as_deref())?;
        let send_client = backend_client.clone();
        let send_state = state.clone();
        let send_app = app.clone();
        let live = Arc::new(Mutex::new(LiveSessionState {
            session_id: session_id.clone(),
            committed_text: String::new(),
            segment_index: 0,
        }));
        let send_live = Arc::clone(&live);
        let send_task = tokio::spawn(async move {
            let mut total_chunk_index = 0usize;
            let mut segment_chunk_index = 0usize;
            let mut segment_started_at = Instant::now();

            while let Some(chunk) = receiver.recv().await {
                total_chunk_index += 1;
                segment_chunk_index += 1;

                let (session_id, segment_index, committed_text) = {
                    let live = send_live.lock().await;
                    (
                        live.session_id.clone(),
                        live.segment_index,
                        live.committed_text.clone(),
                    )
                };

                let request_started_at = Instant::now();
                let response = match send_client.push_chunk(&session_id, &chunk).await {
                    Ok(response) => response,
                    Err(error) => {
                        let error = error.context(format!(
                            "failed to push chunk {total_chunk_index} for session {session_id}"
                        ));
                        send_state
                            .update(|snapshot| {
                                snapshot.dictation_status = DictationStatus::Error;
                                snapshot.error_message = Some(error.to_string());
                            })
                            .await;
                        let _ = emit_snapshot(&send_app, &send_state).await;
                        return Err(error);
                    }
                };
                let request_ms = request_started_at.elapsed().as_secs_f32() * 1000.0;
                let combined_partial = combine_transcript(&committed_text, &response.text);

                log::info!(
                    "chunk ok session={} segment={} total_chunk={} segment_chunk={} samples={} request_ms={:.1} backend_ms={:?} audio_seconds={:?} text_length={:?}",
                    session_id,
                    segment_index,
                    total_chunk_index,
                    segment_chunk_index,
                    chunk.len(),
                    request_ms,
                    response.processing_ms,
                    response.audio_seconds,
                    response.text_length,
                );

                send_state
                    .update(|snapshot| {
                        snapshot.partial_text = combined_partial.clone();
                        snapshot.dictation_status = DictationStatus::Recording;
                        snapshot.error_message = None;
                    })
                    .await;
                emit_snapshot(&send_app, &send_state).await?;

                if should_rollover(
                    segment_chunk_index,
                    response.audio_seconds,
                    response.processing_ms.unwrap_or(request_ms),
                    segment_started_at.elapsed(),
                ) {
                    log::info!(
                        "rolling over session={} segment={} total_chunk={} segment_chunk={} audio_seconds={:?} processing_ms={:?}",
                        session_id,
                        segment_index,
                        total_chunk_index,
                        segment_chunk_index,
                        response.audio_seconds,
                        response.processing_ms,
                    );

                    let finish_response = match send_client.finish_session(&session_id).await {
                        Ok(response) => response,
                        Err(error) => {
                            let error =
                                error.context(format!("failed to finalize rollover session {session_id}"));
                            send_state
                                .update(|snapshot| {
                                    snapshot.dictation_status = DictationStatus::Error;
                                    snapshot.error_message = Some(error.to_string());
                                })
                                .await;
                            let _ = emit_snapshot(&send_app, &send_state).await;
                            return Err(error);
                        }
                    };

                    let committed_after_rollover = {
                        let mut live = send_live.lock().await;
                        append_transcript(&mut live.committed_text, &finish_response.text);
                        live.segment_index += 1;
                        live.committed_text.clone()
                    };
                    send_state
                        .update(|snapshot| {
                            snapshot.partial_text = committed_after_rollover.clone();
                            snapshot.error_message = None;
                        })
                        .await;
                    emit_snapshot(&send_app, &send_state).await?;

                    let new_session_id =
                        match send_client.start_session(dictionary_context.clone(), language_hint.clone()).await {
                            Ok(session_id) => session_id,
                            Err(error) => {
                                let error = error.context("failed to start rollover backend session");
                                send_state
                                    .update(|snapshot| {
                                        snapshot.dictation_status = DictationStatus::Error;
                                        snapshot.error_message = Some(error.to_string());
                                    })
                                    .await;
                                let _ = emit_snapshot(&send_app, &send_state).await;
                                return Err(error);
                            }
                        };
                    {
                        let mut live = send_live.lock().await;
                        live.session_id = new_session_id.clone();
                    }
                    log::info!(
                        "rollover complete old_session={} new_session={} next_segment={}",
                        session_id,
                        new_session_id,
                        segment_index + 1,
                    );
                    segment_chunk_index = 0;
                    segment_started_at = Instant::now();
                }
            }
            Ok(())
        });

        *self.active.lock().await = Some(ActiveSession {
            capture,
            send_task,
            live,
        });

        state
            .update(|snapshot| {
                snapshot.partial_text.clear();
                snapshot.dictation_status = DictationStatus::Recording;
                snapshot.error_message = None;
            })
            .await;
        if current.config.overlay.enabled {
            show_overlay(app, state).await?;
        }
        emit_snapshot(app, state).await?;
        Ok(())
    }

    pub async fn stop_recording(&self, app: &tauri::AppHandle, state: &StateStore) -> anyhow::Result<()> {
        let active = self
            .active
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow!("No active recording"))?;

        state
            .update(|snapshot| {
                snapshot.dictation_status = DictationStatus::Finalizing;
            })
            .await;
        emit_snapshot(app, state).await?;

        active.capture.stop();
        for _ in 0..20 {
            if active.send_task.is_finished() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        if !active.send_task.is_finished() {
            log::warn!("audio send task did not finish promptly after stop; aborting remaining work");
            active.send_task.abort();
        }
        let mut send_task_error = None;
        match active.send_task.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                log::error!("audio send task failed before finalize: {error:#}");
                send_task_error = Some(error.to_string());
            }
            Err(error) if error.is_cancelled() => {}
            Err(error) => {
                log::error!("audio send task join failed before finalize: {error}");
                send_task_error = Some(error.to_string());
            }
        }

        let snapshot = state.snapshot().await;
        let client = BackendClient::new(&snapshot.config.backend);
        let (session_id, mut final_text) = {
            let live = active.live.lock().await;
            (live.session_id.clone(), live.committed_text.clone())
        };
        let finish_result = client
            .finish_session(&session_id)
            .await
            .with_context(|| format!("failed to finish backend session {session_id}"));

        let mut last_transcript = snapshot.last_transcript.clone();
        let mut error_message = send_task_error;
        match finish_result {
            Ok(response) => {
                append_transcript(&mut final_text, &response.text);
                if !final_text.trim().is_empty() {
                    copy_text(&final_text)?;
                    last_transcript = Some(final_text.clone());
                    if snapshot.config.dictation.auto_paste {
                        let outcome = paste_text(&final_text, snapshot.config.dictation.restore_clipboard)?;
                        log::info!("Paste completed, clipboard restored: {}", outcome.clipboard_restored);
                    }
                }
            }
            Err(error) => {
                if error_message.is_none() {
                    error_message = Some(error.to_string());
                } else {
                    log::error!("finish session also failed: {error:#}");
                }
            }
        }

        let preserved_text = preserve_visible_text(&snapshot.partial_text, &final_text);
        let has_error = error_message.is_some();

        state
            .update(|snapshot| {
                snapshot.last_transcript = last_transcript.clone();
                snapshot.partial_text = if has_error {
                    preserved_text.clone()
                } else {
                    String::new()
                };
                snapshot.dictation_status = if has_error {
                    DictationStatus::Error
                } else {
                    DictationStatus::Idle
                };
                snapshot.error_message = error_message.clone();
            })
            .await;
        if !has_error {
            hide_overlay(app)?;
        }
        emit_snapshot(app, state).await?;

        if let Some(error_message) = error_message {
            return Err(anyhow!(error_message));
        }

        Ok(())
    }

    pub async fn cancel_recording(&self, app: &tauri::AppHandle, state: &StateStore) -> anyhow::Result<()> {
        let active = self.active.lock().await.take();
        if let Some(active) = active {
            active.capture.stop();
            active.send_task.abort();

            let snapshot = state.snapshot().await;
            let client = BackendClient::new(&snapshot.config.backend);
            let session_id = { active.live.lock().await.session_id.clone() };
            let _ = client.cancel_session(&session_id).await;
        }

        state
            .update(|snapshot| {
                snapshot.partial_text.clear();
                snapshot.dictation_status = DictationStatus::Idle;
                snapshot.error_message = None;
            })
            .await;
        hide_overlay(app)?;
        emit_snapshot(app, state).await?;
        Ok(())
    }
}

fn should_rollover(
    segment_chunk_index: usize,
    audio_seconds: Option<f32>,
    processing_ms: f32,
    segment_elapsed: Duration,
) -> bool {
    if segment_chunk_index >= MAX_SEGMENT_CHUNKS {
        return true;
    }
    if audio_seconds.unwrap_or_default() >= MAX_SEGMENT_AUDIO_SECONDS {
        return true;
    }
    processing_ms >= MAX_CHUNK_PROCESSING_MS && segment_elapsed >= Duration::from_secs(6)
}

fn combine_transcript(committed_text: &str, partial_text: &str) -> String {
    let mut combined = committed_text.to_string();
    append_transcript(&mut combined, partial_text);
    combined
}

fn preserve_visible_text(current_partial: &str, finalized_text: &str) -> String {
    if !current_partial.trim().is_empty() {
        current_partial.trim().to_string()
    } else {
        finalized_text.trim().to_string()
    }
}

fn append_transcript(target: &mut String, addition: &str) {
    let addition = addition.trim();
    if addition.is_empty() {
        return;
    }

    if target.trim().is_empty() {
        target.clear();
        target.push_str(addition);
        return;
    }

    let needs_space = target
        .chars()
        .last()
        .map(|ch| !ch.is_whitespace() && !matches!(ch, '(' | '[' | '{' | '"' | '\''))
        .unwrap_or(false)
        && addition
            .chars()
            .next()
            .map(|ch| !ch.is_whitespace() && !matches!(ch, '.' | ',' | '!' | '?' | ':' | ';' | ')' | ']' | '}'))
            .unwrap_or(false);

    if needs_space {
        target.push(' ');
    }
    target.push_str(addition);
}
