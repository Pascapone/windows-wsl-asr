from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class SessionMeta(BaseModel):
    client: str | None = None
    version: str | None = None


class StartRequest(BaseModel):
    model_config = ConfigDict(extra="ignore")

    context: str = ""
    language: str | None = None
    session_meta: SessionMeta | None = None


class StartResponse(BaseModel):
    session_id: str


class TranscriptResponse(BaseModel):
    language: str | None = None
    text: str = ""
    chunk_index: int | None = None
    audio_seconds: float | None = None
    processing_ms: float | None = None
    text_length: int | None = None


class CancelResponse(BaseModel):
    ok: Literal[True] = True


class ErrorResponse(BaseModel):
    error: str
    message: str | None = None


class HealthResponse(BaseModel):
    ok: bool = True
    model_loaded: bool
    model_name: str
    backend: str = Field(default="qwen-asr[vllm]")
