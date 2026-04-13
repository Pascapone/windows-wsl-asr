from __future__ import annotations

import logging
from logging.handlers import RotatingFileHandler

from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import JSONResponse

from .asr_service import AsrService, BackendStartingError, InvalidSessionError
from .schemas import CancelResponse, ErrorResponse, HealthResponse, StartRequest, StartResponse, TranscriptResponse
from .settings import get_settings

settings = get_settings()
service = AsrService(settings)
app = FastAPI(title="Pibo Local ASR Backend", version="0.1.0")


def configure_logging() -> None:
    settings.state_dir.mkdir(parents=True, exist_ok=True)
    logger = logging.getLogger("pibo")
    if logger.handlers:
        return

    logger.setLevel(logging.INFO)
    formatter = logging.Formatter("%(asctime)s %(levelname)s %(name)s %(message)s")

    stream_handler = logging.StreamHandler()
    stream_handler.setFormatter(formatter)
    logger.addHandler(stream_handler)

    file_handler = RotatingFileHandler(settings.log_file, maxBytes=5_000_000, backupCount=3, encoding="utf-8")
    file_handler.setFormatter(formatter)
    logger.addHandler(file_handler)


configure_logging()


@app.exception_handler(InvalidSessionError)
async def invalid_session_handler(_: Request, exc: InvalidSessionError) -> JSONResponse:
    return JSONResponse(status_code=404, content=ErrorResponse(error="invalid_session", message=str(exc)).model_dump())


@app.exception_handler(BackendStartingError)
async def backend_starting_handler(_: Request, exc: BackendStartingError) -> JSONResponse:
    return JSONResponse(
        status_code=503,
        content=ErrorResponse(error="backend_starting", message=str(exc) or "Model is still loading").model_dump(),
    )


@app.exception_handler(ValueError)
async def value_error_handler(_: Request, exc: ValueError) -> JSONResponse:
    return JSONResponse(
        status_code=400,
        content=ErrorResponse(error="invalid_request", message=str(exc)).model_dump(),
    )


@app.exception_handler(Exception)
async def generic_error_handler(_: Request, exc: Exception) -> JSONResponse:
    logging.getLogger("pibo.server").exception("Unhandled backend error")
    return JSONResponse(
        status_code=500,
        content=ErrorResponse(error="internal_error", message=str(exc)).model_dump(),
    )


@app.get("/healthz", response_model=HealthResponse)
async def healthz() -> HealthResponse:
    return HealthResponse(**service.health())


@app.post("/api/start", response_model=StartResponse)
async def start_session(payload: StartRequest) -> StartResponse:
    session_id = service.start_session(context=payload.context, language=payload.language)
    return StartResponse(session_id=session_id)


@app.post("/api/chunk", response_model=TranscriptResponse)
async def push_chunk(request: Request, session_id: str = Query(...)) -> TranscriptResponse:
    body = await request.body()
    if not body:
        return TranscriptResponse(language=None, text="")
    result = service.push_chunk(session_id, body)
    return TranscriptResponse(**result)


@app.post("/api/finish", response_model=TranscriptResponse)
async def finish_session(session_id: str = Query(...)) -> TranscriptResponse:
    result = service.finish_session(session_id)
    return TranscriptResponse(**result)


@app.post("/api/cancel", response_model=CancelResponse)
async def cancel_session(session_id: str = Query(...)) -> CancelResponse:
    service.cancel_session(session_id)
    return CancelResponse()
