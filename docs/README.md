# Documentation

This folder documents the current Rust-based stack (`backend_rust` + `tgbot_rust` + `frontend` + `shared_dtos`).

## Index

- `docs/overview.md` - System overview and key flows
- `docs/development.md` - Local development and Docker setup
- `docs/backend_rust.md` - Rust backend API, workers, and configuration
- `docs/tgbot_rust.md` - Rust Telegram bot service and configuration
- `docs/frontend.md` - Admin panel (Next.js) setup
- `docs/withdrawal-operator-flow-plan.md` - Implementation plan for manual USDT deposit/withdrawal operator flow

## API Docs

The backend exposes OpenAPI at `http://<backend-host>:<port>/swagger-ui` and `http://<backend-host>:<port>/openapi.json`.
