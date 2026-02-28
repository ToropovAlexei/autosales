# Development Setup

## Docker Compose (recommended)

1. Create a root `.env` (Docker Compose reads it automatically). Use `backend_rust/.env` as a starting point and adjust secrets/URLs.
2. Start services:

```bash
docker compose up -d db redis captcha_service mock_gateway
```

3. Start the app stack:

```bash
docker compose up -d backend frontend bot
```

Notes:

- `backend_rust` container runs DB migrations on startup, initializes the admin user/bots, and optionally seeds data when `SEED_DB=1|true`.
- `frontend` is built with `NEXT_PUBLIC_API_URL` and `NEXT_PUBLIC_IMAGES_URL` build args.
- `bot` expects a public `WEBHOOK_HOST`/`WEBHOOK_PORT` and a `BACKEND_API_URL` that ends with `/`.

### Optional: monitoring stack (Loki + Promtail + Grafana)

```bash
docker compose \
  -f docker-compose.yml \
  -f docker-compose.monitoring.yml \
  up -d loki promtail grafana
```

Grafana runs on `http://localhost:${GRAFANA_PORT:-3001}`.
See `docs/monitoring.md` for queries and split-deployment examples.

## Local (without Docker)

### Backend

```bash
cd backend_rust
cargo run
```

- In debug builds, migrations run on startup.
- `backend_rust/src/bin/init.rs` bootstraps admin user, roles, and bots.
- `backend_rust/src/bin/seed.rs` seeds demo categories/products and an `admin_dev` user.

### Telegram bot

```bash
cd tgbot_rust
cargo run
```

- Requires Redis and a reachable backend API (`BACKEND_API_URL` must end with `/`).
- The webhook server listens on `WEBHOOK_HOST:WEBHOOK_PORT` and exposes `/webhook/dispatch-message`.

### Frontend (admin panel)

```bash
cd frontend
pnpm install
pnpm dev
```

### Type generation (frontend)

`shared_dtos` is the single source of truth for generated TS models in `frontend/src/types`.

```bash
cd shared_dtos
TS_RS_EXPORT_DIR=../frontend/src/types cargo test --features ts
```

Avoid generating `ts-rs` types from multiple crates into the same output directory.

## Common environment variables

- Database: `DATABASE_HOST`, `DATABASE_PORT`, `DATABASE_USER`, `DATABASE_PASSWORD`, `DATABASE_NAME`
- Redis: `REDIS_HOST`, `REDIS_PORT`
- Backend: `BACKEND_PORT`, `CORS_ORIGINS`, `SERVICE_API_KEY`, `IMAGE_UPLOAD_PATH`
- Payments: `PLATFORM_PAYMENT_SYSTEM_*`, optional `MOCK_PAYMENTS_PROVIDER_URL`
- Bot: `TELEGRAM_API_ID`, `TELEGRAM_API_HASH`, `BACKEND_API_URL`, `WEBHOOK_HOST`, `WEBHOOK_PORT`
- Frontend: `NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_IMAGES_URL`

## Pre-commit (optional)

The repo includes a local `pre-commit` hook that runs `cargo sqlx prepare` in `backend_rust`.
Install with:

```bash
pre-commit install
```

The hook sources `backend_rust/.env` to pick up `DATABASE_URL`.
