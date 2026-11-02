# Code Structure — Utopia

## Repository Layout
```
utopia/
├── src/
│   ├── main.rs                 # Binary entry (tokio::main, build_app, serve)
│   ├── lib.rs                  # Crate root re-exports
│   ├── app.rs                  # AppState + build_app (pool, cache, services, router)
│   ├── config.rs               # AppConfig::from_env + validate
│   ├── api.rs                  # api module root
│   ├── core.rs                 # core module root
│   ├── modules.rs              # modules root
│   ├── api/
│   │   ├── router.rs           # build_router (16 routes + middleware stack)
│   │   ├── handlers.rs         # handlers mod
│   │   ├── middleware.rs       # middleware mod
│   │   ├── handlers/
│   │   │   ├── accounts.rs     # 6 handlers (list/create/get/update/delete + list transactions)
│   │   │   ├── transactions.rs # 5 handlers (list/create/get/update/delete)
│   │   │   ├── tokens.rs       # 3 handlers (issue, revoke, bootstrap)
│   │   │   └── metadata.rs     # 3 handlers (currencies, about, about/user)
│   │   └── middleware/
│   │       ├── accept_negotiation.rs
│   │       └── rate_limiter.rs # In-memory HashMap+RwLock, fail-open
│   ├── core/
│   │   ├── auth.rs             # auth mod
│   │   ├── compatibility.rs    # compatibility mod
│   │   ├── error_mapping.rs    # error_mapping mod
│   │   ├── persistence.rs      # persistence mod
│   │   ├── auth/
│   │   │   ├── models.rs       # Principal, TokenRecord, UserRecord
│   │   │   ├── service.rs      # TokenService (issue, validate, bootstrap)
│   │   │   ├── validator.rs    # token validation + Argon2 verify + cache
│   │   │   ├── middleware.rs   # auth_middleware (Bearer → Principal)
│   │   │   ├── cache.rs        # TokenCache (moka positive+negative)
│   │   │   ├── error.rs        # AuthError
│   │   │   ├── metrics.rs      # PrometheusMetrics + /metrics handler
│   │   │   └── audit_logger.rs # AuditLogger (target: "audit")
│   │   ├── compatibility/
│   │   │   ├── envelope.rs     # FireflyListEnvelope / FireflySingleEnvelope
│   │   │   ├── error_response.rs # FireflyErrorResponse {message, errors}
│   │   │   ├── pagination.rs   # Paginated<T>, PaginationMeta, compute_pagination
│   │   │   └── decimal_amount.rs # DecimalAmount (string-serialized Decimal)
│   │   ├── error_mapping/
│   │   │   └── mapper.rs       # DomainError → (StatusCode, FireflyErrorResponse)
│   │   └── persistence/
│   │       ├── db.rs           # create_pool (PgPool)
│   │       └── repository.rs   # All repository traits + Pg* impls
│   └── modules/
│       ├── accounts.rs         # re-exports
│       ├── accounts/
│       │   ├── service.rs      # AccountService trait + AccountServiceImpl
│       │   └── types.rs        # AccountListQuery, Create/Update requests, FireflyAccountResource
│       ├── transactions.rs     # TransactionService, TransactionListQuery, Create/Update requests
│       ├── metadata.rs         # CurrencyEntry, CURRENCY_TABLE (20), FireflyCurrencyResource
│       └── budgets.rs          # Placeholder (UOW-04)
├── migrations/
│   ├── 0001_initial_schema.sql # users, personal_access_tokens, bootstrap_key_usage
│   ├── 0002_accounts_schema.sql# accounts table
│   ├── 0003_accounts_extended_schema.sql # extended columns, partial unique index, triggers
│   └── 0004_transactions_schema.sql # transaction_journals + indexes + trigger
├── tests/
│   ├── accounts_api_test.rs
│   ├── transactions_api_test.rs
│   ├── auth_integration_test.rs
│   ├── db_integration_test.rs
│   ├── core_tests.rs          # entry point: pulls in `tests/core/support.rs`
│   └── core/
│       ├── support.rs          # shared helpers + `mod` declarations for each test module
│       ├── accounts_query_test.rs
│       ├── auth_validator_test.rs
│       ├── decimal_serialization_test.rs
│       ├── error_mapper_test.rs
│       ├── firefly_error_contract_test.rs
│       ├── pagination_test.rs
│       └── token_lifecycle_test.rs
├── openapi.yaml                # OpenAPI 3.0.3, ~1500 lines, contract source of truth
├── Cargo.toml                  # Rust manifest (utopia binary crate)
├── package.json                # pnpm JS tooling (oxfmt, oxlint scripts)
├── pnpm-workspace.yaml
├── oxlint.config.ts / oxc.config.ts / oxfmt.config.ts
├── Dockerfile                  # multi-stage: rust:1.88-alpine → alpine:3.21
├── docker/                     # compose, Caddy, prometheus/loki/grafana/promtail
├── k6/                         # load tests (accounts, auth, transactions, harness)
├── scripts/
│   ├── seed/                   # DB seed helpers (TypeScript)
│   └── ci/setup-env.ts
└── aidlc/                      # AI-DLC workspace (out of scope for app)
```

> **Snapshot note:** This document reflects the repository state at the reverse-engineering snapshot (`reverse-engineering-timestamp.md`: commit `f92e948e`, 2026-08-29). Route counts, file lists, and dependency graphs describe that snapshot — re-validate against the live tree before relying on them.

