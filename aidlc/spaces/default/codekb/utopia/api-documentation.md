# API Documentation — Utopia (Firefly III Compat Surface)

## Overview
Utopia exposes a **Firefly III-compatible REST API** over Axum. The contract source of truth is `openapi.yaml` (OpenAPI 3.0.3, ~1500 lines) and `src/api/router.rs` (18 routes: 16 business routes + `/metrics` + `/api/v1/accounts/{id}/transactions`). All business routes are under `/api/v1/*` and require `Authorization: Bearer <token>` except `POST /api/v1/bootstrap/tokens` (uses `X-Bootstrap-Key`) and `GET /metrics` (no auth). Responses use Firefly envelope/pagination/error shapes from `src/core/compatibility/*`.

Base URL: `http://localhost:{port}` (default 8080, configurable via `APP_PORT`). Security: `bearerAuth` (HTTP bearer).

## External REST API (Axum Router — `src/api/router.rs`)

### Route Table (18 routes: 16 business + `/metrics` + account-scoped transactions)

| # | Method | Path | Handler | Auth | Description |
|---|---|---|---|---|---|
| 1 | GET | `/api/v1/accounts` | `list_accounts_handler` | Bearer | List accounts (page, limit, type filter) |
| 2 | POST | `/api/v1/accounts` | `create_account_handler` | Bearer | Create account (201) |
| 3 | GET | `/api/v1/accounts/{id}` | `get_account_handler` | Bearer | Get account by UUID |
| 4 | PUT | `/api/v1/accounts/{id}` | `update_account_handler` | Bearer | Update account |
| 5 | DELETE | `/api/v1/accounts/{id}` | `delete_account_handler` | Bearer | Soft-delete account (204) |
| 6 | GET | `/api/v1/accounts/{id}/transactions` | `list_account_transactions_handler` | Bearer | List transactions for account (page, limit) |
| 7 | GET | `/api/v1/transactions` | `list_transactions_handler` | Bearer | List transactions (page, limit, start, end, type) |
| 8 | POST | `/api/v1/transactions` | `create_transaction_handler` | Bearer | Create transaction journal |
| 9 | GET | `/api/v1/transactions/{id}` | `get_transaction_handler` | Bearer | Get transaction by UUID |
| 10 | PUT | `/api/v1/transactions/{id}` | `update_transaction_handler` | Bearer | Update transaction |
| 11 | DELETE | `/api/v1/transactions/{id}` | `delete_transaction_handler` | Bearer | Delete transaction (204) |
| 12 | POST | `/api/v1/tokens` | `issue_token_handler` | Bearer | Issue personal access token (body: label) |
| 13 | DELETE | `/api/v1/tokens/{id}` | `revoke_token_handler` | Bearer | Revoke token |
| 14 | POST | `/api/v1/bootstrap/tokens` | `bootstrap_issue_token_handler` | X-Bootstrap-Key | Bootstrap token (rate-limited, single-use) |
| 15 | GET | `/api/v1/currencies` | `list_currencies_handler` | Bearer | List currencies (static 20, paginated) |
| 16 | GET | `/api/v1/about` | `get_about_handler` | Bearer | System info (version, api_version, php_version, os, driver) |
| 17 | GET | `/api/v1/about/user` | `get_about_user_handler` | Bearer | Authenticated user profile |
| 18 | GET | `/metrics` | `metrics_handler` | None | Prometheus exposition (text/plain) |

### Query Parameters

**Accounts list** (`GET /api/v1/accounts`):
- `page` (int, default 1) — page number
- `limit` (int, default 50, max 100) — per page
- `type` (string, optional) — filter by account type (21 variants, e.g., `asset`, `expense`, `revenue`, `liability`, `cash`, `creditCard`, etc.)

**Transactions list** (`GET /api/v1/transactions`):
- `page`, `limit` — as above
- `start` (ISO8601 DateTime, optional) — filter `date >= start`
- `end` (ISO8601 DateTime, optional) — filter `date <= end`
- `type` (string, optional) — `withdrawal` | `deposit` | `transfer`

