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

Shared DTOs are exported into `frontend/src/types` (Rust `ts-rs` models). Ensure backend exports target the correct path if you regenerate them.
