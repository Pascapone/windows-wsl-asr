mod app_state;
mod backend_manager;
mod config;
mod hotkey;
mod logging;

mod audio {
    pub mod capture;
    pub mod processing;
    pub mod resample;
}

mod dictation {
    pub mod backend_client;
    pub mod paste;
    pub mod session_controller;
}

mod overlay {
    pub mod window;
}

use anyhow::Context;
use app_state::{AppSnapshot, StateStore};
use backend_manager::BackendManager;
use config::AppConfig;
use dictation::session_controller::SessionController;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, Wry,
};

#[derive(Clone)]
pub struct AppContext {
    state: StateStore,
    backend_manager: BackendManager,
    session_controller: SessionController,
}

impl AppContext {
    fn show_main_window(&self, app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }

    async fn ensure_backend_loading(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.show_main_window(app);
        let snapshot = self.state.snapshot().await;
        if snapshot.backend_status == app_state::BackendStatus::Ready {
            return Ok(());
        }

        if snapshot.config.backend.auto_start_backend && snapshot.backend_status == app_state::BackendStatus::Stopped {
            self.backend_manager.start(app, &self.state).await?;
        }

        self.state
            .update(|snapshot| {
                snapshot.backend_status = app_state::BackendStatus::Starting;
                snapshot.dictation_status = app_state::DictationStatus::BackendStarting;
                snapshot.error_message = None;
            })
            .await;
        emit_snapshot(app, &self.state).await?;
        Ok(())
    }

    async fn save_config(&self, app: &AppHandle, config: AppConfig) -> anyhow::Result<AppSnapshot> {
        config
            .save_to(self.state.config_path())
            .with_context(|| format!("failed to save {}", self.state.config_path().display()))?;
        hotkey::register_hotkey(app, &config.capture.hotkey)?;
        let snapshot = self
            .state
            .update(|snapshot| {
                snapshot.config = config.clone();
                snapshot.error_message = None;
            })
            .await;
        emit_snapshot(app, &self.state).await?;
        Ok(snapshot)
    }

    async fn reload_config(&self, app: &AppHandle) -> anyhow::Result<AppSnapshot> {
        let (config, _) = AppConfig::load_or_create()?;
        hotkey::register_hotkey(app, &config.capture.hotkey)?;
        let snapshot = self
            .state
            .update(|snapshot| {
                snapshot.config = config.clone();
                snapshot.error_message = None;
            })
            .await;
        emit_snapshot(app, &self.state).await?;
        Ok(snapshot)
    }

    async fn start_backend(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.backend_manager.start(app, &self.state).await
    }

    async fn stop_backend(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.backend_manager.stop(app, &self.state).await
    }

    async fn restart_backend(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.backend_manager.restart(app, &self.state).await
    }

    async fn start_recording(&self, app: &AppHandle) -> anyhow::Result<()> {
        let snapshot = self.state.snapshot().await;
        if snapshot.backend_status != app_state::BackendStatus::Ready {
            self.ensure_backend_loading(app).await?;
            return Ok(());
        }

        self.session_controller
            .start_recording(app, &self.state, &self.backend_manager)
            .await
    }

    async fn toggle_recording(&self, app: &AppHandle) -> anyhow::Result<()> {
        let snapshot = self.state.snapshot().await;
        if self.session_controller.is_recording().await || snapshot.dictation_status == app_state::DictationStatus::Recording {
            self.session_controller.stop_recording(app, &self.state).await
        } else if snapshot.backend_status != app_state::BackendStatus::Ready
            || snapshot.dictation_status == app_state::DictationStatus::BackendStarting
            || snapshot.dictation_status == app_state::DictationStatus::Finalizing
        {
            self.ensure_backend_loading(app).await
        } else {
            self.session_controller
                .start_recording(app, &self.state, &self.backend_manager)
                .await
        }
    }

    async fn stop_recording(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.session_controller.stop_recording(app, &self.state).await
    }

    async fn cancel_recording(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.session_controller.cancel_recording(app, &self.state).await
    }
}

#[tauri::command]
async fn get_snapshot(context: State<'_, AppContext>) -> Result<AppSnapshot, String> {
    Ok(context.state.snapshot().await)
}

#[tauri::command]
async fn save_config(app: AppHandle, context: State<'_, AppContext>, config: AppConfig) -> Result<AppSnapshot, String> {
    context.save_config(&app, config).await.map_err(err_to_string)
}

