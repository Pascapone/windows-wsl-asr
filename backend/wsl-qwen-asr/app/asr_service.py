from __future__ import annotations

import logging
import threading
import time
import uuid
from dataclasses import dataclass, field

import numpy as np
from qwen_asr import Qwen3ASRModel

from .settings import Settings

logger = logging.getLogger("pibo.asr")

_LANGUAGE_ALIASES = {
    "ar": "Arabic",
    "cantonese": "Cantonese",
    "cs": "Czech",
    "cz": "Czech",
    "da": "Danish",
    "de": "German",
    "deu": "German",
    "el": "Greek",
    "en": "English",
    "es": "Spanish",
    "fa": "Persian",
    "fi": "Finnish",
    "fil": "Filipino",
    "fr": "French",
    "gr": "Greek",
    "hi": "Hindi",
    "hu": "Hungarian",
    "id": "Indonesian",
    "it": "Italian",
    "ja": "Japanese",
    "ko": "Korean",
    "macedonian": "Macedonian",
    "ms": "Malay",
    "nl": "Dutch",
    "pl": "Polish",
    "pt": "Portuguese",
    "ro": "Romanian",
    "ru": "Russian",
    "sv": "Swedish",
    "th": "Thai",
    "tr": "Turkish",
    "vi": "Vietnamese",
    "yue": "Cantonese",
    "zh": "Chinese",
    "zh-cn": "Chinese",
    "zh-hans": "Chinese",
    "zh-hk": "Cantonese",
    "zh-tw": "Chinese",
}


class BackendStartingError(RuntimeError):
    pass


class InvalidSessionError(KeyError):
    pass


def normalize_language_hint(language: str | None) -> str | None:
    if language is None:
        return None

    candidate = language.strip()
    if not candidate:
        return None

    alias = _LANGUAGE_ALIASES.get(candidate.lower())
    if alias is not None:
        return alias

    return candidate[:1].upper() + candidate[1:].lower()


@dataclass(slots=True)
class Session:
    session_id: str
    state: object
    context: str
    language_hint: str | None
    chunk_count: int = 0
    created_at: float = field(default_factory=time.time)
    updated_at: float = field(default_factory=time.time)

    def touch(self) -> None:
        self.updated_at = time.time()


