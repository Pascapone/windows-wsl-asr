use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context};
use tauri::AppHandle;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
    time::sleep,
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::{
    app_state::{BackendStatus, DictationStatus, StateStore},
    dictation::backend_client::BackendClient,
    emit_snapshot,
};

#[derive(Clone, Default)]
pub struct BackendManager {
    child: Arc<Mutex<Option<Child>>>,
    poller_started: Arc<AtomicBool>,
}

impl BackendManager {
    pub async fn start(&self, app: &AppHandle, state: &StateStore) -> anyhow::Result<()> {
        if self.has_running_child().await {
            state
                .update(|snapshot| {
                    snapshot.backend_status = BackendStatus::Starting;
                    snapshot.dictation_status = DictationStatus::BackendStarting;
                    snapshot.backend_owned = true;
                    snapshot.error_message = None;
                })
                .await;
            emit_snapshot(app, state).await?;
            return Ok(());
        }

        let snapshot = state.snapshot().await;
        if snapshot.backend_status == BackendStatus::Ready || snapshot.backend_status == BackendStatus::Starting {
            return Ok(());
        }

        let config = snapshot.config.clone();
        let client = BackendClient::new(&config.backend);
        if let Ok(health) = client.health().await {
            log::info!(
                "adopting existing backend ok={} model_loaded={}",
                health.ok,
                health.model_loaded
            );
            state
                .update(|snapshot| {
                    snapshot.backend_status = if health.model_loaded {
                        BackendStatus::Ready
                    } else {
                        BackendStatus::Starting
                    };
                    snapshot.backend_model_loaded = health.model_loaded;
                    snapshot.backend_owned = false;
                    if health.model_loaded && snapshot.dictation_status == DictationStatus::BackendStarting {
                        snapshot.dictation_status = DictationStatus::Idle;
                    }
                    snapshot.error_message = None;
                })
                .await;
            emit_snapshot(app, state).await?;
            return Ok(());
        }

        let backend_dir = backend_dir()?;
        let wsl_backend_dir = to_wsl_path(&backend_dir)?;
        let command_line = format!(
            "PIBO_ASR_HOST=127.0.0.1 PIBO_ASR_PORT={} PIBO_ASR_MODEL='{}' PIBO_ASR_GPU_MEMORY_UTILIZATION={} PIBO_ASR_CHUNK_SIZE_SEC={} bash scripts/run_server.sh",
            config.backend.port,
            config.backend.model_name.replace('\'', "'\\''"),
            config.backend.gpu_memory_utilization,
            config.backend.chunk_size_sec
        );
        log::info!(
            "launching backend distro={} cwd={} port={} gpu_memory_utilization={}",
            config.backend.wsl_distro,
            wsl_backend_dir,
            config.backend.port,
            config.backend.gpu_memory_utilization
        );

        let mut command = Command::new("wsl.exe");
        command
            .args(["-d", &config.backend.wsl_distro, "--cd", &wsl_backend_dir, "bash", "-lc", &command_line])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        hide_console_window(&mut command);

        let mut child = command.spawn().context("failed to launch backend via wsl.exe")?;
        log::info!("spawned backend bridge pid={:?}", child.id());
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(pipe_child_output(stdout, "wsl-backend:stdout"));
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(pipe_child_output(stderr, "wsl-backend:stderr"));
        }

