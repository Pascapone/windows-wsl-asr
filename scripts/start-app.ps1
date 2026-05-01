$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$windowsTrayDir = Join-Path $repoRoot 'apps\windows-tray'
$tauriDir = Join-Path $windowsTrayDir 'src-tauri'
$distIndex = Join-Path $windowsTrayDir 'dist\index.html'
$exePath = Join-Path $tauriDir 'target\debug\pibo-local-asr-tray.exe'

if (-not (Test-Path (Join-Path $windowsTrayDir 'node_modules'))) {
    Write-Host '[start-app] installing npm dependencies...'
    Push-Location $windowsTrayDir
    try {
        npm install
    }
    finally {
        Pop-Location
    }
}

$needsRustBuild = -not (Test-Path $exePath)
if (-not $needsRustBuild) {
    $exeTimestamp = (Get-Item $exePath).LastWriteTime
    $latestTauriSource = Get-ChildItem $tauriDir -Recurse -File |
        Where-Object {
            $_.FullName -notlike (Join-Path $tauriDir 'target\*') -and
            ($_.Extension -in '.rs', '.toml', '.json')
        } |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1

    $needsRustBuild = $latestTauriSource -and $latestTauriSource.LastWriteTime -gt $exeTimestamp
}

if ($needsRustBuild) {
    if (Get-Process pibo-local-asr-tray -ErrorAction SilentlyContinue) {
        Write-Host '[start-app] app is already running; close it before rebuilding'
        exit 0
    }

    if (-not (Test-Path $distIndex)) {
        Write-Host '[start-app] building frontend bundle...'
        Push-Location $windowsTrayDir
        try {
            npm run build
        }
        finally {
            Pop-Location
        }
    }

    Write-Host '[start-app] building debug app...'
    Push-Location $tauriDir
    try {
        cargo build
    }
    finally {
        Pop-Location
    }
}

if (Get-Process pibo-local-asr-tray -ErrorAction SilentlyContinue) {
    Write-Host '[start-app] app is already running'
    exit 0
}

Write-Host "[start-app] launching $exePath"
Start-Process $exePath
