# Pibo Local ASR Tray Spec

Stand: 2026-04-12
Status: Handoff Spec

## 1. Zweck

Diese Spezifikation definiert ein **neues** lokales Diktierprodukt fuer einen Windows-PC mit NVIDIA-GPU und WSL2.

Das Produkt ersetzt fuer den Heimgebrauch eine kostenpflichtige Cloud-/Modal-Instanz durch eine lokale Laufzeit:

- Windows-App fuer UX, Hotkey, Mikrofon, Overlay und Text-Einfuegen
- WSL2-Linux-Backend fuer `vLLM + Qwen3-ASR`

Diese Spec ist absichtlich **selbsttragend**.

Der implementierende Agent darf **nicht** voraussetzen:

- Zugriff auf fruehere Modal-/ASR-Projekte
- Zugriff auf dieses Workspace-Umfeld
- Zugriff auf historische Referenzimplementierungen

Alles Noetige fuer Architektur, Verhalten, Vertrage und Scope steht in diesem Dokument und im begleitenden Implementierungsplan.

---

## 2. Verbindliche Produktentscheidungen

### 2.1 Produktform

Das Produkt ist eine **Windows-Tray-App** mit kleinem Settings-Fenster und kleinem Streaming-Overlay.

Keine Haupt-Webapp.
Kein Browser als primaere Bedienoberflaeche.
Kein grosses Dashboard.

### 2.2 Architektur

Die Architektur ist verbindlich:

```text
Windows Tray App
-> globaler Hotkey
-> Mikrofon-Capture unter Windows
-> Audio-Streaming an localhost
-> WSL2 Ubuntu Backend
-> qwen-asr[vllm]
-> Qwen/Qwen3-ASR-1.7B
-> NVIDIA GPU
```

### 2.3 Technologiewahl

Die empfohlene und hier spezifizierte technische Richtung ist:

- Windows-App: `Tauri v2`, Frontend `React + TypeScript`, Native Layer `Rust`
- Windows-Audio: Rust-Audio-Capture ueber `cpal`
- Audio-Resampling: Rust-Resampler, empfohlen `rubato`
- Clipboard: Rust, empfohlen `arboard`
- Tastatur-/Paste-Simulation: Windows-API oder Rust-Library mit `SendInput`-Pfad
- WSL-Backend: `Python 3.12`
- Python-API-Server: `FastAPI + Uvicorn`
- ASR-Stack: `qwen-asr[vllm]`
- Modell: `Qwen/Qwen3-ASR-1.7B`

### 2.4 Lokaler Kommunikationspfad

Die Windows-App spricht das WSL-Backend ueber `localhost`.

Keine oeffentliche Exponierung.
Kein LAN-Serving.
Keine Remote-Auth fuer V1.

### 2.5 Aufnahme-Modell

V1 verwendet standardmaessig **Push-to-talk**.

Verbindliches Verhalten:

- Hotkey down -> Aufnahme und Streaming starten
- Hotkey up -> Session finalisieren
- finalen Text erhalten
- finalen Text optional automatisch in aktive App einfuegen

### 2.6 Streaming-UX

Verbindlich:

- waehrend Aufnahme werden **partielle Transkripte** angezeigt
- partielle Transkripte gehen **nur ins Overlay**
- finale Transkripte gehen **in die aktive/fokussierte Texteingabe**

Nicht Bestandteil von V1:

- partielle Texte direkt in Fremd-Apps live ersetzen

### 2.7 Dictionary

V1 muss ein lokales **Context-Dictionary** haben.

Es dient als leichtes Steering fuer Namen, Fachbegriffe und bevorzugte Schreibweisen.

Technisch wird das Dictionary pro Session als `context`-Text an das Backend uebergeben.

Kein hartes Decoder-Biasing in V1.

---

## 3. Produktziel

Ein Nutzer soll:

1. die Tray-App starten,
2. einen Hotkey gedrueckt halten,
3. sprechen,
4. live ein partielles Transkript in einem Overlay sehen,
5. beim Loslassen den finalen Text lokal berechnen lassen,
6. den finalen Text automatisch in die aktive App eingefuegt bekommen.

Zusatzlich soll der Nutzer:

- das Mikrofon auswaehlen koennen
- den Hotkey aendern koennen
- das Dictionary bearbeiten koennen
- das lokale Backend starten/stoppen koennen
- Auto-Paste ein-/ausschalten koennen

---

## 4. Harte Rahmenbedingungen

### 4.1 Zielplattform

Primäre Zielplattform:

- Windows 11
- NVIDIA-GPU
- WSL2 mit Ubuntu

