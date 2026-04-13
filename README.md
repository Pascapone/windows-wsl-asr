# Pibo Local ASR Tray

Lokale Windows-Tray-App für Diktat mit:

- Windows 11 Tray-Client
- WSL2-Ubuntu-Backend
- `qwen-asr[vllm]`
- `Qwen/Qwen3-ASR-1.7B`
- NVIDIA-GPU

Der aktuelle Stand ist auf diese Maschine zugeschnitten und läuft lokal mit WSL als Backend.

## Schnellstart

### App starten

```powershell
cd C:\Users\pasca\Coding\asr-app
powershell -ExecutionPolicy Bypass -File .\scripts\start-app.ps1
```

### Dictionary bearbeiten

In der App:

- Tray-Icon anklicken
- Bereich `Dictionary` bearbeiten
- `Dictionary Speichern` oder `Speichern + Backend Neustarten`

Oder direkt per Config-Datei:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\edit-dictionary.ps1
```

## Nutzung

- Standard-Hotkey: `Ctrl+Shift+Space`
- zusätzlicher Hotkey: `Rollen`
- Hotkeys arbeiten als Toggle: einmal starten, einmal stoppen
- finale Transkripte werden in die Zwischenablage geschrieben und optional direkt eingefügt

Weitere Nutzungsdetails stehen in [docs/usage.md](./docs/usage.md).

## Repo-Struktur

```text
.
|-- apps/
|   `-- windows-tray/
|-- backend/
|   `-- wsl-qwen-asr/
|-- docs/
|   |-- usage.md
|   `-- ...
|-- scripts/
|   |-- start-app.ps1
|   `-- edit-dictionary.ps1
|-- shared/
|   `-- contracts/
|-- implementation-plan.md
`-- spec.md
```

## Entwickler-Setup

### Windows

- Rust toolchain
- Node.js
- Visual Studio Build Tools / MSVC

### WSL

- Ubuntu
- Python 3.12
- NVIDIA-Unterstützung in WSL

Backend bootstrap:

```bash
cd /mnt/c/Users/pasca/Coding/asr-app/backend/wsl-qwen-asr
bash scripts/bootstrap.sh
```

Frontend/Tray-App:

```powershell
cd C:\Users\pasca\Coding\asr-app\apps\windows-tray
npm install
npm run build
cd .\src-tauri
cargo build
```

## Laufzeitdateien

- Config: `%AppData%\PiboLocalAsrTray\config.json`
- Windows-Log: `%LocalAppData%\PiboLocalAsrTray\logs\app.log`
- WSL-Backend-Log: `~/.local/state/pibo-local-asr-tray/backend.log`

## Dokumente

- Produktspezifikation: [spec.md](./spec.md)
- Implementierungsplan: [implementation-plan.md](./implementation-plan.md)
- Nutzung: [docs/usage.md](./docs/usage.md)
- Backend-API: [shared/contracts/backend-api.md](./shared/contracts/backend-api.md)
