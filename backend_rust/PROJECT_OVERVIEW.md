# backend_rust project overview

Last reviewed: 2026-01-26

## Purpose
Rust backend for an admin dashboard + bot-facing API + payment webhooks. It exposes HTTP endpoints with Axum, persists data in Postgres (sqlx), and pushes bot notifications over Redis pub/sub.

## Entry points
- `backend_rust/src/main.rs`: API server, OpenAPI/Swagger, background workers.
- `backend_rust/src/bin/init.rs`: bootstrap admin user, roles/permissions, bots, and initial store balance.
- `backend_rust/src/bin/seed.rs`: dev seed data (categories/products) and admin account.

## High-level architecture
- `presentation/*`: HTTP routes + handlers.
  - `admin`: admin dashboard API (auth, RBAC, products, settings, etc.).
  - `bot`: bot-facing API (catalog, purchase, invoices, etc.).
  - `webhook`: payment provider callbacks.
  - `images`: public image endpoints.
- `middlewares/*`: request extraction and guards.
  - `AuthUser`: Authorization: `Bearer <uuid>` token to admin.
  - `RequirePermission<P>`: permission check via RBAC.
  - `VerifiedService`: `X-API-KEY` for internal service calls.
  - `AuthBot`: `X-BOT-ID` plus `VerifiedService` validation.
  - `RequestContext`: captures IP/user-agent/request-id.
- `services/*`: business logic, mostly traits + concrete impls.
- `infrastructure/*`: repositories (sqlx), external providers (payments, product provider), shared query helpers.
- `models/*`: DB row models, enums, and list query types.
- `workers/*`: background loops (pending payments, broadcasts, contms sync).

## Runtime setup (AppState)
`AppState` wires all repositories, services, Redis pool, and external providers. It also holds:
- `reqwest::Client` shared by external providers.
- `deadpool_redis::Pool` for notifications.
- Feature-gated providers:
  - `contms-provider`: product sync and subscription flow.
  - `mock-payments-provider`: local payment test flow.

## Configuration (env)
Loaded via `Config::from_env()` in `backend_rust/src/config.rs`:
- App/infra: `backend_port`, `cors_origins`, `image_upload_path`.
- Postgres: `database_host`, `database_port`, `database_user`, `database_password`, `database_name`.
- Redis: `redis_host`, `redis_port`.
- Auth: `jwt_secret`, `totp_encode_secret`, token TTLs.
- Service auth: `service_api_key` (used by `VerifiedService`).
- Captcha: `captcha_api_url`.
- Payments: `platform_payment_system_*`, optional `mock_payments_provider_url`.
- External products: `contms_api_url` (feature-gated).
- Payment polling: `payment_notification_minutes`.

## HTTP surface (routers)
Mounted in `create_app()` (`backend_rust/src/lib.rs`):
- `/healthz`
- `/api/admin/*` (admin dashboard API)
- `/api/bot/*` (bot-facing API)
- `/api/webhook/*` (payment webhooks)
- `/api/images/*` (image endpoints)

Admin router (`backend_rust/src/presentation/admin/router.rs`) nests:
- auth, me, categories, products, images, customers, admin-users, roles, permissions
- transactions, stock-movements, settings, audit-logs, bots, orders
- store-balance, broadcasts

Bot router (`backend_rust/src/presentation/bot/router.rs`) nests:
- settings, categories, products, bots, can-operate, captcha
- customers, gateways, invoices, orders

Webhook router (`backend_rust/src/presentation/webhook/router.rs`) nests:
- payment

Swagger/OpenAPI: `backend_rust/src/main.rs` uses `utoipa` + `utoipa-swagger-ui` at `/swagger-ui` with `/openapi.json`. ApiDoc now includes admin + bot + images + webhook endpoints.

## Auth + RBAC
- Admin login is 2-step: login/password -> temp token; then TOTP -> access token (`active_tokens`).
- Access token is a UUID in the Authorization header. No JWT is used for API access.
- `RequirePermission<P>` checks RBAC using effective permissions (roles + direct user permissions).

