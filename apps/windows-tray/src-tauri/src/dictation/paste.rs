use std::{thread, time::Duration};

use anyhow::{anyhow, Context};
use arboard::Clipboard;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_V,
};

#[derive(Debug, Clone)]
pub struct PasteOutcome {
    pub clipboard_restored: bool,
}

pub fn copy_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new().context("failed to access clipboard")?;
    clipboard
        .set_text(text.to_string())
        .context("failed to set clipboard text")?;
    Ok(())
}

pub fn paste_text(text: &str, restore_clipboard: bool) -> anyhow::Result<PasteOutcome> {
    copy_text(text)?;

    send_ctrl_v()?;

    if restore_clipboard {
        // Keep the transcript in the clipboard as well, even after a successful paste.
        thread::sleep(Duration::from_millis(120));
        copy_text(text)?;
        return Ok(PasteOutcome {
            clipboard_restored: false,
        });
    }

    Ok(PasteOutcome {
        clipboard_restored: false,
    })
}

fn send_ctrl_v() -> anyhow::Result<()> {
    let inputs = [
        keyboard_input(VK_CONTROL, false),
        keyboard_input(VK_V, false),
        keyboard_input(VK_V, true),
        keyboard_input(VK_CONTROL, true),
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(anyhow!("SendInput failed to dispatch Ctrl+V"));
    }

    Ok(())
}

fn keyboard_input(key: VIRTUAL_KEY, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { Default::default() },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