class AsrService:
    def __init__(self, settings: Settings) -> None:
        self.settings = settings
        self._lock = threading.RLock()
        self._model: Qwen3ASRModel | None = None
        self._model_error: str | None = None
        self._sessions: dict[str, Session] = {}
        self._last_cleanup = 0.0

    @property
    def model_loaded(self) -> bool:
        return self._model is not None

    @property
    def model_error(self) -> str | None:
        return self._model_error

    def ensure_model_loaded(self) -> Qwen3ASRModel:
        with self._lock:
            if self._model is not None:
                return self._model

            logger.info("Loading ASR model %s", self.settings.model_name)
            try:
                llm_kwargs: dict[str, object] = {
                    "model": self.settings.model_name,
                    "gpu_memory_utilization": self.settings.gpu_memory_utilization,
                    "cpu_offload_gb": self.settings.cpu_offload_gb,
                    "max_new_tokens": self.settings.max_new_tokens,
                    "max_model_len": self.settings.max_model_len,
                    "max_inference_batch_size": self.settings.max_inference_batch_size,
                    "enforce_eager": self.settings.enforce_eager,
                    "trust_remote_code": True,
                }
                if self.settings.max_num_batched_tokens is not None:
                    llm_kwargs["max_num_batched_tokens"] = self.settings.max_num_batched_tokens
                if self.settings.max_num_seqs is not None:
                    llm_kwargs["max_num_seqs"] = self.settings.max_num_seqs
                if self.settings.kv_cache_memory_bytes is not None:
                    llm_kwargs["kv_cache_memory_bytes"] = self.settings.kv_cache_memory_bytes

                logger.info(
                    "vLLM config gpu_memory_utilization=%s cpu_offload_gb=%s max_model_len=%s "
                    "max_num_batched_tokens=%s max_num_seqs=%s max_inference_batch_size=%s "
                    "kv_cache_memory_bytes=%s enforce_eager=%s",
                    self.settings.gpu_memory_utilization,
                    self.settings.cpu_offload_gb,
                    self.settings.max_model_len,
                    self.settings.max_num_batched_tokens,
                    self.settings.max_num_seqs,
                    self.settings.max_inference_batch_size,
                    self.settings.kv_cache_memory_bytes,
                    self.settings.enforce_eager,
                )
                self._model = Qwen3ASRModel.LLM(
                    **llm_kwargs,
                )
            except Exception as exc:  # pragma: no cover - depends on runtime env
                self._model_error = str(exc)
                logger.exception("Failed to load ASR model")
                raise BackendStartingError(self._model_error) from exc

            self._model_error = None
            logger.info("ASR model ready")
            return self._model

    def cleanup_expired_sessions(self) -> None:
        now = time.time()
        if now - self._last_cleanup < 5:
            return

        self._last_cleanup = now
        expired_ids: list[str] = []
        with self._lock:
            ttl = self.settings.session_ttl_seconds
            for session_id, session in self._sessions.items():
                if now - session.updated_at > ttl:
                    expired_ids.append(session_id)

            for session_id in expired_ids:
                self._sessions.pop(session_id, None)

        if expired_ids:
            logger.info("Cleaned up %s expired sessions", len(expired_ids))

    def start_session(self, context: str = "", language: str | None = None) -> str:
        model = self.ensure_model_loaded()
        self.cleanup_expired_sessions()
        normalized_language = normalize_language_hint(language)

        session_id = str(uuid.uuid4())
        state = model.init_streaming_state(
            context=context or "",
            language=normalized_language,
            chunk_size_sec=self.settings.chunk_size_sec,
            unfixed_chunk_num=self.settings.unfixed_chunk_num,
            unfixed_token_num=self.settings.unfixed_token_num,
        )
        session = Session(
            session_id=session_id,
            state=state,
            context=context or "",
            language_hint=normalized_language,
        )
        with self._lock:
            self._sessions[session_id] = session
        logger.info(
            "Started session %s language=%s context_chars=%s chunk_size_sec=%s",
            session_id,
            normalized_language,
            len(context or ""),
            self.settings.chunk_size_sec,
        )
        return session_id

    def push_chunk(self, session_id: str, chunk: bytes) -> dict[str, object]:
        started = time.perf_counter()
        session = self._get_session(session_id)
        model = self.ensure_model_loaded()
        pcm = np.frombuffer(chunk, dtype="<f4")
        session.state = model.streaming_transcribe(pcm, session.state)
        session.touch()
        session.chunk_count = int(getattr(session.state, "chunk_id", session.chunk_count + 1))
        audio_accum = getattr(session.state, "audio_accum", np.zeros((0,), dtype=np.float32))
        audio_seconds = float(audio_accum.shape[0]) / 16_000.0
        text = session.state.text or ""
        processing_ms = (time.perf_counter() - started) * 1000.0
        logger.info(
            "Processed chunk session=%s chunk=%s samples=%s audio_seconds=%.2f processing_ms=%.1f text_length=%s",
            session_id,
            session.chunk_count,
            pcm.shape[0],
            audio_seconds,
            processing_ms,
            len(text),
        )
        return {
            "language": session.state.language or session.language_hint,
            "text": text,
            "chunk_index": session.chunk_count,
            "audio_seconds": audio_seconds,
            "processing_ms": processing_ms,
            "text_length": len(text),
        }

    def finish_session(self, session_id: str) -> dict[str, object]:
        started = time.perf_counter()
        with self._lock:
            session = self._sessions.pop(session_id, None)
        if session is None:
            raise InvalidSessionError(session_id)

        model = self.ensure_model_loaded()
        session.state = model.finish_streaming_transcribe(session.state)
        audio_accum = getattr(session.state, "audio_accum", np.zeros((0,), dtype=np.float32))
        tail = getattr(session.state, "buffer", np.zeros((0,), dtype=np.float32))
        audio_seconds = float(audio_accum.shape[0] + tail.shape[0]) / 16_000.0
        text = session.state.text or ""
        processing_ms = (time.perf_counter() - started) * 1000.0
        logger.info(
            "Finished session %s chunks=%s audio_seconds=%.2f processing_ms=%.1f text_length=%s",
            session_id,
            session.chunk_count,
            audio_seconds,
            processing_ms,
            len(text),
        )
        return {
            "language": session.state.language or session.language_hint,
            "text": text,
            "chunk_index": session.chunk_count,
            "audio_seconds": audio_seconds,
            "processing_ms": processing_ms,
            "text_length": len(text),
        }

    def cancel_session(self, session_id: str) -> None:
        with self._lock:
            removed = self._sessions.pop(session_id, None)
        if removed is None:
            raise InvalidSessionError(session_id)
        logger.info("Cancelled session %s", session_id)

    def health(self) -> dict[str, object]:
        try:
            if self._model is None:
                self.ensure_model_loaded()
        except BackendStartingError:
            pass

        return {
            "ok": True,
            "model_loaded": self.model_loaded,
            "model_name": self.settings.model_name,
        }

    def _get_session(self, session_id: str) -> Session:
        self.cleanup_expired_sessions()
        with self._lock:
            session = self._sessions.get(session_id)
        if session is None:
            raise InvalidSessionError(session_id)
        return session
