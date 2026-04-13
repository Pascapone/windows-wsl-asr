use std::str::FromStr;

use anyhow::{anyhow, Context};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::AppContext;

pub fn register_hotkey(app: &AppHandle, accelerator: &str) -> anyhow::Result<()> {
    let primary = Shortcut::from_str(accelerator)
        .or_else(|_| parse_fallback_shortcut(accelerator))
        .with_context(|| format!("invalid hotkey: {accelerator}"))?;
    let secondary = Shortcut::from_str("ScrollLock")
        .or_else(|_| parse_fallback_shortcut("ScrollLock"))
        .context("invalid fallback hotkey: ScrollLock")?;

    let manager = app.global_shortcut();
    let _ = manager.unregister_all();
    let app_handle = app.clone();
    register_shortcut(&manager, &app_handle, primary)?;
    if secondary != primary {
        register_shortcut(&manager, &app_handle, secondary)?;
    }
    Ok(())
}

fn handle_shortcut_event(app: &AppHandle, _shortcut: Shortcut, event: ShortcutEvent) {
    let Some(context) = app.try_state::<AppContext>() else {
        return;
    };

    if event.state() != ShortcutState::Pressed {
        return;
    }

    let app = app.clone();
    let context = context.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = context.toggle_recording(&app).await {
            log::error!("failed to toggle recording from hotkey: {error}");
        }
    });
}

fn register_shortcut(
    manager: &tauri_plugin_global_shortcut::GlobalShortcut<tauri::Wry>,
    app_handle: &AppHandle,
    shortcut: Shortcut,
) -> anyhow::Result<()> {
    let app_handle = app_handle.clone();
    manager
        .on_shortcut(shortcut, move |_app, shortcut, event| {
            handle_shortcut_event(&app_handle, *shortcut, event)
        })
        .with_context(|| format!("failed to register global shortcut {}", shortcut))
}

fn parse_fallback_shortcut(value: &str) -> anyhow::Result<Shortcut> {
    let mut modifiers = Vec::new();
    let mut code = None;
    for part in value.split('+').map(str::trim).filter(|part| !part.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers.push("CTRL"),
            "shift" => modifiers.push("SHIFT"),
            "alt" => modifiers.push("ALT"),
            "meta" | "win" | "super" => modifiers.push("META"),
            "space" => code = Some(Code::Space),
            "rollen" | "scrolllock" | "scroll_lock" | "scroll-lock" => code = Some(Code::ScrollLock),
            "enter" => code = Some(Code::Enter),
            key if key.len() == 1 => {
                code = Some(match key.chars().next().unwrap().to_ascii_uppercase() {
                    'A' => Code::KeyA,
                    'B' => Code::KeyB,
                    'C' => Code::KeyC,
                    'D' => Code::KeyD,
                    'E' => Code::KeyE,
                    'F' => Code::KeyF,
                    'G' => Code::KeyG,
                    'H' => Code::KeyH,
                    'I' => Code::KeyI,
                    'J' => Code::KeyJ,
                    'K' => Code::KeyK,
                    'L' => Code::KeyL,
                    'M' => Code::KeyM,
                    'N' => Code::KeyN,
                    'O' => Code::KeyO,
                    'P' => Code::KeyP,
                    'Q' => Code::KeyQ,
                    'R' => Code::KeyR,
                    'S' => Code::KeyS,
                    'T' => Code::KeyT,
                    'U' => Code::KeyU,
                    'V' => Code::KeyV,
                    'W' => Code::KeyW,
                    'X' => Code::KeyX,
                    'Y' => Code::KeyY,
                    'Z' => Code::KeyZ,
                    _ => return Err(anyhow!("unsupported hotkey key: {part}")),
                })
            }
            _ => return Err(anyhow!("unsupported hotkey part: {part}")),
        }
    }

    let code = code.ok_or_else(|| anyhow!("hotkey must include a non-modifier key"))?;
    let text = if modifiers.is_empty() {
        code.to_string()
    } else {
        format!("{}+{}", modifiers.join("+"), code)
    };
    Shortcut::from_str(&text).with_context(|| format!("failed to normalize shortcut: {value}"))
}
