$ErrorActionPreference = 'Stop'

$configPath = Join-Path $env:APPDATA 'PiboLocalAsrTray\config.json'

if (-not (Test-Path $configPath)) {
    Write-Host "[edit-dictionary] config not found at $configPath"
    Write-Host '[edit-dictionary] start the app once first so the default config gets created.'
    exit 1
}

Write-Host "[edit-dictionary] opening $configPath"
Start-Process notepad.exe $configPath