**Account transactions** (`GET /api/v1/accounts/{id}/transactions`):
- `page`, `limit` — as above

**Currencies** (`GET /api/v1/currencies`):
- `page`, `limit` — paginated over static 20-entry table

### Request / Response Schemas (Firefly Compat)

**Envelopes** (`src/core/compatibility/envelope.rs`):
```json
// List
{ "data": [ ... ], "meta": { "pagination": { "total": 100, "count": 50, "per_page": 50, "current_page": 1, "total_pages": 2 } } }
// Single
{ "data": { ... } }
```
- `FireflyListEnvelope<T>` — `data: Vec<T>`, `meta.pagination` computed via `compute_pagination` (`div_ceil`).
- `FireflySingleEnvelope<T>` — `data: T`.

**Pagination** (`src/core/compatibility/pagination.rs`):
- `DEFAULT_PAGE=1`, `DEFAULT_LIMIT=50`, `MAX_LIMIT=100`.
- `Paginated<T> { records, total }` → `PaginationMeta { total, count, per_page, current_page, total_pages }`.

**Error** (`src/core/compatibility/error_response.rs`):
```json
{ "message": "Validation failed", "errors": { "field": ["error message"] } }
```
- `FireflyErrorResponse { message: String, errors: HashMap<String, Vec<String>> }`.
- Mapped from `DomainError` via `src/core/error_mapping/mapper.rs` → HTTP 401/404/409/422/500.

**Decimal** (`src/core/compatibility/decimal_amount.rs`):
- `DecimalAmount` serializes `rust_decimal::Decimal` as string, normalized and padded to 2 decimals (e.g., `"100.00"`). Note: JPY should be 0 decimals per Firefly — current mismatch.

**Account Resource** (`src/modules/accounts/types.rs` → `FireflyAccountResource`):
- Fields: `id`, `name`, `type`, `currency_code`, `current_balance` (string), `active`, `order`, `initial_balance`, `virtual_balance`, `iban`, `bic`, `account_number`, `notes`, `include_net_worth`, `account_role`, `liability_type/direction`, `interest`, `cc_type`, etc. (extended via migration 0003).

**Transaction Resource** (`src/modules/transactions.rs` → `FireflyTransactionResource`):
- Fields: `id`, `group_id`, `transaction_type`, `description`, `amount` (string), `currency_code`, `date`, `source_id`/`destination_id`, `category_name`, `notes`, `reconciled`, `created_at`, `updated_at`.
- Known gap: `user` is empty string, `source_name`/`destination_name` are None (no join).

**Currency Resource** (`src/modules/metadata.rs`):
- `FireflyCurrencyAttributes { code, name, symbol, decimal_places, default, enabled, created_at, updated_at }` — 20 static entries, JPY default.

**Token** (`src/core/auth/models.rs`):
- `POST /api/v1/tokens` body: `{ "label": "string" }` → `{ "token": "<raw>", "id": "<uuid>" }`.
- `POST /api/v1/bootstrap/tokens` header: `X-Bootstrap-Key: <key>` → same response, single-use.

### Status Codes
- `200` — success (GET, PUT)
- `201` — created (POST accounts, transactions, tokens)
- `204` — no content (DELETE)
- `401` — unauthorized (missing/invalid bearer or bootstrap key)
- `404` — not found
- `409` — conflict (bootstrap already claimed)
- `422` — validation error (FireflyErrorResponse)
- `429` — rate limited (bootstrap only)
- `500` — server error

## Internal API Surfaces (Traits — `src/core/persistence/repository.rs`)