V1 muss **nicht** fuer macOS oder Linux Desktop gebaut werden.

### 4.2 Hardwareannahmen

Minimalannahmen:

- NVIDIA-GPU mit WSL-faehigem CUDA-Treiber
- funktionsfaehiges Mikrofon
- genug VRAM fuer `Qwen/Qwen3-ASR-1.7B`

### 4.3 Netzwerk

V1 ist **rein lokal**.

Es muss ohne Internet benutzbar sein, **nachdem** Modellgewichte und Python-Abhaengigkeiten bereits installiert sind.

### 4.4 Sicherheitsmodell

Da der Dienst nur lokal auf `localhost` laeuft, ist fuer V1 keine Benutzer-Auth erforderlich.

Verbindlich:

- Backend bindet nur auf `127.0.0.1`
- keine Port-Freigaben im LAN
- keine Cloud-Abhaengigkeit im Normalbetrieb

---

## 5. Nicht-Ziele fuer V1

Folgende Dinge gehoeren **nicht** zu V1:

- Browser-App als Hauptoberflaeche
- Mehrbenutzerbetrieb
- Remote-Zugriff
- Account-System
- Cloud-Sync
- mobile Clients
- automatisches Kontextmanagement ueber lange Sitzungen
- Sprechertrennung
- Audio-Datei-Upload-Workflow
- partielle Live-Injektion direkt in beliebige Fremd-Apps
- komplexe VAD-Pipeline
- Modellumschaltung in V1

---

## 6. Repo-Struktur

Der Repo-Root heisst:

```text
pibo-local-asr-tray
```

Erwartete Struktur:

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
        models.py
        settings.py
      scripts/
        bootstrap.sh
        run_server.sh
        check_env.sh
      requirements.txt
      README.md
  shared/
    contracts/
      backend-api.md
