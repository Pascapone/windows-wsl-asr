# Backend API Contract

Base URL: `http://127.0.0.1:8765`

## `GET /healthz`

```json
{
  "ok": true,
  "model_loaded": true,
  "model_name": "Qwen/Qwen3-ASR-1.7B",
  "backend": "qwen-asr[vllm]"
}
```

## `POST /api/start`

Request:

```json
{
  "context": "Pibo\nOpenClaw\nPascal",
  "language": "de",
  "session_meta": {
    "client": "windows-tray",
    "version": "0.1.0"
  }
}
```

Response:

```json
{
  "session_id": "uuid-string"
}
```

## `POST /api/chunk?session_id=<id>`

- Content-Type: `application/octet-stream`
- Body: raw `float32` little-endian mono `16kHz` PCM

Response:

```json
{
  "language": "de",
  "text": "partielles Transkript"
}
```

## `POST /api/finish?session_id=<id>`

```json
{
  "language": "de",
  "text": "finales Transkript"
}
```

## `POST /api/cancel?session_id=<id>`

```json
{
  "ok": true
}
```

## Fehler

```json
{
  "error": "invalid_session"
}
```

```json
{
  "error": "backend_starting",
  "message": "model loading failed"
}
```

```json
{
  "error": "internal_error",
  "message": "human readable message"
}
```
