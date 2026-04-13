#!/usr/bin/env bash
set -euo pipefail

EXPECTED_MAJOR=3
EXPECTED_MINOR=12
STATE_DIR="${PIBO_ASR_STATE_DIR:-$HOME/.local/state/pibo-local-asr-tray}"
HF_HOME="${HF_HOME:-$HOME/.cache/pibo-local-asr-tray/hf}"
VENV_DIR="${PIBO_ASR_VENV_DIR:-$HOME/.local/share/pibo-local-asr-tray/venv}"

echo "[check] probing Python"
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found in WSL" >&2
  exit 1
fi

PYTHON_VERSION="$(python3 - <<'PY'
import sys
print(f"{sys.version_info.major}.{sys.version_info.minor}")
PY
)"

if [[ "$PYTHON_VERSION" != "${EXPECTED_MAJOR}.${EXPECTED_MINOR}" ]]; then
  echo "expected Python ${EXPECTED_MAJOR}.${EXPECTED_MINOR}, got ${PYTHON_VERSION}" >&2
  exit 1
fi

echo "[check] probing NVIDIA visibility"
if command -v nvidia-smi >/dev/null 2>&1; then
  nvidia-smi >/dev/null
else
  echo "nvidia-smi not found inside WSL" >&2
  exit 1
fi

echo "[check] ensuring runtime directories are writable"
mkdir -p "$STATE_DIR" "$HF_HOME" "$(dirname "$VENV_DIR")"
touch "$STATE_DIR/.write-test"
rm -f "$STATE_DIR/.write-test"

echo "[check] environment looks good"
