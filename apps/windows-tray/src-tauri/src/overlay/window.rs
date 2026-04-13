use std::mem::size_of;

use anyhow::Context;
use tauri::{
    image::Image,
    LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};
use windows::Win32::{
    Foundation::POINT,
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::WindowsAndMessaging::GetCursorPos,
};

use crate::{app_state::StateStore, emit_snapshot};

pub fn ensure_overlay(app: &tauri::AppHandle) -> anyhow::Result<()> {
    if app.get_webview_window("overlay").is_some() {
        return Ok(());
    }

    let _window = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html#/overlay".into()))
        .title("Pibo Overlay")
        .always_on_top(true)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .transparent(true)
        .visible(false)
        .inner_size(420.0, 108.0)
        .build()
        .context("failed to build overlay window")?;

    Ok(())
}

pub async fn show_overlay(app: &tauri::AppHandle, state: &StateStore) -> anyhow::Result<()> {
    ensure_overlay(app)?;
    let window = app
        .get_webview_window("overlay")
        .context("overlay window missing after creation")?;
    position_overlay(&window, state).await.context("failed to position overlay")?;
    window.show().context("failed to show overlay")?;
    emit_snapshot(app, state).await?;
    Ok(())
}

pub fn hide_overlay(app: &tauri::AppHandle) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.hide().context("failed to hide overlay")?;
    }
    Ok(())
}

async fn position_overlay(window: &tauri::WebviewWindow, state: &StateStore) -> anyhow::Result<()> {
    let snapshot = state.snapshot().await;
    let mut cursor = POINT::default();
    unsafe {
        GetCursorPos(&mut cursor)?;
    }

    let monitor = unsafe { MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetMonitorInfoW(monitor, &mut info) }.ok().context("failed to query monitor info")?;

    let work = info.rcWork;
    let work_width = work.right - work.left;
    let work_height = work.bottom - work.top;
    let overlay_width = (snapshot.config.overlay.max_width as i32).min(work_width.saturating_sub(24)).max(280);
    let overlay_height = 108i32.min(work_height.saturating_sub(24)).max(92);
    let max_x = (work.right - overlay_width).max(work.left);
    let max_y = (work.bottom - overlay_height).max(work.top);

    let (x, y) = if snapshot.config.overlay.anchor == "mouse" {
        (
            (cursor.x + snapshot.config.overlay.offset_x).clamp(work.left, max_x),
            (cursor.y + snapshot.config.overlay.offset_y).clamp(work.top, max_y),
        )
    } else {
        (
            ((work.left + work.right - overlay_width) / 2 + snapshot.config.overlay.offset_x).clamp(work.left, max_x),
            (work.bottom - overlay_height - 24 + snapshot.config.overlay.offset_y).clamp(work.top, max_y),
        )
    };

    window.set_size(LogicalSize::new(overlay_width as f64, overlay_height as f64))?;
    window.set_position(LogicalPosition::new(x as f64, y as f64))?;
    Ok(())
}

pub fn tray_icon() -> anyhow::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../../../src/assets/hero.png"))
        .context("failed to decode tray icon bytes")
}
