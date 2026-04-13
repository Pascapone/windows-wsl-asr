# Pibo Local ASR Tray Implementation Plan

Stand: 2026-04-12
Status: Handoff Plan

## 1. Ziel dieses Plans

Dieser Plan beschreibt die konkrete Baufolge fuer das Projekt `pibo-local-asr-tray`.

Er ist fuer einen Agenten gedacht, der auf einem Windows-PC mit WSL2 und NVIDIA-GPU arbeitet und **keinen** Zugriff auf fruehere interne ASR-Projekte hat.

Der Agent soll mit diesem Plan:

- das Repo scaffolden oder erweitern
- die Windows-App bauen
- das WSL-Backend bauen
- lokal testen
- das V1-Endergebnis lauffaehig machen

Dieser Plan ergaenzt die Produktspezifikation in `docs/spec.md`.

---

## 2. Grundsaetze fuer die Umsetzung

### 2.1 Erst das robuste Rueckgrat

Nicht mit Overlay-Polish anfangen.

Zuerst bauen:

- Backend laeuft
- Tray-App laeuft
- Hotkey funktioniert
- Audio geht rein
- finaler Text kommt raus

### 2.2 Erst Finals, dann Partials

Wenn Streaming-Overlay hakt, erst den finalen Fluss stabil machen:

- start
- chunk
- finish
- final text
- paste

Danach Overlay mit Partials sauberziehen.

### 2.3 Kein Architekturdrift

Nicht unterwegs auf Browser-UI, Electron oder Voll-WSLG kippen.

Zielbild bleibt:

- Windows-UX
- WSL-Backend

---

## 3. Empfohlene Arbeitsumgebung auf dem Windows-PC

## 3.1 Windows-Seite

Installiert und verfuegbar:

- Rust toolchain (`rustup`, stable)
- Node.js LTS
- Paketmanager `pnpm` oder `npm`
- Git
- VS Code oder anderer Editor

## 3.2 WSL-Seite

Vorausgesetzt:

- WSL2
- Ubuntu
- funktionierende NVIDIA-CUDA-Unterstuetzung in WSL
- Python `3.12`
- `pip`
- `venv`

## 3.3 Repo-Ort

Empfohlener Repo-Ort auf Windows:

```text
C:\dev\pibo-local-asr-tray
```

WSL sieht dasselbe Repo dann unter:

```text
/mnt/c/dev/pibo-local-asr-tray
```

Hinweis:

- Source auf Windows-Seite ist fuer Tauri-Entwicklung praktisch
- Python-Venv und Modellcache sollen **nicht** auf `/mnt/c` liegen, sondern in WSL-Linux-Pfaden

## 3.4 Konkrete Startkommandos fuer manuelles Arbeiten

Diese Kommandos sind als Debug-/Handarbeitsbasis gedacht.

### Windows PowerShell

Repo wechseln:

```powershell
cd C:\dev\pibo-local-asr-tray
```

### WSL

Repo aus WSL:

```bash
cd /mnt/c/dev/pibo-local-asr-tray
```

Backend bootstrap:

```bash
cd /mnt/c/dev/pibo-local-asr-tray/backend/wsl-qwen-asr
bash scripts/bootstrap.sh
```

Backend manuell starten:

```bash
cd /mnt/c/dev/pibo-local-asr-tray/backend/wsl-qwen-asr
bash scripts/run_server.sh
```

Health pruefen:

```bash
curl http://127.0.0.1:8765/healthz
```

### Windows ruft WSL-Backend auf

Beispiel fuer einen manuellen Start aus PowerShell:

```powershell
wsl.exe -d Ubuntu --cd /mnt/c/dev/pibo-local-asr-tray/backend/wsl-qwen-asr bash -lc "bash scripts/run_server.sh"
```

---

## 4. Zielstruktur nach Abschluss

```text
pibo-local-asr-tray/
  README.md
  docs/
    spec.md
    implementation-plan.md
  apps/
    windows-tray/
      src/
      src-tauri/
      package.json
      tauri.conf.json
  backend/
    wsl-qwen-asr/
      app/
        server.py
        asr_service.py
        settings.py
        schemas.py
      scripts/
        bootstrap.sh
        run_server.sh
        check_env.sh
      requirements.txt
      README.md
```

