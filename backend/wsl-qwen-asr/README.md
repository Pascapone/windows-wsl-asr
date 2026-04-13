# WSL Qwen ASR Backend

Lokales FastAPI-Backend für Streaming-ASR in WSL.

## Zuständigkeit

Das Backend übernimmt:

- Modellstart
- Session-Lifecycle
- `start/chunk/finish/cancel`
- Health-Status
- Logging

## Laufzeitpfade

- Venv: `~/.local/share/pibo-local-asr-tray/venv`
- HuggingFace-Cache: `~/.cache/pibo-local-asr-tray/hf`
- State/Logs: `~/.local/state/pibo-local-asr-tray`
- Logdatei: `~/.local/state/pibo-local-asr-tray/backend.log`

## Voraussetzungen

- WSL2
- Ubuntu
- Python 3.12
- NVIDIA/CUDA in WSL sichtbar

Kurztest:

```bash
python3 --version
nvidia-smi
```

## Bootstrap

```bash
cd /mnt/c/Users/pasca/Coding/asr-app/backend/wsl-qwen-asr
bash scripts/check_env.sh
bash scripts/bootstrap.sh
```

## Start

```bash
cd /mnt/c/Users/pasca/Coding/asr-app/backend/wsl-qwen-asr
bash scripts/run_server.sh
```

Health prüfen:

```bash
curl http://127.0.0.1:8765/healthz
```

## Wichtige Runtime-Parameter

Die Defaults werden über `scripts/run_server.sh` gesetzt, unter anderem:

- `PIBO_ASR_HOST=127.0.0.1`
- `PIBO_ASR_PORT=8765`
- `PIBO_ASR_MODEL=Qwen/Qwen3-ASR-1.7B`
- `PIBO_ASR_GPU_MEMORY_UTILIZATION=0.85`
- `PIBO_ASR_CHUNK_SIZE_SEC=0.5`
- `PIBO_ASR_MAX_MODEL_LEN=2048`
- `PIBO_ASR_MAX_NUM_BATCHED_TOKENS=256`
- `PIBO_ASR_MAX_NUM_SEQS=1`

## Betriebshinweis

Qwen-ASR verarbeitet im Streaming-Modus akkumuliertes Audio wiederholt. Lange Sessions werden daher mit der Zeit teurer. Der Windows-Client rollt Sessions inzwischen automatisch in Segmente, um Hänger zu vermeiden.

## Troubleshooting

- `healthz` hängt oder bleibt auf `model_loaded=false`:
  Log in `backend.log` prüfen, Modellstart abwarten und freien VRAM kontrollieren.
- `address already in use`:
  Es läuft bereits ein Backend auf Port `8765`.
- `nvidia-smi` geht in WSL nicht:
  Erst WSL-/Treiber-Setup reparieren, nicht am App-Code suchen.
- Python-/venv-Probleme:
  `bash scripts/bootstrap.sh` erneut ausführen.
