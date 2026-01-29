# Documentation

This folder documents the current Rust-based stack (backend_rust + tgbot_rust + frontend). Legacy services (`backend_go`, `tgbot` Python) are deprecated and not covered here.

## Index

- `docs/overview.md` - System overview and key flows
- `docs/development.md` - Local development and Docker setup
- `docs/backend_rust.md` - Rust backend API, workers, and configuration
- `docs/tgbot_rust.md` - Rust Telegram bot service and configuration
- `docs/frontend.md` - Admin panel (Next.js) setup

## API Docs

The backend exposes OpenAPI at `http://<backend-host>:<port>/swagger-ui` and `http://<backend-host>:<port>/openapi.json`.
