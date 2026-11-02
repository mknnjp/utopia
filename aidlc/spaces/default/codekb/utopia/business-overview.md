# Business Overview — Utopia (Firefly III Compat)

## Purpose
Utopia is a lightweight, self-hostable personal finance API that implements a **Firefly III-compatible** subset of the Firefly III household finance domain. It targets users who want Firefly III client compatibility (mobile apps, importers, scripts) without running the full PHP Firefly III stack. The system is implemented in Rust (Axum + SQLx + Postgres) and exposes a JSON API whose envelope, pagination, error shape, and resource fields intentionally mirror Firefly III so existing clients can interoperate with minimal changes.

> Intent of this inventory: `Firefly III互換APIの現状を棚卸しして、実装済み仕様と本家との差分、今後の優先順位を整理したい。` — take stock of what is implemented, what diverges from upstream Firefly III, and prioritize what to build next.

## Business Domain
**Domain:** Personal finance / household bookkeeping (double-entry-inspired journals).

Core domain concepts (Firefly III model, partially implemented):

| Concept | Status in Utopia | Notes |
|---|---|---|
| **Accounts** | Implemented (CRUD + list) | 21 account types, soft-delete, extended attributes (IBAN, BIC, virtual_balance, etc.) |
| **Transaction Journals** | Implemented (CRUD + list) | Single-journal per group_id today; withdrawal/deposit/transfer; balance updates with row locking |
| **Currencies** | Partial (read-only static list) | 20 hardcoded currencies, JPY default; no CRUD, no DB table |
| **Budgets** | Not implemented | `src/modules/budgets.rs` is a placeholder |
| **Categories / Tags / Bills / Piggy Banks / Attachments / Recurring / Search / Bulk / Webhooks / Import-Export** | Not implemented | No routes, no schemas, no tables |

## Key Functionality (Implemented)

### 1. Accounts Management
- Create / read / update / soft-delete accounts scoped to the authenticated user.
- List with pagination (`page`, `limit`) and optional `type` filter.
- 21 allowed account types (asset, expense, revenue, liability variants, etc.) validated via `ALLOWED_ACCOUNT_TYPES`.
- Extended attributes via migration `0003` (active, order, initial/virtual balance, IBAN/BIC, notes, account_role, liability fields, cc fields, opening_balance_date).
- Partial unique index `idx_accounts_user_id_name_active` enforces unique active names per user while allowing soft-deleted duplicates.

### 2. Transactions / Journals
- Create / read / update / delete transaction journals.
- List globally (`GET /api/v1/transactions` with `start`, `end`, `type` filters) or per-account (`GET /api/v1/accounts/{id}/transactions`).
- Fields: `group_id`, `transaction_type`, `description`, `amount` (NUMERIC 20,8), `currency_code`, `date`, `source_id`/`destination_id`, `category_name`, `notes`, `reconciled`.
- Balance side-effects: `AccountBalanceUpdate` applied atomically with `SELECT FOR UPDATE` (`lock_accounts_for_update`) inside a DB transaction.

### 3. Authentication & Token Lifecycle
- Bearer token auth: raw token → SHA256 lookup → Argon2 verify → Principal injection.
- Personal access tokens: `POST /api/v1/tokens` (issue), `DELETE /api/v1/tokens/{id}` (revoke).
- Bootstrap provisioning: `POST /api/v1/bootstrap/tokens` with `X-Bootstrap-Key` header, rate-limited (5 req / 60s default), single-use via `bootstrap_key_usage` table with constant-time compare.
- Token cache: `moka` positive + negative caches with configurable TTLs; `update_last_used_at` fire-and-forget.

### 4. Metadata & System Info
- `GET /api/v1/currencies` — paginated static list (20 entries).
- `GET /api/v1/about` — version, api_version, php_version (reports Rust), os, driver (PostgreSQL).
- `GET /api/v1/about/user` — authenticated user profile.
- `GET /metrics` — Prometheus exposition (no auth).

## Users & Actors
- **End user (household member):** owns accounts and journals; authenticates via personal access token.
- **Bootstrap operator:** holds `BOOTSTRAP_KEY` to provision the first token/user; single-use claim.
- **Firefly III clients:** third-party apps expecting Firefly envelope/pagination/error contracts.
- **Operator / SRE:** consumes `/metrics`, JSON logs, audit events.

## Business Rules & Constraints
- All data is **user-scoped** — every repository query filters by `user_id` from the authenticated Principal.
- Soft-delete for accounts (`deleted_at IS NULL` filtering); transactions are hard-deleted.
- Monetary amounts are `rust_decimal::Decimal` stored as `NUMERIC(20,8)`, serialized as strings via `DecimalAmount` (2-decimal formatting today — see tech debt re JPY).
- Pagination defaults: `page=1`, `limit=50`, `max=100` with `div_ceil` total_pages.
- Security: Argon2 `memory_cost >= 65536`, `time_cost >= 3`; `BOOTSTRAP_KEY >= 16 chars`; `DATABASE_URL` must contain `sslmode=require` when `APP_STRICT_SSL=true`.

## Value Proposition
- Drop-in Firefly III API subset for self-hosters who prefer a small Rust service over PHP.
- Strong typing, compile-time SQL checks (`sqlx`), and structured observability (Prometheus + JSON tracing + audit logger).
- Clear extension path: openapi.yaml is the contract source of truth; missing Firefly surface is explicitly inventoried for prioritization.

## Out of Scope / Known Gaps (for prioritization)
- Budgets, categories, tags, bills, piggy banks, attachments, search, bulk ops, webhooks, recurring transactions, data import/export — all absent.
- Currency CRUD, dynamic decimal_places per currency, Link headers, and full Firefly error detail parity.
- See `architecture.md` and `api-documentation.md` for the detailed gap matrix.
