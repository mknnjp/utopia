# Component Inventory — Utopia

> **Snapshot note:** This inventory reflects the repository state at commit `f92e948e` (2026-08-29, see `reverse-engineering-timestamp.md`). Component counts, route numbers, and dependency graphs describe that snapshot — re-validate before acting.

## Overview
Complete inventory of logical components (code ownership boundaries) in the Utopia Rust monolith. Each component owns its files, responsibilities, and public surface. Dependencies are in-process calls via `AppState` / traits.

> **Note:** Component numbers (1-17) are independent of route counts (18 routes). Component 16 is "Migrations"; the "16 business routes" refers to the API surface, not this component number.

## Components

### 1. Bootstrap / Config (`src/main.rs`, `src/config.rs`, `src/app.rs`)
- **Responsibility:** Process entry, env config loading/validation, `AppState` assembly (pool, cache, metrics, services), server startup.
- **Key files:** `src/main.rs`, `src/config.rs`, `src/app.rs`, `src/lib.rs`
- **Public surface:** `AppConfig::from_env()`, `build_app() -> Router`, `AppState` struct
- **Dependencies:** `core/persistence` (create_pool), `core/auth` (TokenService, TokenCache, AuditLogger, PrometheusMetrics), `modules/accounts` (AccountServiceImpl), `api/router` (build_router)
- **Dependents:** None (root)

### 2. API Router (`src/api/router.rs`)
- **Responsibility:** Route table (18 routes: 16 business + `/metrics` + `/api/v1/accounts/{id}/transactions`), middleware layering, `AppState` injection via `Arc`.
- **Key files:** `src/api/router.rs`
- **Public surface:** `build_router(state: Arc<AppState>) -> Router` (18 routes: 16 business + `/metrics` + `/api/v1/accounts/{id}/transactions`)
- **Dependencies:** `api/handlers/*`, `api/middleware/*`, `core/auth/middleware`, `core/auth/metrics`
- **Dependents:** `app.rs`

### 3. API Handlers — Accounts (`src/api/handlers/accounts.rs`)
- **Responsibility:** HTTP boundary for accounts: parse query/body, call `AccountService`, shape `FireflyListEnvelope`/`FireflySingleEnvelope`, map `DomainError`.
- **Key files:** `src/api/handlers/accounts.rs`
- **Public surface:** `list_accounts_handler`, `create_account_handler`, `get_account_handler`, `update_account_handler`, `delete_account_handler`, `list_account_transactions_handler`
- **Dependencies:** `modules/accounts` (AccountService), `modules/transactions` (for account-scoped listing), `core/compatibility` (envelope, pagination, decimal), `core/error_mapping`
- **Dependents:** `api/router`

### 4. API Handlers — Transactions (`src/api/handlers/transactions.rs`)
- **Responsibility:** HTTP boundary for transaction journals: parse filters/body, call `TransactionService`, shape Firefly resources.
- **Key files:** `src/api/handlers/transactions.rs`
- **Public surface:** `list_transactions_handler`, `create_transaction_handler`, `get_transaction_handler`, `update_transaction_handler`, `delete_transaction_handler`
- **Dependencies:** `modules/transactions`, `core/compatibility`, `core/error_mapping`
- **Dependents:** `api/router`

### 5. API Handlers — Tokens (`src/api/handlers/tokens.rs`)
- **Responsibility:** Token issuance/revocation and bootstrap provisioning HTTP boundary.
- **Key files:** `src/api/handlers/tokens.rs`
- **Public surface:** `issue_token_handler`, `revoke_token_handler`, `bootstrap_issue_token_handler`
- **Dependencies:** `core/auth/service` (TokenService), `core/error_mapping`
- **Dependents:** `api/router`

### 6. API Handlers — Metadata (`src/api/handlers/metadata.rs`)
- **Responsibility:** Currencies (static list, paginated), about, about/user endpoints.
- **Key files:** `src/api/handlers/metadata.rs`
- **Public surface:** `list_currencies_handler`, `get_about_handler`, `get_about_user_handler`
- **Dependencies:** `modules/metadata` (CURRENCY_TABLE, all_currencies), `core/compatibility` (envelope, pagination)
- **Dependents:** `api/router`

