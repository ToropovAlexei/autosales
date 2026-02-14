# frontend (Admin Panel)

## Purpose

Next.js admin panel for managing catalog, orders, settings, users, and bots.

## Tech

- Next.js App Router
- MUI + Emotion for UI
- TanStack React Query for data fetching
- `ky` for HTTP requests

## Configuration

- `NEXT_PUBLIC_API_URL` - admin API base URL (for example `http://localhost:8000/api/admin`)
- `NEXT_PUBLIC_IMAGES_URL` - images base URL (for example `http://localhost:8000/api/images`)

## Commands

```bash
cd frontend
pnpm install
pnpm dev
```

Build/serve:

```bash
pnpm build
pnpm start
```

## Types

Frontend types are generated from `shared_dtos` (single source of truth).
Do not generate `ts-rs` types from multiple crates into `frontend/src/types`.

Generate types:

```bash
cd shared_dtos
TS_RS_EXPORT_DIR=../frontend/src/types cargo test --features ts
```