        *self.child.lock().await = Some(child);
        state
            .update(|snapshot| {
                snapshot.backend_status = BackendStatus::Starting;
                snapshot.dictation_status = DictationStatus::BackendStarting;
                snapshot.backend_owned = true;
                snapshot.error_message = None;
            })
            .await;
        emit_snapshot(app, state).await?;
        Ok(())
    }

    pub async fn stop(&self, app: &AppHandle, state: &StateStore) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.lock().await.take() {
            log::info!("stopping backend bridge pid={:?}", child.id());
            child.kill().await.ok();
            child.wait().await.ok();
        }

        let snapshot = state.snapshot().await;
        let cleanup_cmd = "pkill -f 'uvicorn app.server:app' || true";
        let mut cleanup = Command::new("wsl.exe");
        cleanup
            .args(["-d", &snapshot.config.backend.wsl_distro, "bash", "-lc", cleanup_cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        hide_console_window(&mut cleanup);
        let _ = cleanup.spawn();

        state
            .update(|snapshot| {
                snapshot.backend_status = BackendStatus::Stopped;
                snapshot.backend_owned = false;
                snapshot.backend_model_loaded = false;
                if snapshot.dictation_status != DictationStatus::Recording {
                    snapshot.dictation_status = DictationStatus::Idle;
                }
            })
            .await;
        emit_snapshot(app, state).await?;
        Ok(())
    }

    pub async fn restart(&self, app: &AppHandle, state: &StateStore) -> anyhow::Result<()> {
        self.stop(app, state).await?;
        self.start(app, state).await
    }

    pub async fn wait_until_ready(&self, app: &AppHandle, state: &StateStore, timeout: Duration) -> anyhow::Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() > deadline {
                return Err(anyhow!("Backend did not become ready within {} seconds", timeout.as_secs()));
            }
            let snapshot = state.snapshot().await;
            let client = BackendClient::new(&snapshot.config.backend);
            match client.health().await {
                Ok(health) if health.ok && health.model_loaded => {
                    state
                        .update(|snapshot| {
                            snapshot.backend_status = BackendStatus::Ready;
                            snapshot.backend_model_loaded = true;
                            if snapshot.dictation_status == DictationStatus::BackendStarting {
                                snapshot.dictation_status = DictationStatus::Idle;
                            }
                            snapshot.error_message = None;
                        })
                        .await;
                    emit_snapshot(app, state).await?;
                    return Ok(());
                }
                Ok(_) => {
                    state
                        .update(|snapshot| {
                            snapshot.backend_status = BackendStatus::Starting;
                            snapshot.backend_model_loaded = false;
                            snapshot.error_message = None;
                            if snapshot.dictation_status != DictationStatus::Recording
                                && snapshot.dictation_status != DictationStatus::Finalizing
                                && snapshot.dictation_status != DictationStatus::Error
                            {
                                snapshot.dictation_status = DictationStatus::BackendStarting;
                            }
                        })
                        .await;
                    emit_snapshot(app, state).await?;
                }
                Err(error) => {
                    log::debug!("Backend health probe failed while waiting: {error}");
                }
            }
            sleep(Duration::from_millis(600)).await;
        }
    }

    pub fn spawn_health_poller(&self, app: AppHandle, state: StateStore) {
        if self.poller_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let snapshot = state.snapshot().await;
                let client = BackendClient::new(&snapshot.config.backend);
                let child_exited = manager.child_exited().await;
                match client.health().await {
                    Ok(health) if health.ok && health.model_loaded => {
                        state
                            .update(|snapshot| {
                                snapshot.backend_status = BackendStatus::Ready;
                                snapshot.backend_model_loaded = true;
                                snapshot.backend_owned = !child_exited;
                                if snapshot.dictation_status == DictationStatus::BackendStarting {
                                    snapshot.dictation_status = DictationStatus::Idle;
                                }
                                snapshot.error_message = None;
                            })
                            .await;
                    }
                    Ok(_) => {
                        state
                            .update(|snapshot| {
                                snapshot.backend_status = BackendStatus::Starting;
                                snapshot.backend_model_loaded = false;
                            })
                            .await;
                    }
                    Err(error) => {
                        state
                            .update(|snapshot| {
                                snapshot.backend_model_loaded = false;
                                let expected_startup_failure = snapshot.backend_owned
                                    && snapshot.backend_status == BackendStatus::Starting
                                    && !child_exited;
                                snapshot.backend_status = if snapshot.backend_owned {
                                    BackendStatus::Starting
                                } else {
                                    BackendStatus::Stopped
                                };
                                if child_exited {
                                    snapshot.backend_owned = false;
                                }
                                if expected_startup_failure {
                                    snapshot.error_message = None;
                                    if snapshot.dictation_status != DictationStatus::Recording
                                        && snapshot.dictation_status != DictationStatus::Finalizing
                                        && snapshot.dictation_status != DictationStatus::Error
                                    {
                                        snapshot.dictation_status = DictationStatus::BackendStarting;
                                    }
                                    return;
                                }
                                if snapshot.dictation_status != DictationStatus::Recording
                                    && snapshot.dictation_status != DictationStatus::Finalizing
                                    && snapshot.dictation_status != DictationStatus::Error
                                {
                                    snapshot.dictation_status = DictationStatus::Idle;
                                }
                                if snapshot.backend_owned {
                                    snapshot.error_message = Some(error.to_string());
                                }
                            })
                            .await;
                    }
                }
                let _ = emit_snapshot(&app, &state).await;
                sleep(Duration::from_secs(2)).await;
            }
        });
    }

    async fn child_exited(&self) -> bool {
        let mut guard = self.child.lock().await;
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    *guard = None;
                    true
                }
                Ok(None) => false,
                Err(_) => true,
            }
        } else {
            true
        }
    }

    async fn has_running_child(&self) -> bool {
        let mut guard = self.child.lock().await;
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    *guard = None;
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    *guard = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

async fn pipe_child_output<T>(stream: T, source: &'static str)
where
    T: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::info!("[{source}] {line}");
    }
}

#[cfg(windows)]
fn hide_console_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console_window(_command: &mut Command) {}

fn backend_dir() -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../backend/wsl-qwen-asr")
        .canonicalize()
        .context("failed to resolve backend/wsl-qwen-asr path")?;
    Ok(path)
}

fn to_wsl_path(path: &Path) -> anyhow::Result<String> {
    let mut text = path
        .to_str()
        .ok_or_else(|| anyhow!("backend path is not valid UTF-8"))?
        .replace('\\', "/");
    if let Some(stripped) = text.strip_prefix("//?/") {
        text = stripped.to_string();
    }
    if text.len() < 2 || !text.as_bytes()[1].eq(&b':') {
        return Err(anyhow!("expected Windows drive path, got {}", text));
    }
    let drive = text[..1].to_ascii_lowercase();
    let rest = &text[2..];
    Ok(format!("/mnt/{drive}{rest}"))
}