### 7. API Middleware (`src/api/middleware/*`)
- **Responsibility:** Cross-cutting HTTP concerns: accept negotiation, rate limiting (bootstrap only), request ID, security headers (via tower-http layers in router).
- **Key files:** `src/api/middleware/accept_negotiation.rs`, `src/api/middleware/rate_limiter.rs`, `src/api/middleware.rs`
- **Public surface:** `accept_header_middleware`, `rate_limit_middleware`, `RateLimitState`
- **Dependencies:** `app::AppState` (RateLimitState), `tower-http`
- **Dependents:** `api/router`

### 8. Core — Auth (`src/core/auth/*`)
- **Responsibility:** Token lifecycle (issue, validate, revoke, bootstrap), Argon2 hashing, SHA256 lookup, moka cache (positive+negative), Principal injection, audit logging, Prometheus metrics.
- **Key files:** `src/core/auth/models.rs`, `service.rs`, `validator.rs`, `middleware.rs`, `cache.rs`, `error.rs`, `metrics.rs`, `audit_logger.rs`
- **Public surface:** `TokenService`, `TokenCache`, `Principal`, `TokenRecord`/`UserRecord`, `auth_middleware`, `AuthError`, `PrometheusMetrics`, `AuditLogger`
- **Dependencies:** `core/persistence/repository` (Token/User/Bootstrap repos), `config::AppConfig`, `moka`, `argon2`, `sha2`, `prometheus`, `tracing`
- **Dependents:** `api/middleware` (auth_middleware), `api/handlers/tokens`, `app.rs`

### 9. Core — Compatibility (`src/core/compatibility/*`)
- **Responsibility:** Firefly III contract shapes: list/single envelope, pagination, decimal string serialization, error response.
- **Key files:** `src/core/compatibility/envelope.rs`, `pagination.rs`, `decimal_amount.rs`, `error_response.rs`, `src/core/compatibility.rs`
- **Public surface:** `FireflyListEnvelope<T>`, `FireflySingleEnvelope<T>`, `Paginated<T>`, `PaginationMeta`, `compute_pagination`, `DecimalAmount`, `FireflyErrorResponse`
- **Dependencies:** `serde`, `rust_decimal`, `chrono`
- **Dependents:** `api/handlers/*`, `modules/*`, `core/error_mapping`

### 10. Core — Persistence (`src/core/persistence/*`)
- **Responsibility:** DB pool creation, repository traits + Postgres implementations, transaction handling.
- **Key files:** `src/core/persistence/db.rs`, `repository.rs`, `src/core/persistence.rs`
- **Public surface:** `create_pool`, `Repositories`, `PgTokenRepository`, `PgUserRepository`, `PgBootstrapKeyRepository`, `PgAccountRepository`, `PgTransactionRepository`, all `*Repository` traits, `RepoError`, `TransactionFilter`, `AccountBalanceUpdate`
- **Dependencies:** `sqlx` (PgPool, Transaction), `async-trait`, `uuid`, `chrono`, `rust_decimal`
- **Dependents:** `core/auth`, `modules/accounts`, `modules/transactions`, `app.rs`

### 11. Core — Error Mapping (`src/core/error_mapping/mapper.rs`)
- **Responsibility:** Map `DomainError` (and `AuthError`) to HTTP status + `FireflyErrorResponse`.
- **Key files:** `src/core/error_mapping/mapper.rs`, `src/core/error_mapping.rs`
- **Public surface:** `map_domain_error`, `DomainError` enum
- **Dependencies:** `core/compatibility/error_response`, `axum::http::StatusCode`
- **Dependents:** `api/handlers/*`

### 12. Modules — Accounts (`src/modules/accounts/*`)
- **Responsibility:** Account domain logic: validation (21 types, name uniqueness via partial index), CRUD, pagination, Firefly resource shaping.
- **Key files:** `src/modules/accounts/service.rs`, `types.rs`, `src/modules/accounts.rs`
- **Public surface:** `AccountService` trait, `AccountServiceImpl`, `AccountListQuery`, `CreateAccountRequest`, `UpdateAccountRequest`, `FireflyAccountResource`
- **Dependencies:** `core/persistence/repository` (AccountRead/Write), `core/compatibility` (pagination, decimal), `core/error_mapping`
- **Dependents:** `api/handlers/accounts`, `app.rs`