## Data model (migrations)
Migrations in `backend_rust/migrations` create the core tables:
- auth/RBAC: `admin_users`, `roles`, `permissions`, `role_permissions`, `user_roles`, `user_permissions`
- tokens: `active_tokens`, `temporary_tokens`
- catalog: `categories`, `products`, `images`
- orders/payments: `orders`, `order_items`, `transactions`, `payment_invoices`
- customers/bots: `customers`, `bots`, `user_subscriptions`
- ops: `settings`, `audit_logs`, `stock_movements`, `broadcasts`
- utility: updated-at trigger (`init_updated_at_trigger`)

## Key flows

### Admin login
1) `POST /api/admin/auth/login` -> temp token
2) `POST /api/admin/auth/login/2fa` -> access token (UUID)
3) `AuthUser` middleware validates token against `active_tokens`

### Product pricing
- `ProductService` combines product rows with current settings (pricing/markup/discounts) to derive `price` vs `base_price`.
- Stock changes use `stock_movements` and are logged into `audit_logs`.

### Purchase (bot API)
`PurchaseService::purchase_product`:
- Validate stock and customer balance.
- Create `orders` + `order_items` + `transactions` (type `Purchase`).
- For subscription products with `provider_name == "contms"`, call Contms and create `user_subscriptions`.

### Payment invoices
`PaymentInvoiceService`:
- Creates invoices via `mock` or `AutosalesPlatform` provider.
- Applies gateway-specific discount from `settings`.
- Stores payment details and order_id for tracking.
- Supports confirm/cancel/receipt submission.

`PaymentProcessingService::handle_payment_success`:
- Reads invoice + customer.
- Creates a `Deposit` transaction and updates store balance.
- Marks invoice as completed and publishes a notification to Redis.

### Background workers
- `pending_payments_task` (every 1 min):
  - Expire old invoices.
  - Poll Autosales platform for status updates.
  - Notify users about pending/receipt-required/completed payments.
  - Update invoice statuses.
- `broadcasts_task` (every 1 min):
  - Fetch scheduled broadcasts, apply customer filters, push notifications via Redis.
- `contms_products_sync_task` (every 5 min, feature-gated):
  - Sync Contms products into `products`.
  - Ensures a "Прокси" category exists.

## External integrations
- Payments:
  - Autosales platform (card/SBP) with 2FA auth; cached token.
  - Mock provider for local testing.
- Products:
  - Contms provider for proxy subscriptions (create/renew/unsubscribe).
- Captcha:
  - External captcha API called via `CaptchaService`.

## Redis notifications
- `NotificationService` publishes JSON messages to `bot-notifications:{bot_id}`.
- Messages include generic text/image, invoice trouble alerts, and receipt requests.

## File storage (images)
- `ImageService` hashes uploads (blake3), validates MIME type, extracts dimensions.
- Files are stored on disk under `image_upload_path`, sharded by first 2 chars of hash.

## Testing
- Unit tests exist for money-related service flows:
  - `PaymentInvoiceService` (mock discount + platform card details)
  - `PaymentProcessingService` (deposit transaction + invoice update + notification)
- `ImageService` tests cover metadata extraction and path building.
- Typical commands:
  - `cargo nextest run`
  - `cargo llvm-cov nextest`

## Code layout (core)
- `backend_rust/src/main.rs`: server + Swagger + worker spawning
- `backend_rust/src/lib.rs`: router setup, CORS, tracing, migrations
- `backend_rust/src/state.rs`: dependency graph and service wiring
- `backend_rust/src/presentation/**`: HTTP handlers and DTOs
- `backend_rust/src/services/**`: business logic
- `backend_rust/src/infrastructure/**`: repositories + external providers
- `backend_rust/src/models/**`: DB models + enums
- `backend_rust/src/workers/**`: background tasks

## Notes / gotchas
- Tokens are UUIDs stored in DB, not JWTs in headers.
- Many services emit audit logs; check `audit_logs` table for admin actions.
- `pending_payments_task` assumes Autosales order status polling; if provider is down, invoices may not advance.
- `contms_products_sync_task` is enabled only with the `contms-provider` feature.

