#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="${PIBO_ASR_VENV_DIR:-$HOME/.local/share/pibo-local-asr-tray/venv}"
STATE_DIR="${PIBO_ASR_STATE_DIR:-$HOME/.local/state/pibo-local-asr-tray}"
HF_HOME="${HF_HOME:-$HOME/.cache/pibo-local-asr-tray/hf}"

bash "$ROOT_DIR/scripts/check_env.sh"

mkdir -p "$STATE_DIR" "$HF_HOME" "$(dirname "$VENV_DIR")"

if [[ -d "$VENV_DIR" && ! -f "$VENV_DIR/bin/activate" ]]; then
  rm -rf "$VENV_DIR"
fi

if [[ ! -d "$VENV_DIR" ]]; then
  python3 -m venv "$VENV_DIR"
fi

source "$VENV_DIR/bin/activate"
python -m pip install --upgrade pip setuptools wheel
python -m pip install -r "$ROOT_DIR/requirements.txt"

echo "[bootstrap] complete"
echo "[bootstrap] venv: $VENV_DIR"
