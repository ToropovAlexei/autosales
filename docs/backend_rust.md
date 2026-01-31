# backend_rust

## Purpose

Rust backend for admin dashboard, bot-facing API, images, and payment webhooks. It also runs background workers for payment polling and broadcasts.

## Entry points

- `backend_rust/src/main.rs` - API server + OpenAPI + workers
- `backend_rust/src/bin/init.rs` - bootstraps admin user, roles/permissions, bots, and store balance
- `backend_rust/src/bin/seed.rs` - seeds demo categories/products and `admin_dev`

## HTTP surface

- `/healthz`
- `/api/admin/*`
- `/api/bot/*`
- `/api/webhook/*`
- `/api/images/*`
- `/api/bot/customers/{telegram_id}/referral-analytics` (referral stats)

OpenAPI is available at `/swagger-ui` and `/openapi.json`.

## Data model notes

- `bots.owner_id` references `customers.id` (the bot API maps from `telegram_id` when creating bots).
- Referral payouts are tracked as `transactions` with `type = referral_payout` and `bot_id` set.
- Bot list queries are encoded with `serde_qs` and shared list DTOs from `shared_dtos::list_query`.

## Auth notes

- Admin auth is 2-step (login + TOTP) and uses `Authorization: Bearer <uuid>` access tokens stored in DB.
- Bot auth uses `X-API-KEY` (service key) + `X-BOT-ID`.

## Subscriptions (Contms)

- Subscription products are created/synced by the Contms worker; access details are provider-owned.
- On purchase, a subscription is created with access credentials and returned to the bot.
- `order_items.quantity` is required to be `> 0` by DB constraint.

## Background workers

- Pending payments polling
- Broadcasts scheduler
- Optional Contms product sync (`contms-provider` feature)

## Configuration

Loaded from environment via `Config::from_env()` in `backend_rust/src/config.rs`.

Required:

- `BACKEND_PORT`
- `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_USER`, `DATABASE_PASSWORD`, `DATABASE_NAME`
- `REDIS_HOST`, `REDIS_PORT`
- `CORS_ORIGINS`
- `JWT_SECRET`
- `TOTP_ENCODE_SECRET`
- `TWO_FA_TOKEN_TTL_MINUTES`, `ACCESS_TOKEN_TTL_MINUTES`, `REFRESH_TOKEN_TTL_MINUTES`
- `IMAGE_UPLOAD_PATH`
- `SERVICE_API_KEY`
- `CAPTCHA_API_URL`
- `PAYMENT_NOTIFICATION_MINUTES`
- `PLATFORM_PAYMENT_SYSTEM_BASE_URL`, `PLATFORM_PAYMENT_SYSTEM_LOGIN`, `PLATFORM_PAYMENT_SYSTEM_PASSWORD`, `PLATFORM_PAYMENT_SYSTEM_2FA_KEY`

Feature-gated:

- `CONTMS_API_URL` (`contms-provider` feature)
- `MOCK_PAYMENTS_PROVIDER_URL` (`mock-payments-provider` feature)

## Migrations

- Dev: migrations run automatically on startup in debug builds.
- Docker: entrypoint runs `sqlx migrate run` before starting the server.
- Manual: `sqlx migrate run --source backend_rust/migrations`.

## Image storage

Files are stored under `IMAGE_UPLOAD_PATH` and sharded by hash prefix.