## Package / Crate Organization
| Package | Type | Language | Purpose |
|---|---|---|---|
| `utopia` (root `Cargo.toml`) | binary crate | Rust | Entire API server — single crate, no workspace members for Rust; router defines 18 routes (16 business + `/metrics` + `/api/v1/accounts/{id}/transactions`) |
| `scripts/seed` (`pnpm-workspace.yaml`) | workspace package | TypeScript | DB seed helpers (not part of runtime) |

The Rust crate is **not** a Cargo workspace — all Rust code lives in one crate with `src/` modules. `pnpm-workspace.yaml` only groups JS tooling (`oxfmt`, `oxlint`) and `scripts/seed`.

## Module Classification

| Module | Classification | Responsibility |
|---|---|---|
| `src/main.rs`, `src/app.rs`, `src/config.rs` | Bootstrap / Infra | Env config, pool/cache/metrics wiring, `AppState`, server startup |
| `src/api/router.rs` | API boundary | Route table (18 routes: 16 business + `/metrics` + account-scoped transactions), middleware layering, `AppState` injection |
| `src/api/handlers/*` | API boundary | HTTP handlers: parse request, call service, shape Firefly envelope, map errors |
| `src/api/middleware/*` | Cross-cutting | Accept negotiation, rate limiting (bootstrap only), request ID, security headers |
| `src/core/auth/*` | Core / Security | Token lifecycle, validation, caching, audit, metrics |
| `src/core/compatibility/*` | Core / Contract | Firefly envelope, pagination, decimal string, error response |
| `src/core/persistence/*` | Core / Data | DB pool, repository traits + Postgres impls |
| `src/core/error_mapping/*` | Core / Contract | DomainError → HTTP status + FireflyErrorResponse |
| `src/modules/accounts/*` | Domain | Account business rules, validation, Firefly resource shaping |
| `src/modules/transactions.rs` | Domain | Transaction business rules, balance updates, filtering |
| `src/modules/metadata.rs` | Domain | Static currency table, about/user resources |
| `src/modules/budgets.rs` | Domain (stub) | Placeholder — no logic |

## Code Patterns

### 1. Repository Pattern with Async Traits
- Traits: `TokenReadRepository`, `AccountReadRepository`, `TransactionWriteRepository`, etc. in `src/core/persistence/repository.rs`.
- Impls: `PgTokenRepository`, `PgAccountRepository`, `PgTransactionRepository` — all take `Executor` generic or `&mut Transaction<Postgres>` so callers can pass `&PgPool` or `&mut Transaction`.
- Aggregated in `Repositories` struct (`src/core/persistence/db.rs` or `repository.rs`) and held in `AppState`.

### 2. Service Trait + Impl
- `AccountService` trait with `AccountServiceImpl` (`src/modules/accounts/service.rs`) — `Arc<dyn AccountService>` in `AppState` for testability.
- `TransactionService` is a concrete struct (no trait) — inconsistency; accounts uses trait, transactions does not.

### 3. Firefly Compatibility Layer
- `FireflyListEnvelope<T>` / `FireflySingleEnvelope<T>` wrap domain resources; `FireflyListEnvelope::from_paginated` computes `PaginationMeta` via `compute_pagination`.
- `DecimalAmount` newtype serializes `Decimal` as string with 2-decimal formatting.
- `FireflyErrorResponse { message, errors: HashMap<String, Vec<String>> }` — Firefly-compatible error shape.

### 4. Middleware Stack (tower)
- `build_router` layers: `accept_header_middleware` (outer), `SetRequestId`/`PropagateRequestId`, security headers (`CSP`, `HSTS`, `nosniff`, `DENY`, `referrer-policy`), then per-router `auth_middleware` (protected) and `rate_limit_middleware` (bootstrap).
- `auth_middleware` extracts `Authorization: Bearer`, validates via `TokenService` + `TokenCache`, injects `Principal` into `request.extensions()`.

### 5. Error Handling
- `anyhow` + `thiserror` for internal errors; `DomainError` enum for business errors mapped to HTTP via `error_mapping::mapper`.
- `AuthError` separate for auth flow.

### 6. Configuration Validation
- `AppConfig::from_env` reads 12 env vars, `validate()` enforces Argon2 minima, bootstrap key length, strict SSL (`sslmode=require`).

### 7. Testing Patterns
- `#[tokio::test]` async tests; `testcontainers` (postgres:17-alpine) for integration tests — most are `#[ignore = "requires Docker daemon"]`.
- `proptest` for property-based tests (decimal, pagination).
- `tests/core/support.rs` shared helpers.

## File Naming & Conventions
- Rust: `snake_case` files, `PascalCase` types, `SCREAMING_SNAKE` constants (`ALLOWED_ACCOUNT_TYPES`, `CURRENCY_TABLE`, `ACCOUNT_COLUMNS`).
- Handlers: `*_handler` suffix (`list_accounts_handler`, `create_transaction_handler`).
- Repositories: `*Repository` trait + `Pg*Repository` impl.
- Migrations: `000N_*.sql` sequential, `IF NOT EXISTS` idempotent.

## Notable Structural Issues
- Pagination parsing duplicated in 3 places (`metadata.rs`, `accounts/types.rs`, `transactions.rs`) — should be unified in `core/compatibility/pagination.rs`.
- `AccountWriteRepository::create` takes 15+ positional args — builder/struct param would be safer.
- `TransactionService` lacks trait abstraction unlike `AccountService` — inconsistent test seam.
- `openapi.yaml` duplicate `UpdateAccountRequest` schema block (copy-paste error).