#[tauri::command]
async fn reload_config(app: AppHandle, context: State<'_, AppContext>) -> Result<AppSnapshot, String> {
    context.reload_config(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn list_input_devices() -> Result<Vec<audio::capture::AudioDeviceInfo>, String> {
    audio::capture::list_input_devices().map_err(err_to_string)
}

#[tauri::command]
async fn refresh_audio_devices() -> Result<Vec<audio::capture::AudioDeviceInfo>, String> {
    audio::capture::list_input_devices().map_err(err_to_string)
}

#[tauri::command]
async fn start_backend(app: AppHandle, context: State<'_, AppContext>) -> Result<(), String> {
    context.start_backend(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn stop_backend(app: AppHandle, context: State<'_, AppContext>) -> Result<(), String> {
    context.stop_backend(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn restart_backend(app: AppHandle, context: State<'_, AppContext>) -> Result<(), String> {
    context.restart_backend(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn start_recording(app: AppHandle, context: State<'_, AppContext>) -> Result<(), String> {
    context.start_recording(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn stop_recording(app: AppHandle, context: State<'_, AppContext>) -> Result<(), String> {
    context.stop_recording(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn cancel_recording(app: AppHandle, context: State<'_, AppContext>) -> Result<(), String> {
    context.cancel_recording(&app).await.map_err(err_to_string)
}

#[tauri::command]
async fn open_logs(context: State<'_, AppContext>) -> Result<(), String> {
    let log_path = context.state.log_path().display().to_string();
    std::process::Command::new("explorer.exe")
        .arg(log_path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn open_config(context: State<'_, AppContext>) -> Result<(), String> {
    let config_path = context.state.config_path().display().to_string();
    std::process::Command::new("notepad.exe")
        .arg(config_path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    dictation::paste::copy_text(&text).map_err(err_to_string)
}

fn err_to_string(error: anyhow::Error) -> String {
    error.to_string()
}

pub async fn emit_snapshot(app: &AppHandle, state: &StateStore) -> anyhow::Result<()> {
    let snapshot = state.snapshot().await;
    app.emit("state://changed", snapshot).map_err(Into::into)
}

fn build_tray(app: &AppHandle) -> anyhow::Result<()> {
    let start = MenuItem::with_id(app, "start_recording", "Start Recording", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop_recording", "Stop Recording", true, None::<&str>)?;
    let cancel = MenuItem::with_id(app, "cancel_recording", "Cancel Recording", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open_settings", "Open Settings", true, None::<&str>)?;
    let start_backend = MenuItem::with_id(app, "start_backend", "Start Backend", true, None::<&str>)?;
    let stop_backend = MenuItem::with_id(app, "stop_backend", "Stop Backend", true, None::<&str>)?;
    let restart_backend = MenuItem::with_id(app, "restart_backend", "Restart Backend", true, None::<&str>)?;
    let open_logs = MenuItem::with_id(app, "open_logs", "Open Logs", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&start, &stop, &cancel, &open, &start_backend, &stop_backend, &restart_backend, &open_logs, &quit],
    )?;

    let context = app.state::<AppContext>().inner().clone();
    let menu_app_handle = app.clone();
    let click_app_handle = app.clone();
    TrayIconBuilder::with_id("pibo-tray")
        .icon(overlay::window::tray_icon()?)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |_tray, event| {
            let app = menu_app_handle.clone();
            let context = context.clone();
            match event.id().0.as_str() {
                "start_recording" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = context.start_recording(&app).await;
                    });
                }
                "stop_recording" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = context.stop_recording(&app).await;
                    });
                }
                "cancel_recording" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = context.cancel_recording(&app).await;
                    });
                }
                "open_settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "start_backend" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = context.start_backend(&app).await;
                    });
                }
                "stop_backend" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = context.stop_backend(&app).await;
                    });
                }
                "restart_backend" => {
                    tauri::async_runtime::spawn(async move {
                        let _ = context.restart_backend(&app).await;
                    });
                }
                "open_logs" => {
                    let _ = std::process::Command::new("explorer.exe")
                        .arg(context.state.log_path().display().to_string())
                        .spawn();
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = click_app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

pub fn run() {
    let (config, config_path) = AppConfig::load_or_create().expect("failed to load config");
    let log_path = AppConfig::log_path().expect("failed to resolve log path");
    logging::init_logging(&log_path).expect("failed to initialize logging");
    let snapshot = AppSnapshot::new(config.clone());
    let state = StateStore::new(snapshot, config_path, log_path);
    let context = AppContext {
        state: state.clone(),
        backend_manager: BackendManager::default(),
        session_controller: SessionController::default(),
    };

    tauri::Builder::<Wry>::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(context)
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            save_config,
            reload_config,
            list_input_devices,
            refresh_audio_devices,
            start_backend,
            stop_backend,
            restart_backend,
            start_recording,
            stop_recording,
            cancel_recording,
            open_logs,
            open_config,
            copy_text_to_clipboard
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let context = app.state::<AppContext>().inner().clone();
            build_tray(&app_handle)?;
            overlay::window::ensure_overlay(&app_handle)?;
            hotkey::register_hotkey(&app_handle, &config.capture.hotkey)?;
            context
                .backend_manager
                .spawn_health_poller(app_handle.clone(), context.state.clone());
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.hide();
            }
            tauri::async_runtime::block_on(async {
                if config.backend.auto_start_backend {
                    let _ = context
                        .state
                        .update(|snapshot| {
                            snapshot.backend_status = app_state::BackendStatus::Starting;
                            snapshot.dictation_status = app_state::DictationStatus::BackendStarting;
                            snapshot.error_message = None;
                        })
                        .await;
                }
                let _ = emit_snapshot(&app_handle, &context.state).await;
            });
            if config.backend.auto_start_backend {
                let app_handle = app_handle.clone();
                let context = context.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = context.start_backend(&app_handle).await {
                        log::error!("auto-start backend failed: {error:#}");
                    }
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