---

## 5. Delivery-Reihenfolge

Die Umsetzung erfolgt in 10 Phasen.

Keine Phase ueberspringen.
Jede Phase muss mit einem kurzen lokalen Test abgeschlossen werden.

## Phase 1

Repo- und Projektgrundgeruest

## Phase 2

WSL-Backend bootstrap und Modellstart

## Phase 3

Windows-Tauri-Tray-Shell

## Phase 4

Backend-Prozesssteuerung aus Windows

## Phase 5

Globaler Hotkey und Session-State

## Phase 6

Windows-Audio-Capture und Resampling

## Phase 7

HTTP-Session-Client und Final-Transkript

## Phase 8

Overlay und partielle Updates

## Phase 9

Clipboard/Paste, Dictionary und Settings

## Phase 10

Polish, Fehlerpfade, Tests, Packaging

---

## 6. Phase 1: Repo- und Projektgrundgeruest

## 6.1 Ziel

Ein kompilierbares Grundgeruest fuer:

- Windows-Tauri-App
- WSL-Python-Backend
- gemeinsame Doku

## 6.2 Aufgaben

1. Lege `apps/windows-tray` an.
2. Lege `backend/wsl-qwen-asr` an.
3. Lege `docs/` an oder nutze die vorhandenen Dateien.
4. Erstelle eine Root-README mit:
   - Produktziel
   - Repo-Struktur
   - Setup-Ueberblick
5. Erstelle eine Backend-README mit:
   - WSL-Setup
   - manuellem Start
   - Troubleshooting

## 6.3 Definition of Done

- Repo-Struktur steht
- Tauri-App startet als leeres Fenster oder Tray-Shell
- Backend-Ordner ist vorbereitet

## 6.4 Test

- `apps/windows-tray` laesst sich installieren und starten
- `backend/wsl-qwen-asr` enthaelt bootstrap-Skripte und README

---

## 7. Phase 2: WSL-Backend bootstrap und Modellstart

## 7.1 Ziel

Das Backend soll in WSL lokal starten und `healthz` beantworten.

Noch ohne Windows-App.

## 7.2 Aufgaben

1. Implementiere `scripts/check_env.sh`.
2. Implementiere `scripts/bootstrap.sh`.
3. Lege WSL-Runtime-Pfade fest:
   - Venv: `~/.local/share/pibo-local-asr-tray/venv`
   - HF cache: `~/.cache/pibo-local-asr-tray/hf`
   - log dir: `~/.local/state/pibo-local-asr-tray`
4. Erstelle `requirements.txt` mit:
   - `fastapi`
   - `uvicorn[standard]`
   - `numpy`
   - `qwen-asr[vllm]`
5. Implementiere `app/settings.py` fuer env/config loading.
6. Implementiere `app/asr_service.py`:
   - Modell lazy laden
   - session map
   - `start_session`
   - `push_chunk`
   - `finish_session`
   - `cancel_session`
   - TTL cleanup
7. Implementiere `app/server.py` mit:
   - `GET /healthz`
   - `POST /api/start`
   - `POST /api/chunk`
   - `POST /api/finish`
   - `POST /api/cancel`
8. Implementiere `scripts/run_server.sh`.

## 7.2.1 Empfohlene Dateien fuer das Backend

```text
backend/wsl-qwen-asr/
  app/
    server.py
    asr_service.py
    settings.py
    schemas.py
  scripts/
    bootstrap.sh
    run_server.sh
    check_env.sh
  requirements.txt
  README.md
```

## 7.2.2 Empfohlene Startumgebung in `run_server.sh`

Das Script soll mindestens diese Variablen setzen:

```bash
export HF_HOME="${HF_HOME:-$HOME/.cache/pibo-local-asr-tray/hf}"
export VLLM_USE_V1="${VLLM_USE_V1:-1}"
export PIBO_ASR_HOST="${PIBO_ASR_HOST:-127.0.0.1}"
export PIBO_ASR_PORT="${PIBO_ASR_PORT:-8765}"
export PIBO_ASR_MODEL="${PIBO_ASR_MODEL:-Qwen/Qwen3-ASR-1.7B}"
export PIBO_ASR_GPU_MEMORY_UTILIZATION="${PIBO_ASR_GPU_MEMORY_UTILIZATION:-0.82}"
export PIBO_ASR_CHUNK_SIZE_SEC="${PIBO_ASR_CHUNK_SIZE_SEC:-0.5}"
export PIBO_ASR_UNFIXED_CHUNK_NUM="${PIBO_ASR_UNFIXED_CHUNK_NUM:-4}"
export PIBO_ASR_UNFIXED_TOKEN_NUM="${PIBO_ASR_UNFIXED_TOKEN_NUM:-5}"
```

## 7.3 Wichtige Implementierungsregeln

- bind host nur `127.0.0.1`
- Sessions im RAM
- rohes `float32` Audio akzeptieren
- JSON bei Fehlern

## 7.4 Definition of Done

- `bootstrap.sh` installiert das Backend sauber
- `run_server.sh` startet den Server
- `curl http://127.0.0.1:8765/healthz` antwortet

## 7.5 Test

1. Starte Backend manuell in WSL.
2. Rufe `healthz` auf.
3. Fuehre einen Minimaltest fuer `start -> cancel` aus.

---

## 8. Phase 3: Windows-Tauri-Tray-Shell

## 8.1 Ziel

Eine minimal lauffaehige Windows-App mit Tray-Icon und Settings-Fenster.

Noch ohne Audio.

## 8.2 Aufgaben

1. Scaffold `Tauri v2` App mit `React + TypeScript`.
2. Lege Hauptfenster klein an oder initial hidden.
3. Implementiere Tray-Icon.
4. Implementiere Tray-Menue:
   - Start Recording
   - Stop Recording
   - Cancel Recording
   - Open Settings
   - Start Backend
   - Stop Backend
   - Restart Backend
   - Quit
5. Implementiere App-State fuer:
   - backend status
   - recording status
   - current config
6. Lege Konfigurationsdatei unter `%AppData%` an und implementiere laden/speichern.

## 8.3 Definition of Done

- Tray-App startet stabil
- Settings-Fenster laesst sich oeffnen
- Konfiguration wird gelesen/geschrieben

## 8.4 Test

- App starten
- Tray-Menue bedienen
- Setting aendern und nach Neustart wiederfinden

---

## 9. Phase 4: Backend-Prozesssteuerung aus Windows

## 9.1 Ziel

Die Windows-App kann das WSL-Backend starten, stoppen und den Status pruefen.

## 9.2 Aufgaben

1. Implementiere Rust-Service `BackendManager`.
2. Definiere konfigurierbaren WSL-Distro-Namen.
3. Implementiere Startkommando via `wsl.exe`.
4. Das Startkommando soll in WSL `run_server.sh` aufrufen.
5. Leite stdout/stderr in Windows-App-Logs um oder speichere sie getrennt.
6. Implementiere periodischen `healthz`-Check.
7. Implementiere Stop-Logik:
   - bevorzugt Child-Prozess beenden
   - ansonsten sauberer WSL-stop Pfad
8. Zeige Backend-Status im UI an:
   - stopped
   - starting
   - ready
   - error

## 9.2.1 Empfohlene Verantwortung von `BackendManager`

`BackendManager` soll:

- den Startbefehl zusammensetzen
- den Child-Prozess halten
- stdout/stderr in Logs schreiben
- Health pollen
- Start-Timeout erkennen
- Stop mit Graceful-First-Strategie ausfuehren
- dem Frontend Status-Events liefern

## 9.3 Definition of Done

- Klick auf `Start Backend` startet real den WSL-Service
- `Backend Status` wird korrekt aktualisiert
- Klick auf `Stop Backend` stoppt den Prozess

## 9.4 Test