### 13. Modules — Transactions (`src/modules/transactions.rs`)
- **Responsibility:** Transaction journal domain logic: validation (withdrawal/deposit/transfer), CRUD, filtering (date/type), per-account listing, atomic balance updates with `SELECT FOR UPDATE`.
- **Key files:** `src/modules/transactions.rs`
- **Public surface:** `TransactionService`, `TransactionListQuery`, `CreateTransactionRequest`, `UpdateTransactionRequest`, `FireflyTransactionResource`, `TransactionView`
- **Dependencies:** `core/persistence/repository` (TransactionRead/Write, AccountRead for locking), `core/compatibility`, `core/error_mapping`
- **Dependents:** `api/handlers/transactions`, `api/handlers/accounts` (account-scoped list)

### 14. Modules — Metadata (`src/modules/metadata.rs`)
- **Responsibility:** Static currency table (20 entries, JPY default), about/system info, user profile resources.
- **Key files:** `src/modules/metadata.rs`
- **Public surface:** `CurrencyEntry`, `CURRENCY_TABLE`, `all_currencies`, `FireflyCurrencyResource`, `FireflyAboutResource`
- **Dependencies:** `core/compatibility` (envelope, pagination), `chrono`, `uuid`
- **Dependents:** `api/handlers/metadata`

### 15. Modules — Budgets (`src/modules/budgets.rs`)
- **Responsibility:** Placeholder for future budgets domain. No logic, no routes, no tables. Note: "UOW-04" reference in source is an unverified internal tag — treat as unimplemented surface (FR1 "Not Implemented"), not a tracked work item.
- **Key files:** `src/modules/budgets.rs` (single comment)
- **Public surface:** None
- **Dependencies:** None
- **Dependents:** None

### 16. Migrations (`migrations/*.sql`)
- **Responsibility:** Schema evolution via `sqlx::migrate!` — users, tokens, bootstrap_key_usage, accounts (base + extended), transaction_journals, indexes, triggers.
- **Key files:** `migrations/0001_initial_schema.sql`, `0002_accounts_schema.sql`, `0003_accounts_extended_schema.sql`, `0004_transactions_schema.sql`
- **Public surface:** SQL DDL (tables, indexes, triggers)
- **Dependencies:** Postgres (`pgcrypto`, `gen_random_uuid()`)
- **Dependents:** `core/persistence` (migrate macro), `app.rs` (pool)

### 17. Observability & Infra (`docker/*`, `src/core/auth/metrics.rs`, `src/core/auth/audit_logger.rs`)
- **Responsibility:** Prometheus metrics exposition, JSON tracing, audit logging (target: "audit"), Grafana/Loki/Promtail/Prometheus configs, Caddy reverse proxy.
- **Key files:** `src/core/auth/metrics.rs`, `audit_logger.rs`, `docker/docker-compose.yml`, `docker/caddy/Caddyfile`, `docker/prometheus/*`, `docker/grafana/*`, `docker/loki/*`, `docker/promtail/*`
- **Public surface:** `GET /metrics`, `AuditLogger`, `PrometheusMetrics`, JSON logs
- **Dependencies:** `prometheus`, `tracing`, `tracing-subscriber`
- **Dependents:** `app.rs`, `core/auth`

## Dependency Graph (simplified)

> **Note:** This graph is a high-level simplification. Detailed per-component dependencies (e.g., `api/handlers/accounts` depends on `modules/transactions` for account-scoped listings; `core/auth` depends on `core/persistence` repos) are listed in each component's `Dependencies` / `Dependents` fields above. See `architecture.md` for the full interaction diagrams (sequence diagrams for list accounts, create transaction, bootstrap token, and error mapping flow).

```
Bootstrap/Config → API Router → Handlers → Modules → Persistence → Postgres
                ↘ Middleware → Core/Auth → Persistence
                ↘ Core/Compatibility (used by Handlers + Modules)
                ↘ Core/ErrorMapping (used by Handlers)
Migrations → Postgres (via sqlx migrate)
Observability → Prometheus/Grafana/Loki (sidecar)
```

## Ownership Notes
- Each entity is owned by exactly one component: `accounts` table by Modules/Accounts + Persistence/Account repo; `transaction_journals` by Modules/Transactions + Persistence/Transaction repo; `personal_access_tokens`/`users`/`bootstrap_key_usage` by Core/Auth + Persistence/Token/User/Bootstrap repos.
- `AppState` is the composition root — owns `Arc` references to all services/repos/cache/metrics.
- No shared mutable state except `RateLimitState` (Arc<RwLock<HashMap>>) and `TokenCache` (moka concurrent).
