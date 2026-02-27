# E-commerce platform (Rust backend + Next.js admin + Rust Telegram bot)

This repository contains a Rust backend, a Next.js admin panel, and a Rust Telegram bot.

## Components

- `backend_rust` - Admin + bot API, webhooks, images, and background workers
- `tgbot_rust` - Telegram bot service + webhook endpoint for notifications
- `frontend` - Admin panel (Next.js + MUI)
- `shared_dtos` - Shared Rust DTOs (API contracts, enums, error models)
- Supporting services: PostgreSQL, Redis, captcha service, optional mock payment gateway

## Quick start (Docker Compose)

1. Create a root `.env` (Docker Compose reads it automatically). Use `backend_rust/.env` as a starting point and update secrets/URLs.
2. Start services:

```bash
docker compose up -d db redis captcha_service mock_gateway
```

3. Start the app stack:

```bash
docker compose up -d backend frontend bot
```

API docs are available at `http://localhost:8000/swagger-ui` once the backend is running.

## Split deployment (backend and bot on different servers)

- Core server (`backend_rust` + `frontend` + infra):

```bash
docker compose -f docker-compose.core.yml up -d db redis captcha_service mock_gateway backend frontend
```

- Bot server (`tgbot_rust` only):

```bash
docker compose -f docker-compose.bot.yml up -d bot
```

Bot server env must include:

- `REDIS_HOST`, `REDIS_PORT` (shared Redis address reachable from bot VPS)
- `BACKEND_API_URL` (for example `https://api.example.com/api/`, must end with `/`)
- `SERVICE_API_KEY` (same value as backend)
- `TELEGRAM_API_ID`, `TELEGRAM_API_HASH`, `WEBHOOK_HOST`, `WEBHOOK_PORT`, `PAYMENT_INSTRUCTIONS_URL`

## Local development (without Docker)

- Backend:

```bash
cd backend_rust
cargo run
```

- Telegram bot:

```bash
cd tgbot_rust
cargo run
```

Note: set `MANAGER_BOT_TOKEN` in env to enable the separate manager bot runtime for operator group actions.

- Frontend:

```bash
cd frontend
pnpm install
pnpm dev
```

## Documentation

See `docs/README.md` for detailed setup and architecture notes.

## TS DTO generation (important)

`shared_dtos` is the single source of truth for generated frontend types.
Generate TS models from `shared_dtos` only.

```bash
cd shared_dtos
TS_RS_EXPORT_DIR=../frontend/src/types cargo test --features ts
```

## Pre-commit (optional)

This repo includes a local `pre-commit` hook that runs `cargo sqlx prepare` in `backend_rust`.
Install hooks with `pre-commit install` from the repo root after you install `pre-commit`.
