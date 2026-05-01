import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, AppSnapshot, AudioDeviceInfo } from './types'

export function getSnapshot() {
  return invoke<AppSnapshot>('get_snapshot')
}

export function listInputDevices() {
  return invoke<AudioDeviceInfo[]>('list_input_devices')
}

export function saveConfig(config: AppConfig) {
  return invoke<AppSnapshot>('save_config', { config })
}

export function reloadConfig() {
  return invoke<AppSnapshot>('reload_config')
}

export function startBackend() {
  return invoke<void>('start_backend')
}

export function stopBackend() {
  return invoke<void>('stop_backend')
}

export function restartBackend() {
  return invoke<void>('restart_backend')
}

export function startRecording() {
  return invoke<void>('start_recording')
}

export function stopRecording() {
  return invoke<void>('stop_recording')
}

export function cancelRecording() {
  return invoke<void>('cancel_recording')
}

export function refreshAudioDevices() {
  return invoke<AudioDeviceInfo[]>('refresh_audio_devices')
}

export function openLogs() {
  return invoke<void>('open_logs')
}

export function openConfig() {
  return invoke<void>('open_config')
}

export function copyTextToClipboard(text: string) {
  return invoke<void>('copy_text_to_clipboard', { text })
}