- Start
- Restart
- Stop
- App-Neustart mit bereits laufendem Backend

---

## 10. Phase 5: Globaler Hotkey und Session-State

## 10.1 Ziel

Die App reagiert auf einen globalen Hotkey und modelliert den Diktier-Lebenszyklus.

## 10.2 Aufgaben

1. Registriere globalen Shortcut.
2. Implementiere State Machine:
   - idle
   - backend_starting
   - recording
   - finalizing
   - error
3. Definiere Aktionen:
   - hotkey_down
   - hotkey_up
   - start_recording
   - stop_recording
   - cancel_recording
4. Implementiere Hotkey-Rebinding im Settings-UI.
5. Implementiere `Esc`-Cancel waehrend Recording, falls global sinnvoll umsetzbar.

## 10.3 Definition of Done

- Hotkey kann gesetzt werden
- hotkey down startet noch keine echte Aufnahme, aber den State
- hotkey up beendet den State sauber

## 10.4 Test

- Shortcut registrieren
- Shortcut wechseln
- Shortcut nach Neustart wiederherstellen

---

## 11. Phase 6: Windows-Audio-Capture und Resampling

## 11.1 Ziel

Die App nimmt Mikrofon-Audio auf und bringt es in das Zielformat fuer das Backend.

## 11.2 Aufgaben

1. Liste Input-Devices per `cpal`.
2. Implementiere Device-Auswahl und Fallback auf Default.
3. Implementiere Capture-Stream.
4. Normalisiere auf mono.
5. Resample auf `16kHz`.
6. Konvertiere auf `float32`.
7. Puffer-Management fuer `200ms` Chunks.
8. Lege internen Debug-Path an, um bei Bedarf Rohdaten mitzuschreiben.

## 11.2.1 Empfohlene Audio-Konvertierungskette

Der Capture-Pfad soll logisch so aufgebaut sein:

```text
input device
-> native sample format
-> normalize to f32
-> downmix to mono
-> resample to 16kHz
-> collect 200ms frames
-> send queue
```

## 11.3 Wichtige Regeln

- keine Blockierung des UI-Threads
- Audio-Thread nicht mit Netzwerkarbeit vermischen
- Chunk-Puffer ueber Channel/Queue an Sender uebergeben

## 11.4 Definition of Done

- Aufnahme laeuft stabil
- Chunk-Pipeline erzeugt `float32 mono 16kHz`
- Device-Wechsel ist moeglich

## 11.5 Test

- Default-Mikrofon
- externes Mikrofon
- Device fehlt nach Replug

---

## 12. Phase 7: HTTP-Session-Client und Final-Transkript

## 12.1 Ziel

Die App kann eine echte Session mit dem Backend durchlaufen und finale Texte erhalten.

## 12.2 Aufgaben

1. Implementiere Rust-Client fuer:
   - `healthz`
   - `start`
   - `chunk`
   - `finish`
   - `cancel`
2. Beim `start` Dictionary-Context mitsenden.
3. Beim Recording Chunks periodisch schicken.
4. `finish` beim Hotkey release aufrufen.
5. Finalen Text im App-State speichern.

## 12.2.1 Konkrete API-Requests fuer den ersten manuellen Test

Manueller Start:

```bash
curl -X POST http://127.0.0.1:8765/api/start \
  -H 'Content-Type: application/json' \
  -d '{"context":"Pibo\nOpenClaw\nPascal","language":"de"}'
```

Erwartung:

- JSON mit `session_id`

Manueller Cancel:

```bash
curl -X POST "http://127.0.0.1:8765/api/cancel?session_id=<SESSION_ID>"
```

## 12.3 Definition of Done

- Es gibt einen echten End-to-End-Fluss:
  - Hotkey down
  - Audio rein
  - Hotkey up
  - finaler Text kommt zurueck

## 12.4 Test

- kurzer deutscher Satz
- leerer Satz
- Cancel statt Finish

---

## 13. Phase 8: Overlay und partielle Updates

## 13.1 Ziel

Partielle Transkripte sollen sichtbar sein.

