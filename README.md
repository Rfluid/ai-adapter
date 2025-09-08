# AI Adapter (Rust) — WAHA → AI Agent Bridge

A tiny, production-ready Axum (Rust) service that:

1. receives **WAHA** webhooks,
2. normalizes them into your **AI Agent**’s `InputRequest`,
3. posts to the AI, and
4. **only** replies back to WAHA if the AI returned a `response`.

It’s structured so you can easily plug in **new messaging products** and **new message types**.

## Features

- **Axum 0.7** HTTP server (Tokio runtime).
- **Env-driven config** (`dotenvy`) with safe defaults.
- **Strict models** (`serde`) and typed services for **AI** and **WAHA**.
- **Message dispatch** with a clear switch for `text` and `unsupported`.
- **Threading**: `thread_id = THREAD_PREFIX_WAHA + user_wa_id`.
- **OpenAPI/Swagger** docs via **utoipa** + **utoipa-swagger-ui** at `/docs`.
- **Docker** multi-stage build, small runtime, non-root.

## Project structure

```
ai-adapter/
├── Cargo.toml
├── Dockerfile
├── .dockerignore
├── src
│   ├── main.rs
│   ├── config.rs
│   ├── utils.rs
│   ├── apidoc.rs
│   ├── routes/
│   │   └── waha.rs
│   ├── services/
│   │   ├── ai.rs
│   │   └── waha.rs
│   ├── models/
│   │   ├── common.rs
│   │   ├── ai.rs
│   │   └── waha.rs
│   └── handlers/
│       ├── mod.rs
│       └── text.rs
└── tests/
    └── integration.rs
```

**Key idea:** keep HTTP endpoints in `routes/`, outbound calls in `services/`, domain types in `models/`, and message-type logic in `handlers/`.

## Requirements

- Rust 1.80+ (or latest stable)
- Cargo
- (Optional) Docker & Docker Compose

## Quick start (local)

1. **Install deps / build**

```bash
cargo build
```

2. **Environment**
   Create `.env` (or export envs). Example:

```ini
APP_HOST=0.0.0.0
APP_PORT=8080

WAHA_BASE_URL=http://localhost:3000
# WAHA_API_KEY_PLAIN=<token>    # if your WAHA requires it

AI_BASE_URL=http://localhost:8000
AI_MESSAGES_USER_PATH=/agent/messages/user

THREAD_PREFIX_WAHA=waha:

CHAT_INTERFACE=api
MAX_RETRIES=1
LOOP_THRESHOLD=3
TOP_K=5
SUMMARIZE_MESSAGE_WINDOW=4
SUMMARIZE_MESSAGE_KEEP=6
SUMMARIZE_SYSTEM_MESSAGES=false
```

3. **Run**

```bash
RUST_LOG=info cargo run
# Server: http://localhost:8080
# Swagger: http://localhost:8080/docs
```

4. **Send a test webhook**

```bash
curl -X POST http://localhost:8080/webhooks/waha \
  -H "Content-Type: application/json" \
  -d '{
        "messages":[
          {"from":"5511912345678","type":"text","text":{"body":"hello"}}
        ]
      }'
```

If your AI responds with `{ "response": "..." }`, the adapter posts a WhatsApp text back to WAHA at `POST {WAHA_BASE_URL}/messages`.

## Docker

**Dockerfile** (multi-stage) and `.dockerignore` are included.

Build & run:

```bash
docker build -t ai-adapter:latest .
docker run --rm -p 8080:8080 \
  -e WAHA_BASE_URL=http://host.docker.internal:3000 \
  -e AI_BASE_URL=http://host.docker.internal:8000 \
  ai-adapter:latest
```

### docker-compose snippet

```yaml
services:
    ai-adapter:
        build: .
        image: ai-adapter:latest
        environment:
            APP_HOST: 0.0.0.0
            APP_PORT: 8080
            WAHA_BASE_URL: http://waha:3000
            AI_BASE_URL: http://ai-agent:8000
            AI_MESSAGES_USER_PATH: /agent/messages/user
            THREAD_PREFIX_WAHA: "waha:"
            CHAT_INTERFACE: api
            MAX_RETRIES: 1
            LOOP_THRESHOLD: 3
            TOP_K: 5
            SUMMARIZE_MESSAGE_WINDOW: 4
            SUMMARIZE_MESSAGE_KEEP: 6
            SUMMARIZE_SYSTEM_MESSAGES: "false"
            # WAHA_API_KEY_PLAIN: <token>
        ports:
            - "8080:8080"
        depends_on:
            - waha
            - ai-agent
        restart: unless-stopped
```

## Configuration (env vars)

