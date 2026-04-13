#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="${PIBO_ASR_VENV_DIR:-$HOME/.local/share/pibo-local-asr-tray/venv}"
STATE_DIR="${PIBO_ASR_STATE_DIR:-$HOME/.local/state/pibo-local-asr-tray}"

export HF_HOME="${HF_HOME:-$HOME/.cache/pibo-local-asr-tray/hf}"
export VLLM_USE_V1="${VLLM_USE_V1:-1}"
export PIBO_ASR_HOST="${PIBO_ASR_HOST:-127.0.0.1}"
export PIBO_ASR_PORT="${PIBO_ASR_PORT:-8765}"
export PIBO_ASR_MODEL="${PIBO_ASR_MODEL:-Qwen/Qwen3-ASR-1.7B}"
export PIBO_ASR_GPU_MEMORY_UTILIZATION="${PIBO_ASR_GPU_MEMORY_UTILIZATION:-0.85}"
export PIBO_ASR_CPU_OFFLOAD_GB="${PIBO_ASR_CPU_OFFLOAD_GB:-0}"
export PIBO_ASR_CHUNK_SIZE_SEC="${PIBO_ASR_CHUNK_SIZE_SEC:-0.5}"
export PIBO_ASR_UNFIXED_CHUNK_NUM="${PIBO_ASR_UNFIXED_CHUNK_NUM:-4}"
export PIBO_ASR_UNFIXED_TOKEN_NUM="${PIBO_ASR_UNFIXED_TOKEN_NUM:-5}"
export PIBO_ASR_MAX_NEW_TOKENS="${PIBO_ASR_MAX_NEW_TOKENS:-32}"
export PIBO_ASR_MAX_MODEL_LEN="${PIBO_ASR_MAX_MODEL_LEN:-2048}"
export PIBO_ASR_MAX_NUM_BATCHED_TOKENS="${PIBO_ASR_MAX_NUM_BATCHED_TOKENS:-256}"
export PIBO_ASR_MAX_NUM_SEQS="${PIBO_ASR_MAX_NUM_SEQS:-1}"
export PIBO_ASR_KV_CACHE_MEMORY_BYTES="${PIBO_ASR_KV_CACHE_MEMORY_BYTES:-}"
export PIBO_ASR_ENFORCE_EAGER="${PIBO_ASR_ENFORCE_EAGER:-0}"
export PIBO_ASR_MAX_INFERENCE_BATCH_SIZE="${PIBO_ASR_MAX_INFERENCE_BATCH_SIZE:-1}"
export PIBO_ASR_STATE_DIR="$STATE_DIR"
export PIBO_ASR_LOG_FILE="${PIBO_ASR_LOG_FILE:-$STATE_DIR/backend.log}"
export CC="${CC:-/usr/bin/gcc}"
export CXX="${CXX:-/usr/bin/g++}"

mkdir -p "$STATE_DIR" "$HF_HOME"

# vLLM spawns worker processes and flushes stdio during startup. In non-interactive
# WSL launches, redirecting to a stable file avoids BrokenPipeError on closed stdout.
if [[ ! -t 1 ]]; then
  exec >>"$PIBO_ASR_LOG_FILE" 2>&1
fi

source "$VENV_DIR/bin/activate"

cd "$ROOT_DIR"
exec python -m uvicorn app.server:app --host "$PIBO_ASR_HOST" --port "$PIBO_ASR_PORT"