## 13.2 Aufgaben

1. Implementiere Overlay-Fenster als separates Tauri-Window.
2. Snapshotte Cursorposition beim Aufnahmebeginn.
3. Platziere Overlay nahe dieser Position.
4. Aktualisiere Overlay mit dem letzten Partial aus `/api/chunk`.
5. Zeige Status:
   - recording
   - finalizing
   - error
6. Blende Overlay bei Finish/Cancel aus.

## 13.3 Definition of Done

- Overlay erscheint
- Overlay zeigt Partial-Text
- Overlay verschwindet sauber

## 13.4 Test

- Start in Browser
- Start in Editor
- Start auf mehreren Monitoren

---

## 14. Phase 9: Clipboard/Paste, Dictionary und Settings

## 14.1 Ziel

Der Nutzer bekommt den finalen Text dort hin, wo er ihn braucht.

## 14.2 Aufgaben

1. Implementiere Clipboard-Lesen/Schreiben.
2. Implementiere Paste ueber `Ctrl+V` / Windows-Input.
3. Implementiere optionales Clipboard-Restore.
4. Implementiere `last transcript`.
5. Implementiere Dictionary-Editor im Settings-Fenster.
6. Dictionary als Zeilenliste speichern.
7. Vor jedem `start` `context` aus Dictionary rendern.

## 14.2.1 Konkrete Paste-Fallback-Regel

Wenn `Ctrl+V` scheitert oder unklar ist, ob es erfolgreich war:

1. finalen Text im Clipboard belassen
2. `last transcript` setzen
3. Fehlerstatus im UI anzeigen

Nicht:

- den Text wegwerfen
- sofort altes Clipboard wiederherstellen

## 14.3 Wichtige Paste-Regeln

- nur finals pasten
- bei leerem Final nichts einfuegen
- bei Paste-Fehler Nutzer informieren
- finalen Text als Fallback verfuergbar lassen

## 14.4 Definition of Done

- Finaler Text wird in Notepad/VS Code/Browser eingefuegt
- Dictionary wirkt sich auf Session-Start aus

## 14.5 Test

- Auto-Paste an
- Auto-Paste aus
- Restore-Clipboard an
- Dictionary mit Eigennamen

---

## 15. Phase 10: Polish, Fehlerpfade, Tests, Packaging

## 15.1 Ziel

Das Produkt ist benutzbar und nicht nur technisch moeglich.

## 15.2 Aufgaben

1. Fehlertexte im UI sauber formulieren.
2. Logs oeffnbar machen.
3. Cold-start-Hinweis fuer Modell-Laden zeigen.
4. Robustheit gegen mehrfaches Starten/Stoppen pruefen.
5. Konfiguration validieren.
6. Packaging fuer Windows testen.
7. README fuer Endnutzer und Entwickler trennen.

## 15.3 Definition of Done

- App ist fuer V1 lokal installierbar
- offensichtliche Fehlerpfade sind behandelt
- Kernszenario funktioniert mehrfach hintereinander

---

## 16. Konkrete Backend-Implementierungsdetails

## 16.1 `check_env.sh`

Soll pruefen:

- Python vorhanden
- richtige Python-Version
- NVIDIA/CUDA in WSL prinzipiell sichtbar
- Schreibrechte fuer Cache-/State-Verzeichnisse

Es soll **keine** tiefen magischen Reparaturen ausfuehren.
Nur pruefen und klare Meldungen ausgeben.

## 16.2 `bootstrap.sh`

Soll:

- Runtime-Verzeichnisse anlegen
- Venv anlegen
- pip upgraden
- requirements installieren

## 16.3 `run_server.sh`

Soll:

- Runtime-Env exportieren
- Port setzen
- FastAPI/Uvicorn starten
- Backend auf `127.0.0.1` binden

---

## 17. Konkrete Windows-Implementierungsdetails

## 17.1 Rust-Module

Empfohlene Trennung in `src-tauri/src/`:

