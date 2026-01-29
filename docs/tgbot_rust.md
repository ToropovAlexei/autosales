# tgbot_rust

## Purpose

Telegram bot service that talks to the bot API in `backend_rust`, stores dialogue state in Redis, and exposes a webhook for notifications from the backend.

## HTTP surface

- `POST /webhook/dispatch-message` - accepts notification payloads from the backend and dispatches Telegram messages.

## Configuration

Loaded via `Config::from_env()` in `tgbot_rust/src/config.rs`.

Required:
- `SERVICE_API_KEY` - shared service key (same value as backend)
- `TELEGRAM_API_ID`, `TELEGRAM_API_HASH`
- `REDIS_HOST`, `REDIS_PORT`
- `BACKEND_API_URL` (must end with `/`, for example `http://localhost:8000/api/`)
- `WEBHOOK_HOST`, `WEBHOOK_PORT`
- `CAPTCHA_API_URL`
- `SUPPORT_URL`
- `PAYMENT_INSTRUCTIONS_URL`

## Runtime notes

- The bot starts an Axum server for webhook delivery and runs bot polling logic in a separate task.
- Notifications are pushed from the backend to `BOT_DISPATCHER_WEBHOOK_URL`.
- Dialogue state and user flow state are persisted in Redis.
