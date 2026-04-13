from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


def _env_str(name: str, default: str) -> str:
    value = os.getenv(name)
    if value is None or not value.strip():
        return default
    return value.strip()


def _env_float(name: str, default: float) -> float:
    value = os.getenv(name)
    if value is None or not value.strip():
        return default
    return float(value)


def _env_int(name: str, default: int) -> int:
    value = os.getenv(name)
    if value is None or not value.strip():
        return default
    return int(value)


def _env_optional_int(name: str) -> int | None:
    value = os.getenv(name)
    if value is None or not value.strip():
        return None
    return int(value)


def _env_optional_float(name: str) -> float | None:
    value = os.getenv(name)
    if value is None or not value.strip():
        return None
    return float(value)


def _env_bool(name: str, default: bool) -> bool:
    value = os.getenv(name)
    if value is None or not value.strip():
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


@dataclass(slots=True, frozen=True)
class Settings:
    host: str = _env_str("PIBO_ASR_HOST", "127.0.0.1")
    port: int = _env_int("PIBO_ASR_PORT", 8765)
    model_name: str = _env_str("PIBO_ASR_MODEL", "Qwen/Qwen3-ASR-1.7B")
    gpu_memory_utilization: float = _env_float("PIBO_ASR_GPU_MEMORY_UTILIZATION", 0.85)
    cpu_offload_gb: float = _env_float("PIBO_ASR_CPU_OFFLOAD_GB", 0.0)
    chunk_size_sec: float = _env_float("PIBO_ASR_CHUNK_SIZE_SEC", 0.5)
    unfixed_chunk_num: int = _env_int("PIBO_ASR_UNFIXED_CHUNK_NUM", 4)
    unfixed_token_num: int = _env_int("PIBO_ASR_UNFIXED_TOKEN_NUM", 5)
    max_new_tokens: int = _env_int("PIBO_ASR_MAX_NEW_TOKENS", 32)
    max_model_len: int = _env_int("PIBO_ASR_MAX_MODEL_LEN", 2048)
    max_num_batched_tokens: int | None = _env_optional_int("PIBO_ASR_MAX_NUM_BATCHED_TOKENS") or 256
    max_num_seqs: int | None = _env_optional_int("PIBO_ASR_MAX_NUM_SEQS") or 1
    kv_cache_memory_bytes: int | None = _env_optional_int("PIBO_ASR_KV_CACHE_MEMORY_BYTES")
    enforce_eager: bool = _env_bool("PIBO_ASR_ENFORCE_EAGER", False)
    max_inference_batch_size: int = _env_int("PIBO_ASR_MAX_INFERENCE_BATCH_SIZE", 1)
    session_ttl_seconds: int = _env_int("PIBO_ASR_SESSION_TTL_SECONDS", 600)
    state_dir: Path = Path(_env_str("PIBO_ASR_STATE_DIR", str(Path.home() / ".local/state/pibo-local-asr-tray")))
    hf_home: Path = Path(_env_str("HF_HOME", str(Path.home() / ".cache/pibo-local-asr-tray/hf")))
    log_file: Path = Path(_env_str("PIBO_ASR_LOG_FILE", str(Path.home() / ".local/state/pibo-local-asr-tray/backend.log")))


def get_settings() -> Settings:
    settings = Settings()
    if settings.host != "127.0.0.1":
        raise ValueError("PIBO_ASR_HOST must be 127.0.0.1 for local-only V1.")
    return settings
