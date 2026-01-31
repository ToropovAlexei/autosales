# System Overview

## Components

- `backend_rust` - Core API (admin + bot), payments, images, webhooks, and background workers.
- `tgbot_rust` - Telegram bot service, long-running bot logic + webhook for dispatching notifications.
- `frontend` - Admin panel (Next.js + MUI) that talks to `backend_rust`.
- `shared_dtos` - Shared Rust DTO crate for bot-facing API contracts and enums.
- Supporting services: PostgreSQL (primary storage), Redis (notifications + bot state), captcha service, optional mock payment gateway.

## High-level flow

1. Admins manage catalog, settings, and orders via the frontend (admin API).
2. The Telegram bot fetches catalog data and purchases through the bot API.
3. Payments are created as invoices; background workers track status and trigger notifications.
4. Notifications are published to Redis; `tgbot_rust` picks them up and sends Telegram messages.
5. Referral stats are derived from `referral_payout` transactions grouped by bot.

## Subscriptions (Contms)

- Subscription products are provisioned via the Contms provider.
- Access credentials are returned to the bot and shown to the user after purchase.
- Product details for subscriptions are provider-owned (not managed in admin UI).

## Key HTTP surfaces

- `backend_rust`:
  - `/api/admin/*` - Admin panel API
  - `/api/bot/*` - Bot-facing API
  - `/api/webhook/*` - Payment provider callbacks
  - `/api/images/*` - Image storage endpoints
- `tgbot_rust`:
  - `/webhook/dispatch-message` - Bot notification entrypoint (called by backend)

## Auth model (summary)

- Admin: 2-step login (password -> temp token, TOTP -> access token). API uses `Authorization: Bearer <uuid>` (tokens stored in DB).
- Bot: uses `X-API-KEY` (service key) and `X-BOT-ID` headers.