```

Die exakte Dateiverteilung darf leicht abweichen, solange diese Funktionsbloecke klar vorhanden sind.

---

## 7. Laufzeit-Topologie

### 7.1 Windows-Seite

Die Windows-App ist Orchestrator fuer:

- Tray-Icon
- globalen Hotkey
- Settings
- Overlay
- Audio-Capture
- Audio-Resampling
- Session-Steuerung
- Kommunikation mit dem Backend
- finale Texteinfuegung
- Verwaltung des WSL-Backend-Prozesses

### 7.2 WSL-Seite

Das WSL-Backend ist zustaendig fuer:

- Modellinitialisierung
- Session-Lebenszyklus
- inkrementelle Streaming-Transkription
- finalen Flush einer Session
- lokale Health- und Status-Endpunkte

---

## 8. UX-Komponenten

## 8.1 Tray-Menue

Das Tray-Menue muss mindestens folgende Eintraege besitzen:

- `Start Recording`
- `Stop Recording`
- `Cancel Recording`
- `Open Settings`
- `Start Backend`
- `Stop Backend`
- `Restart Backend`
- `Backend Status`
- `Quit`

Optional, aber sinnvoll:

- `Open Logs`
- `Paste Last Transcript`

### 8.1.1 Verhalten

`Start Recording`

- nur aktiv, wenn Backend ready ist
- startet dieselbe Logik wie Hotkey down

`Stop Recording`

- finalisiert die laufende Session

`Cancel Recording`

- verwirft die laufende Session ohne Einfuegen

`Quit`

- beendet aktive Aufnahme
- stoppt Child-Prozesse der App
- beendet die Tray-App sauber

## 8.2 Settings-Fenster

Das Settings-Fenster ist klein und funktional.

Keine Marketing-UI.
Keine komplexe Navigation.

Sektionen:

### 8.2.1 General

- `Launch backend automatically`
- `Start app on login` optional, darf in V1 noch deaktiviert bleiben

### 8.2.2 Input

- Auswahl Audio-Input-Device
- Anzeige des aktuellen Devices
- Refresh der Device-Liste

### 8.2.3 Hotkey

- globaler Hotkey
- Anzeige des aktuellen Shortcuts
- Shortcut neu aufzeichnen

### 8.2.4 Backend

- WSL-Distro-Name
- Port
- Modellname
- `gpu_memory_utilization`
- `chunk_size_sec`
- Backend-Status
- Button fuer Start/Stop/Restart

### 8.2.5 Dictation

- `Auto paste final transcript`
- `Restore clipboard after paste`
- `Show overlay while recording`
- `Overlay follows mouse anchor at recording start`

### 8.2.6 Dictionary

- grosses mehrzeiliges Textfeld
- ein Begriff pro Zeile
- leere Zeilen werden ignoriert

## 8.3 Streaming-Overlay

Das Overlay ist ein kleines, nicht dominantes Fenster.

Verbindliche Eigenschaften:

- always-on-top
- ohne Taskbar-Eintrag
- nicht modal
- standardmaessig nicht fokussierend
- nahe der Mausposition zum Zeitpunkt von `record start`
- auf Bildschirmgrenzen gecampt
- zeigt:
  - Aufnahmezustand
  - partiellen Text
  - optional kleinen Status wie `recording`, `finalizing`, `error`

### 8.3.1 Overlay-Platzierung

Standard:

- beim Start der Aufnahme aktuelle Cursorposition snapshotten
- Overlay rechts unten mit kleinem Offset platzieren
- wenn der Platz nicht reicht, intelligent an sichtbaren Displaybereich anpassen

### 8.3.2 Overlay-Inhalt

Minimal:

- Statuszeile
- partieller Text

Keine Buttons im Overlay in V1 erforderlich.

---

## 9. Aufnahme- und Session-Flow

## 9.1 Erfolgsfall: Push-to-talk

1. Nutzer drueckt Hotkey.
2. App prueft Backend-Readiness.
3. App snapshotet Cursorposition.
4. Overlay erscheint.
5. App startet eine neue Backend-Session mit Dictionary-Context.
6. App startet Mikrofon-Capture.
7. App konvertiert Audio in `mono float32 16 kHz`.
8. App schickt periodisch Chunks ans Backend.
9. Backend gibt partiellen Text zurueck.
10. Overlay aktualisiert sich mit dem neuesten Partial.
11. Nutzer laesst Hotkey los.
12. App beendet Capture und ruft `finish` auf.
13. Backend liefert finalen Text.
14. App blendet Overlay aus.
15. Wenn `autoPaste = true` und Text nicht leer:
    - Clipboard sichern
    - finalen Text ins Clipboard setzen
    - Paste an aktive App senden
    - Clipboard optional wiederherstellen
16. Letztes Final wird in App-State gespeichert.

## 9.2 Cancel-Fall

1. Nutzer drueckt waehrend Aufnahme `Esc` oder waehlt `Cancel Recording`.
2. App stoppt Capture.
3. App ruft `cancel` am Backend auf oder verwirft lokale Session.
4. Overlay verschwindet.
5. Es wird **kein** Text eingefuegt.

## 9.3 Backend-nicht-bereit-Fall

1. Nutzer startet Aufnahme.
2. Backend ist noch nicht bereit.
3. App versucht Backend automatisch zu starten, falls aktiviert.
4. Overlay zeigt `backend starting`.
5. Wenn Backend innerhalb eines definierten Fensters bereit wird, startet Aufnahme.
6. Wenn nicht, bekommt der Nutzer klaren Fehlerstatus.

---

## 10. Audio-Pipeline

## 10.1 Capture

Audio wird auf Windows aufgenommen, nicht in WSL.

Der Audio-Capture-Layer muss:

- die verfuegbaren Input-Devices auflisten
- das konfigurierte Device waehlen
- auf Default-Device zurueckfallen koennen
- Capture in einem stabilen Hintergrundpfad liefern

## 10.2 Zielformat

Verbindliches internes Zielformat vor Versand ans Backend:

- mono
- `float32`
- `16_000 Hz`
- little-endian beim binaren Versand

## 10.3 Resampling

Da Eingangsgeraete oft `44.1 kHz` oder `48 kHz` liefern, ist Resampling auf `16 kHz` verpflichtend.

## 10.4 Chunking

V1 soll mit kleinen Chunks arbeiten.

Empfohlene Defaults:

- Client-Sendeintervall: `200 ms`
- Backend `chunk_size_sec`: `0.5`

Begruendung:

- spuerbar niedrigere Latenz als 1-Sekunden-Defaults
- gleichzeitig robust genug fuer V1

## 10.5 VAD

V1 benoetigt **keine** komplexe Voice Activity Detection.

Begruendung:

- Push-to-talk begrenzt Stille bereits stark
- VAD fuegt fuer V1 unnoetige Komplexitaet hinzu

---

## 11. Backend-API

Das Backend ist ein lokaler HTTP-Server.

Basis-URL:

```text
http://127.0.0.1:8765
```

Port konfigurierbar.

## 11.1 `GET /healthz`

Antwort:

```json
{
  "ok": true,
  "model_loaded": true,
  "model_name": "Qwen/Qwen3-ASR-1.7B"
}
```

## 11.2 `POST /api/start`

Request:

```json
{
  "context": "OpenClaw\nPibo\nPascal\nQwen",
  "language": "de",
  "session_meta": {
    "client": "windows-tray",
    "version": "0.1.0"
  }
}
```

`language` darf `null` sein.

Response:

```json
{
  "session_id": "uuid-string"
}
```

## 11.3 `POST /api/chunk?session_id=<id>`

Request:

- Content-Type: `application/octet-stream`
- Body: rohe `float32`-Samples, mono, `16kHz`

Response:

```json
{
  "language": "de",
  "text": "das ist der aktuelle partielle text"
}
```

## 11.4 `POST /api/finish?session_id=<id>`

Response:

```json
{
  "language": "de",
  "text": "das ist der finale text"
}
```

## 11.5 `POST /api/cancel?session_id=<id>`

Response:

```json
{
  "ok": true
}
```

## 11.6 Fehlervertrag

Bei Fehlern soll das Backend JSON liefern:

```json
{
  "error": "invalid_session"
}
```

oder

```json
{
  "error": "backend_starting"
}
```

oder

```json
{
  "error": "internal_error",
  "message": "human readable message"
}
```

---

## 12. Backend-Verhalten

## 12.1 Modellinitialisierung

Verbindliche Defaults:

- Modell: `Qwen/Qwen3-ASR-1.7B`
- `gpu_memory_utilization`: `0.82`
- `max_new_tokens`: `32`
- `chunk_size_sec`: `0.5`
- `unfixed_chunk_num`: `4`
- `unfixed_token_num`: `5`

## 12.2 Session-Lebensdauer

Sessions duerfen im Speicher gehalten werden.

Verbindlich:

- inaktive Sessions per TTL bereinigen
- empfohlene Session-TTL: `10 Minuten`

## 12.3 Bindung

Verbindlich:

- Bind Host: `127.0.0.1`
- kein `0.0.0.0`

## 12.4 Logs

Das Backend schreibt Logs nach:

- stdout fuer Entwicklung
- Datei fuer Debugging optional

Empfohlener Log-Speicherort in WSL:

```text
~/.local/state/pibo-local-asr-tray/backend.log
```

---

## 13. WSL-Prozessmodell

## 13.1 Besitz des Backend-Prozesses

Die Windows-Tray-App besitzt den WSL-Backend-Prozess.

Das heisst:

- App kann Backend starten
- App kann Backend stoppen
- App kann Backend restarten
- beim App-Quit wird ein von der App gestarteter Backend-Prozess sauber beendet

## 13.2 Manuelles Debugging

Es muss moeglich sein, das Backend auch manuell in WSL zu starten.

Dafuer braucht das Backend:

- `scripts/bootstrap.sh`
- `scripts/run_server.sh`
- `scripts/check_env.sh`

---

## 14. Windows-Text-Einfuegen

## 14.1 V1-Strategie

Finale Texte werden per **Clipboard + Paste** eingefuegt.

Nicht per komplexem UI-Automation-Diffing.

Standardablauf:

1. existierendes Clipboard lesen
2. finalen Text ins Clipboard setzen
3. `Ctrl+V` an aktive App senden
4. Clipboard optional wiederherstellen

## 14.2 Wichtige Regel

Partials werden nicht in Fremd-Apps injiziert.

Nur finals.

## 14.3 Fehlerverhalten

Wenn Paste fehlschlaegt:

- Transcript bleibt als `last transcript` verfuegbar
- App zeigt klaren Fehlerhinweis
- Clipboard-Wiederherstellung darf den finalen Text nicht zerstören, bevor der Nutzer ihn retten kann

Empfohlener Fallback:

- bei Paste-Fehler finalen Text im Clipboard belassen und Nutzer informieren

---

## 15. Konfiguration

## 15.1 Windows-Konfigurationsdatei

Speicherort:

```text
%AppData%\PiboLocalAsrTray\config.json
```

Beispiel:

```json
{
  "backend": {
    "wslDistro": "Ubuntu",
    "host": "127.0.0.1",
    "port": 8765,
    "modelName": "Qwen/Qwen3-ASR-1.7B",
    "gpuMemoryUtilization": 0.82,
    "chunkSizeSec": 0.5,
    "autoStartBackend": true
  },
  "capture": {
    "hotkey": "Ctrl+Shift+Space",
    "inputDeviceId": null
  },
  "overlay": {
    "enabled": true,
    "anchor": "mouse",
    "offsetX": 16,
    "offsetY": 20,
    "maxWidth": 520
  },
  "dictation": {
    "languageHint": "de",
    "autoPaste": true,
    "restoreClipboard": true
  },
  "dictionary": {
    "terms": [
      "Pibo",
      "OpenClaw",
      "Pascal"
    ]
  }
}
```

## 15.2 Dictionary-Rendering

Aus `dictionary.terms` wird beim Start einer Session ein `context`-String erzeugt:

```text
Pibo
OpenClaw
Pascal
```

Leere oder duplizierte Eintraege werden entfernt.

---

## 16. Statusmodell der App

Die App kennt mindestens diese Status:

- `idle`
- `backend_starting`
- `backend_ready`
- `recording`
- `finalizing`
- `error`

Diese Status muessen:

- im internen State modelliert sein
- im Settings-Fenster sichtbar sein
- fuer das Overlay nutzbar sein

---

## 17. Fehler- und Degradationsverhalten

## 17.1 WSL fehlt

Wenn `wsl.exe` nicht verfuegbar ist:

- App zeigt klaren Fehler
- Recording wird deaktiviert

## 17.2 Backend startet nicht

Wenn Backend-Start scheitert:

- klarer Fehlerstatus
- Link/Button zu Logs
- kein stilles Hängenbleiben

## 17.3 Modell nicht ladbar

Wenn Modell nicht geladen werden kann:

- Health zeigt `model_loaded = false`
- Settings zeigen Fehlermeldung
- Recording bleibt gesperrt

## 17.4 Mikrofon fehlt

Wenn kein Input-Device verfuegbar ist:

- Recording ist gesperrt
- Settings zeigen Fehler

---

## 18. Performance-Ziele

Zielwerte fuer V1 auf realistischer NVIDIA-Hardware:

- Warm backend start bis `healthz ok`: unter `20 s`
- Cold model load: darf deutlich laenger dauern
- partielles Update nach Sprachbeginn: subjektiv schnell, Ziel grob `unter 1 s`
- finaler Text nach Hotkey release: Ziel grob `unter 2 s`

Diese Werte sind Richtwerte, keine harte Echtzeitgarantie.

---

## 19. Logging

## 19.1 Windows-App

Empfohlener Log-Pfad:

```text
%LocalAppData%\PiboLocalAsrTray\logs\app.log
```

## 19.2 Backend

Empfohlener Log-Pfad in WSL:

```text
~/.local/state/pibo-local-asr-tray/backend.log
```

Logs sollen mindestens enthalten:

- Backend-Start/Stop
- Health-Checks
- Session-Start/Finish/Cancel
- Audio-Device-Auswahl
- Paste-Erfolg/Paste-Fehler
- Modellladefehler

---

## 20. Akzeptanzkriterien

V1 gilt als fertig, wenn alle folgenden Punkte erfuellt sind:

1. App startet unter Windows als Tray-App.
2. WSL-Backend kann aus der App heraus gestartet werden.
3. Backend laedt `Qwen/Qwen3-ASR-1.7B` lokal in WSL.
4. Nutzer kann ein Mikrofon auswaehlen.
5. Nutzer kann einen globalen Hotkey setzen.
6. Hotkey down startet lokale Aufnahme.
7. Waehren der Aufnahme erscheint ein Overlay.
8. Overlay zeigt partiellen Text.
9. Hotkey up finalisiert die Session.
10. Finaler Text wird automatisch in die aktive Windows-App eingefuegt.
11. Dictionary kann in den Settings bearbeitet werden.
12. Dictionary wird beim Start einer Session ans Backend uebergeben.
13. Nutzer kann eine Aufnahme abbrechen, ohne dass Text eingefuegt wird.
14. Nutzer kann die App beenden, ohne zombiehafte Prozesse zu hinterlassen.

---

## 21. Testmatrix

Mindestens diese Ziel-Apps auf Windows pruefen:

- Notepad
- VS Code
- Browser-Textarea
- Chat-App im Browser

Mindestens diese Szenarien pruefen:

- Backend cold start
- Backend warm start
- kurzer Satz
- langer Satz
- Abbruch waehrend Aufnahme
- Mikrofonwechsel
- Hotkey-Aenderung
- Dictionary mit Eigennamen

---

## 22. Offene Freiheiten fuer den implementierenden Agenten

Folgende Freiheiten sind erlaubt, solange die Produktentscheidungen oben eingehalten werden:

- genaue State-Management-Library im Tauri-Frontend
- genaue UI-Komponentenbibliothek oder Vanilla-UI
- genaue Rust-Implementierung fuer `SendInput`
- genaue Python-Dateistruktur im Backend

Nicht frei verhandelbar sind:

- Tray-App statt Browser-App
- Windows fuer UX und Mic
- WSL fuer vLLM/Qwen
- Push-to-talk als V1-Default
- Overlay fuer Partials
- Finals in aktive App
- Dictionary via `context`

