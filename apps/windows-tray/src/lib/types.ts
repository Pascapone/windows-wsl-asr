export type BackendStatus = 'stopped' | 'starting' | 'ready' | 'error'
export type DictationStatus = 'idle' | 'backend_starting' | 'backend_ready' | 'recording' | 'finalizing' | 'error'

export type AppConfig = {
  general: {
    startOnLogin: boolean
  }
  backend: {
    wslDistro: string
    host: string
    port: number
    modelName: string
    gpuMemoryUtilization: number
    chunkSizeSec: number
    autoStartBackend: boolean
  }
  capture: {
    hotkey: string
    inputDeviceId: string | null
  }
  audioProcessing: {
    highPassEnabled: boolean
    autoGainEnabled: boolean
    compressorEnabled: boolean
    outputGainEnabled: boolean
    limiterEnabled: boolean
    meteringEnabled: boolean
    highPassCutoffHz: number
    targetRmsDb: number
    autoGainMinDb: number
    autoGainMaxDb: number
    autoGainAttackMs: number
    autoGainReleaseMs: number
    compressorThresholdDb: number
    compressorRatio: number
    compressorAttackMs: number
    compressorReleaseMs: number
    outputGainDb: number
    limiterCeilingDb: number
  }
  overlay: {
    enabled: boolean
    anchor: 'mouse' | 'center'
    offsetX: number
    offsetY: number
    maxWidth: number
  }
  dictation: {
    languageHint: string | null
    autoPaste: boolean
    restoreClipboard: boolean
  }
  dictionary: {
    terms: string[]
  }
}

export type AppSnapshot = {
  config: AppConfig
  backendStatus: BackendStatus
  dictationStatus: DictationStatus
  backendOwned: boolean
  backendModelLoaded: boolean
  audioMetrics: {
    inputRmsDb: number
    outputRmsDb: number
    inputPeakDb: number
    outputPeakDb: number
    currentGainDb: number
    gainReductionDb: number
    clipCount: number
    limitCount: number
    droppedChunks: number
  } | null
  partialText: string
  lastTranscript: string | null
  errorMessage: string | null
}

export type AudioDeviceInfo = {
  id: string
  name: string
  isDefault: boolean
}
