# Nutzung

## App Starten

Die einfachste Variante auf Windows:

```powershell
cd C:\Users\pasca\Coding\asr-app
powershell -ExecutionPolicy Bypass -File .\scripts\start-app.ps1
```

Dadurch wird die vorhandene Debug-Binary gestartet. Falls sie noch nicht gebaut ist, baut das Script sie zuerst.

## Dictionary Bearbeiten

Es gibt zwei Wege:

### 1. In der App

- Tray-App starten
- auf das Tray-Icon klicken
- im Hauptfenster den Bereich `Dictionary` bearbeiten
- `Dictionary Speichern` oder `Speichern + Backend Neustarten` klicken

### 2. Direkt per Datei

```powershell
cd C:\Users\pasca\Coding\asr-app
powershell -ExecutionPolicy Bypass -File .\scripts\edit-dictionary.ps1
```

Die Datei liegt unter:

```text
%AppData%\PiboLocalAsrTray\config.json
```

Das Dictionary steht dort unter:

```json
"dictionary": {
  "terms": [
    "Pibo",
    "OpenClaw",
    "Pascal"
  ]
}
```

## Aufnahme

- Backend starten lassen oder im Fenster `Backend Starten` klicken
- Hotkey `Ctrl+Shift+Space` oder `Rollen` verwenden
- erneut drücken zum Stoppen

## Logs

- Windows-App: `%LocalAppData%\PiboLocalAsrTray\logs\app.log`
- WSL-Backend: `~/.local/state/pibo-local-asr-tray/backend.log`
