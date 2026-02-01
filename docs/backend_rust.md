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
- `/api/admin/dashboard/*` (stats, time series, top products, sales by category)
- `/api/bot/*`
- `/api/webhook/*`
- `/api/images/*`
- `/api/bot/customers/{telegram_id}/referral-analytics` (referral stats)

OpenAPI is available at `/swagger-ui` and `/openapi.json`.

## Data model notes

- `bots.owner_id` references `customers.id` (the bot API maps from `telegram_id` when creating bots).
- Referral payouts are tracked as `transactions` with `type = referral_payout` and `bot_id` set.
- Bot list queries are encoded with `serde_qs` and shared list DTOs from `shared_dtos::list_query`.

## Dashboard analytics (admin)

We re-created the old Go dashboard behavior on top of the current Rust schema.
The frontend expects these exact endpoints (see `frontend/src/constants/endpoints.ts`):

- `GET /api/admin/dashboard/stats`
- `GET /api/admin/dashboard/time-series?start_date=...&end_date=...` (RFC3339)
- `GET /api/admin/dashboard/top-products`
- `GET /api/admin/dashboard/sales-by-category`

### Why the metrics are computed this way

The Rust schema splits order and product info into `orders` + `order_items`.
In the old Go app, most of these values were derived from `orders` (because orders
had a single `product_id`). In Rust, the correct source of truth is `order_items`.

Definitions used in code:

- **Total users** = `COUNT(*)` from `customers`.
- **New users (30 days)** = `customers.created_at` in last 30 days.
- **Users with purchases (all time)** = `COUNT(DISTINCT orders.customer_id)`.
- **Users with purchases (30 days)** = same, but filtered by order `created_at`.
- **Products sold (30 days)** = `SUM(order_items.quantity)` for orders in period.
- **Available products** = `products.stock > 0` and `deleted_at IS NULL`.
- **Revenue** = `SUM(order_items.price_at_purchase * quantity)`.

Notes:

- We intentionally do **not** subtract commissions: commissions are only known on
  **deposit** transactions, while sales happen against user balance. The UI only
  shows purchase revenue, not deposits/fees.
- We do **not** filter by bot: the dashboard is store-wide. Bot filtering can be added
  later (e.g., via `orders.bot_id` and `transactions.bot_id`).
- We do **not** filter by order status today. This matches the old dashboard behavior.
  If you later want to count only `paid/fulfilled`, add a status filter in repo queries.

### Time-series charts

The frontend needs daily series for:

- sales (products sold)
- new users
- revenue
- users with purchases

The service fills missing dates with `0` to ensure charts render with a continuous
date axis. This matches the old Go implementation (`fillMissingDates`).

### Top products / sales by category

- Top products: aggregate revenue by `order_items.product_id`.
- Sales by category: join `products.category_id`, group by category name.
- Category can be `NULL`; we return `"Без категории"` in responses.

Implementation locations:

- Repo: `backend_rust/src/infrastructure/repositories/dashboard.rs`
- Service: `backend_rust/src/services/dashboard.rs`
- DTOs: `backend_rust/src/presentation/admin/dtos/dashboard.rs`
- Handlers/Routes: `backend_rust/src/presentation/admin/handlers/dashboard.rs`

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

## Logging

- Console: pretty human-readable logs to stdout.
- File: JSON logs at `logs/app.log` (daily rolling).
- Control verbosity with `RUST_LOG` (for example `info`, `debug`, or `backend_rust=debug,sqlx::query=info`).

## Migrations

- Dev: migrations run automatically on startup in debug builds.
- Docker: entrypoint runs `sqlx migrate run` before starting the server.
- Manual: `sqlx migrate run --source backend_rust/migrations`.

## Image storage

Files are stored under `IMAGE_UPLOAD_PATH` and sharded by hash prefix.