| Var                         | Type / Default         | Description                                     |
| --------------------------- | ---------------------- | ----------------------------------------------- |
| `APP_HOST`                  | `0.0.0.0`              | Bind host                                       |
| `APP_PORT`                  | `8080`                 | Bind port                                       |
| `WAHA_BASE_URL`             | **required**           | WAHA base URL (e.g. `http://waha:3000`)         |
| `WAHA_API_KEY_PLAIN`        | optional               | X-Api-Key header value for WAHA, if needed      |
| `AI_BASE_URL`               | **required**           | AI Agent base URL (e.g. `http://ai-agent:8000`) |
| `AI_MESSAGES_USER_PATH`     | `/agent/messages/user` | Path appended to `AI_BASE_URL`                  |
| `THREAD_PREFIX_WAHA`        | `waha:`                | Prefix for thread id; final id = prefix + wa_id |
| `CHAT_INTERFACE`            | `api`                  | Forwarded to AI                                 |
| `MAX_RETRIES`               | `1`                    | Forwarded to AI                                 |
| `LOOP_THRESHOLD`            | `3`                    | Forwarded to AI                                 |
| `TOP_K`                     | `5`                    | Forwarded to AI                                 |
| `SUMMARIZE_MESSAGE_WINDOW`  | `4`                    | Forwarded to AI                                 |
| `SUMMARIZE_MESSAGE_KEEP`    | `6`                    | Forwarded to AI                                 |
| `SUMMARIZE_SYSTEM_MESSAGES` | `false`                | Forwarded to AI                                 |

## API

### POST `/webhooks/waha`

- **Purpose**: Ingest a WAHA webhook and trigger AI.
- **Request body**: Accepts flexible WAHA JSON. Common shape:

```json
{
    "messages": [
        {
            "from": "5511912345678",
            "type": "text",
            "text": { "body": "hello" }
        }
    ]
}
```

- **Behavior**:
    1. Extracts `user_id` (`messages[0].from`), `type` (`messages[0].type`), and optional `text.body`.
    2. Builds `thread_id = THREAD_PREFIX_WAHA + user_id`.
    3. Dispatch:
        - **text** → `InputRequest { data: { text, source: "waha", user_id }, … }`
        - **unsupported** → `InputRequest { data: { unsupported_message_type, raw }, … }`

    4. Calls `POST {AI_BASE_URL}{AI_MESSAGES_USER_PATH}`.
    5. **If** AI returns an object with `response`, posts back to WAHA at `POST {WAHA_BASE_URL}/messages` with a WhatsApp text payload:

        ```json
        {
            "messaging_product": "whatsapp",
            "to": "5511912345678",
            "type": "text",
            "text": { "body": "AI reply here" }
        }
        ```

- **Responses**:
    - `200 OK` – Webhook accepted (reply posting, if any, is already triggered).
    - `500` – Handler error (see logs).

### Documentation (Swagger / OpenAPI)

- **Swagger UI**: `GET /docs`
- **OpenAPI JSON**: `GET /api-docs/openapi.json`

> The OpenAPI includes schemas for `WahaWebhook`, `InputRequest` (doc variant), `LlmApiResponse`, and a basic error body.

## Internals / Flow

1. **routes/waha.rs** → `receive_waha`
   Parses incoming JSON into `WahaWebhook` (lenient), then calls `handlers::dispatch_waha`.

2. **handlers/**
    - `dispatch_waha` switches on `message_type`:
        - `text` → `handlers::text::handle_text`
        - everything else → `handlers::text::handle_unsupported`

    - Both build an `InputRequest` using knobs from `Config`.

3. **services/ai.rs** → `send_user_message`
   Posts JSON to the AI endpoint and parses an `LlmApiResponse`.

4. **services/waha.rs** → `send_text_message`
   If `response` is present, posts a WhatsApp `text` message to WAHA’s `/messages` endpoint (adds `Authorization: Bearer …` if `WAHA_API_KEY_PLAIN` is set).

5. **utils.rs** → `thread_id_for_waha`
   `format!("{}{}", cfg.thread_prefix_waha, user_id)`.

## Extending

### New message types (e.g., image, audio)

- Add a new handler function in `handlers/` (e.g., `handlers/image.rs`).
- Update `handlers::dispatch_waha` switch to route `message_type == "image"` to your new function.
- Expand `models/waha.rs` extraction if you need more fields (e.g., media URLs).

### New messaging products

- Create new route file(s) under `routes/` (e.g., `routes/telegram.rs`).
- Define product-specific models in `models/`.
- Add a new service for sending replies if needed.
- Mirror the same **normalize → AI call → conditional reply** pattern.

## Development

- **Run**: `RUST_LOG=info cargo run`
- **Fmt**: `cargo fmt`
- **Lint**: `cargo clippy -- -D warnings`
- **Build (release)**: `cargo build --release`
- **Test**: add integration tests in `tests/` (e.g., mock AI and post to `/webhooks/waha`).

---

## Notes

- WAHA’s send-message endpoint path (`/messages`) may vary by version/config—adjust in `services/waha.rs` if needed.
- The AI response type in code is `LlmApiResponse` with `response: Option<String>` for tolerance. If your AI always returns a `response`, set it to a non-optional field and tighten checks.
