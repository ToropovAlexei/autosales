# tgbot_rust

## Purpose

Telegram bot service with two runtime parts:

- primary customer bots (polled via `BotManager`).
- optional manager bot (`MANAGER_BOT_TOKEN`) for operator group flow.

The service talks to `backend_rust`, stores dialogue state in Redis, and also consumes Redis pub/sub notifications.

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
- `PAYMENT_INSTRUCTIONS_URL`

Optional:

- `MANAGER_BOT_TOKEN` - enables separate manager bot runtime.

## Logging

- Console: pretty human-readable logs to stdout.
- File: JSON logs at `logs/app.log` (daily rolling).
- Control verbosity with `RUST_LOG` (for example `info`, `debug`, or `tgbot_rust=debug`).

## Runtime notes

- The bot starts an Axum server for webhook delivery and runs bot polling logic in a separate task.
- Customer notifications are consumed from Redis channel `bot-notifications:{bot_id}`.
- Dialogue state and user flow state are persisted in Redis.
- Subscription purchases return access details (host/port/login/password) which are rendered in the bot UI.
- Referral stats are fetched from `/api/bot/customers/{telegram_id}/referral-analytics`.

Manager bot flow notes:

- Manager bot subscribes to Redis channel `bot-admin-notifications`.
- On `StoreBalanceRequestNotification`, it sends a message with inline buttons to the configured manager group.
- Group id is auto-synced into backend settings (`manager_group_chat_id`) from manager-bot group updates.
- On callback:
  - approve -> `POST /api/bot/store-balance/{id}/complete`
  - reject -> `POST /api/bot/store-balance/{id}/reject`
- After successful callback processing, the manager bot deletes the original request message.