| Trait | Methods | Purpose |
|---|---|---|
| `TokenReadRepository` | `find_by_sha256`, `find_by_id` | Token lookup by hash or id |
| `TokenWriteRepository` | `create_token`, `revoke_token` | Token lifecycle (in tx) |
| `TokenUpdateRepository` | `update_last_used_at` | Fire-and-forget last-used update |
| `UserReadRepository` | `find_by_id`, `find_by_email` | User lookup |
| `UserWriteRepository` | `create_user` | User creation (in tx) |
| `BootstrapKeyRepository` | `claim_bootstrap_key` | Atomic single-use claim (PK insert) |
| `AccountReadRepository` | `list_by_user`, `find_by_id`, `find_by_ids`, `lock_accounts_for_update` | Account queries + row locking |
| `AccountWriteRepository` | `create`, `update`, `soft_delete` | Account mutations (15+ params on create) |
| `TransactionReadRepository` | `list`, `find_by_id`, `list_by_account` | Journal queries with filters |
| `TransactionWriteRepository` | `create`, `update`, `delete` | Journal mutations + balance deltas |
| `AccountService` (trait) | `list`, `get`, `create`, `update`, `delete` | Domain service (accounts) |
| `TransactionService` (struct) | `list`, `get`, `create`, `update`, `delete` | Domain service (transactions) |

All repository methods are `async` via `async-trait`, generic over `Executor` or `&mut Transaction<Postgres>` for pool/tx flexibility.

## Firefly III Compatibility Matrix

| Firefly Surface | Utopia Status | Evidence |
|---|---|---|
| Accounts CRUD + list + type filter | ✅ Implemented | `router.rs` 5 routes + `accounts.rs` handlers |
| Accounts soft-delete | ✅ Implemented | `deleted_at` + partial unique index |
| Accounts extended attributes (21 types, IBAN, etc.) | ✅ Implemented | `types.rs` + migration 0003 |
| Transactions CRUD + list + filters | ✅ Implemented | `transactions.rs` 5 routes |
| Transaction balance updates (atomic) | ✅ Implemented | `lock_accounts_for_update` + `AccountBalanceUpdate` |
| Currencies list | ⚠️ Partial (static 20, no CRUD) | `metadata.rs` CURRENCY_TABLE |
| About / About User | ✅ Implemented | `metadata.rs` handlers |
| Token issuance / revocation | ✅ Implemented | `tokens.rs` + `TokenService` |
| Bootstrap provisioning | ✅ Implemented | `bootstrap_issue_token_handler` + rate limiter |
| Envelope / Pagination / Error shape | ✅ Implemented | `compatibility/*` |
| Decimal string serialization | ⚠️ Partial (2-decimal fixed, JPY mismatch) | `decimal_amount.rs` |
| Budgets | ❌ Not implemented | `budgets.rs` placeholder |
| Categories / Tags / Bills / Piggy Banks | ❌ Not implemented | No routes/schemas/tables |
| Attachments / Search / Bulk / Recurring / Webhooks / Import-Export | ❌ Not implemented | No routes/schemas/tables |
| Link header / X-Total-Count pagination | ❌ Not implemented | Only `meta.pagination` |

## OpenAPI Contract
- File: `openapi.yaml` (OpenAPI 3.0.3, ~1500 lines).
- Defines all 16 business routes + schemas (`FireflyListEnvelopeAccount`, `CreateAccountRequest`, `UpdateAccountRequest`, `FireflyTransactionResource`, etc.) + security (`bearerAuth`) + error responses.
- Known issue: `UpdateAccountRequest` schema duplicated (second `type: object` block overwrites first) — needs fix.
- No budget/category/tag/bill paths defined.

## Middleware & Cross-Cutting Contracts
- **Auth:** `Authorization: Bearer <token>` → `Principal { user_id, token_id }` in request extensions. Validated via SHA256 + Argon2 + `moka` cache.
- **Bootstrap:** `X-Bootstrap-Key: <key>` → constant-time compare, single-use, rate-limited (5/60s).
- **Rate limiting:** `rate_limit_middleware` only on `/api/v1/bootstrap/tokens` — in-memory `HashMap` + `RwLock`, fail-open.
- **Accept negotiation:** `accept_header_middleware` — content negotiation (JSON).
- **Request ID:** `SetRequestId` + `PropagateRequestId` (`x-request-id` UUID).
- **Security headers:** `CSP: default-src 'self'`, `HSTS`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`.
- **Metrics:** `GET /metrics` — Prometheus text format, no auth.