```text
app_state.rs
backend_manager.rs
config.rs
audio/
  capture.rs
  resample.rs
dictation/
  session_controller.rs
  backend_client.rs
  paste.rs
overlay/
  window.rs
hotkey.rs
logging.rs
main.rs
```

## 17.2 Frontend-Komponenten

Empfohlene Trennung in `src/`:

```text
App.tsx
pages/SettingsPage.tsx
components/BackendStatusCard.tsx
components/HotkeyField.tsx
components/DeviceSelect.tsx
components/DictionaryEditor.tsx
components/ToggleField.tsx
lib/api.ts
lib/state.ts
```

---

## 18. Explizite Reihenfolge der ersten realen Coding-Schritte

Wenn ein Agent jetzt sofort anfangen soll, dann in genau dieser Reihenfolge:

1. Repo-Struktur anlegen.
2. WSL-Backend skeleton bauen.
3. `healthz` lauffaehig machen.
4. Modell-Ladepfad lauffaehig machen.
5. Manuelles `start/chunk/finish` gegen Backend testen.
6. Tauri-Tray-Shell bauen.
7. Backend aus der App starten.
8. Hotkey-State bauen.
9. Audio-Capture bauen.
10. End-to-End Final-Transkript bauen.
11. Overlay mit Partials bauen.
12. Paste und Dictionary fertigziehen.

Nicht zuerst:

- visuelles Design
- Packaging
- Autostart
- extra Komfortfunktionen

---

## 19. Debugging-Reihenfolge bei Problemen

Wenn etwas nicht funktioniert, in dieser Reihenfolge debuggen:

1. Laeuft WSL ueberhaupt?
2. Ist `healthz` erreichbar?
3. Ist das Modell geladen?
4. Kommen Chunks im Backend an?
5. Kommen partielle Texte zurueck?
6. Wird `finish` korrekt aufgerufen?
7. Scheitert nur Paste?
8. Oder scheitert schon Audio-Capture?

Nicht sofort im UI suchen, wenn das Backend schon tot ist.

## 19.1 Schnellpruefung fuer den implementierenden Agenten

Wenn nur 5 Minuten Zeit fuer eine Erstdiagnose da sind:

1. `wsl.exe --status`
2. `wsl.exe -d Ubuntu nvidia-smi`
3. Backend manuell per `run_server.sh`
4. `curl /healthz`
5. erst dann Tauri-App starten

---

## 20. Risiko- und Entscheidungsnotizen

## 20.1 Warum nicht Browser-Frontend

Weil das Produkt eine permanente Desktop-Funktion ist:

- globaler Hotkey
- Texteinfuegen in beliebige Apps
- Tray-Verhalten

Das ist als Webapp die falsche Form.

## 20.2 Warum nicht Mikrofon in WSL

Weil Windows fuer:

- Device-Auswahl
- Hotkeys
- Desktop-UX
- Paste in aktive App

der natuerliche Ort ist.

## 20.3 Warum keine Live-Partial-Injektion in Fremdprogramme

Weil das in V1 die Fehlerflaeche massiv vergroessert.

Partials im Overlay sind produktiv genug.
Finals in die aktive App sind der robuste Kern.

---

## 21. V1-Fertigkeitscheckliste

Vor Abschluss muessen alle Punkte abgehakt sein:

- Backend startet in WSL
- Modell laedt lokal
- Tray-App startet
- globaler Hotkey funktioniert
- Mikrofon-Auswahl funktioniert
- Recording startet/stopt
- Partial-Overlay funktioniert
- finaler Text kommt zurueck
- finaler Text wird eingefuegt
- Dictionary ist editierbar
- Quit hinterlaesst keine Zombie-Prozesse

---

## 22. Handoff-Anweisung fuer den implementierenden Agenten

Arbeite **nicht** so, als gaebe es verstecktes Vorwissen.

Nimm diese beiden Dateien als Source of Truth:

- `docs/spec.md`
- `docs/implementation-plan.md`

Wenn du Entscheidungen treffen musst, priorisiere:

1. robuste lokale Benutzbarkeit
2. klares Prozessmodell
3. minimalen V1-Scope
4. spaetere Erweiterbarkeit
