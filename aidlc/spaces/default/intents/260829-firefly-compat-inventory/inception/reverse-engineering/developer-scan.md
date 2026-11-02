## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - ./openapi.yaml
  - ./Cargo.toml
  - ./package.json
  - ./pnpm-workspace.yaml
  - ./oxlint.config.ts
  - ./oxc.config.ts
  - ./oxfmt.config.ts
  - ./src/main.rs
  - ./src/lib.rs
  - ./src/app.rs
  - ./src/config.rs
  - ./src/api.rs
  - ./src/api/router.rs
  - ./src/api/handlers.rs
  - ./src/api/handlers/accounts.rs
  - ./src/api/handlers/transactions.rs
  - ./src/api/handlers/tokens.rs
  - ./src/api/handlers/metadata.rs
  - ./src/api/middleware.rs
  - ./src/api/middleware/accept_negotiation.rs
  - ./src/api/middleware/rate_limiter.rs
  - ./src/core.rs
  - ./src/core/auth.rs
  - ./src/core/auth/models.rs
  - ./src/core/auth/service.rs
  - ./src/core/auth/validator.rs
  - ./src/core/auth/middleware.rs
  - ./src/core/auth/cache.rs
  - ./src/core/auth/error.rs
  - ./src/core/auth/metrics.rs
  - ./src/core/auth/audit_logger.rs
  - ./src/core/compatibility.rs
  - ./src/core/compatibility/envelope.rs
  - ./src/core/compatibility/error_response.rs
  - ./src/core/compatibility/pagination.rs
  - ./src/core/compatibility/decimal_amount.rs
  - ./src/core/error_mapping.rs
  - ./src/core/error_mapping/mapper.rs
  - ./src/core/persistence.rs
  - ./src/core/persistence/db.rs
  - ./src/core/persistence/repository.rs
  - ./src/modules.rs
  - ./src/modules/accounts.rs
  - ./src/modules/accounts/service.rs
  - ./src/modules/accounts/types.rs
  - ./src/modules/transactions.rs
  - ./src/modules/metadata.rs
  - ./src/modules/budgets.rs
  - ./migrations/0001_initial_schema.sql
  - ./migrations/0002_accounts_schema.sql
  - ./migrations/0003_accounts_extended_schema.sql
  - ./migrations/0004_transactions_schema.sql
  - ./tests/accounts_api_test.rs
  - ./tests/transactions_api_test.rs
  - ./tests/auth_integration_test.rs
  - ./tests/db_integration_test.rs
  - ./tests/core_tests.rs
  - ./tests/core/accounts_query_test.rs
  - ./tests/core/auth_validator_test.rs
  - ./tests/core/decimal_serialization_test.rs
  - ./tests/core/error_mapper_test.rs
  - ./tests/core/firefly_error_contract_test.rs
  - ./tests/core/pagination_test.rs
  - ./tests/core/token_lifecycle_test.rs
  - ./README.md
  - ./Dockerfile
- **Skimmed only**:
  - ./docker/ (docker-compose.yml, Caddyfile, prometheus/loki/grafana configs — infra only)
  - ./k6/ (load test scripts: accounts.ts, auth.ts, transactions.ts, harness.ts — not app logic)
  - ./scripts/seed/ (seed data helpers)
  - ./scripts/ci/setup-env.ts
  - ./aidlc/ (AI-DLC workspace artifacts — out of scope for code scan)
  - ./.aidlc/ (tooling cache)
  - ./target/ (build artifacts)
  - ./k6-results/ (generated output)

### Packages Found
- `utopia` — binary crate — Rust — Firefly III-compatible household finance API (Axum + SQLx + Postgres)
- `scripts/seed` — workspace package — TypeScript (pnpm) — DB seed helpers (referenced in pnpm-workspace.yaml)

### Build System
- **Type**: Cargo (Rust 1.73+, edition 2021) + pnpm (JS tooling)
- **Config Files**: `./Cargo.toml`, `./package.json`, `./pnpm-workspace.yaml`, `./Dockerfile` (multi-stage: rust:1.88-alpine builder → alpine:3.21 runtime), `./docker/docker-compose.yml`, `./migrations/*.sql` (sqlx migrate macro)
- **Build Dependencies**:
  - Rust → Postgres via `sqlx` (compile-time checked queries, runtime-tokio-rustls)
  - `tokio` runtime → `axum` router → `tower`/`tower-http` middleware
  - `moka` cache → auth validator
  - `prometheus` crate → metrics handler
  - JS tooling: `oxfmt 0.44.0` (formatter), `oxlint 1.59.0` (linter) — no app JS build, only lint/format scripts

### APIs Discovered
- **REST (Axum) — 16 routes in `src/api/router.rs` + `openapi.yaml`**:
  - `GET /api/v1/accounts` — list accounts (query: page, limit, type) — FireflyListEnvelopeAccount
  - `POST /api/v1/accounts` — create account — FireflySingleEnvelopeAccount (201)
  - `GET /api/v1/accounts/{id}` — get account
  - `PUT /api/v1/accounts/{id}` — update account
  - `DELETE /api/v1/accounts/{id}` — delete account (204, soft-delete via deleted_at)
  - `GET /api/v1/accounts/{id}/transactions` — list account transactions (page, limit)
  - `GET /api/v1/transactions` — list transactions (page, limit, start, end, type=withdrawal|deposit|transfer)
  - `POST /api/v1/transactions` — create transaction (group_id, transaction_type, description, amount, currency_code, date, source/destination, category, notes, reconciled)
  - `GET /api/v1/transactions/{id}` — get transaction
  - `PUT /api/v1/transactions/{id}` — update transaction
  - `DELETE /api/v1/transactions/{id}` — delete transaction (204)
  - `POST /api/v1/tokens` — issue personal access token (auth required, body: label)
  - `DELETE /api/v1/tokens/{id}` — revoke token
  - `POST /api/v1/bootstrap/tokens` — bootstrap token (X-Bootstrap-Key header, rate-limited, single-use via bootstrap_key_usage table)
  - `GET /api/v1/currencies` — list currencies (static 20-entry table, paginated)
  - `GET /api/v1/about` — system info (version, api_version, php_version=Rust, os, driver=PostgreSQL)
  - `GET /api/v1/about/user` — authenticated user profile
  - `GET /metrics` — Prometheus metrics (no auth, text/plain)
- **Internal traits (repository layer in `src/core/persistence/repository.rs`)**:
  - `TokenReadRepository` / `TokenWriteRepository` / `TokenUpdateRepository` (find_by_sha256, find_by_id, create_token, revoke_token, update_last_used_at)
  - `UserReadRepository` / `UserWriteRepository` (find_by_id, find_by_email, create_user)
  - `BootstrapKeyRepository` (claim_bootstrap_key — atomic single-use)
  - `AccountReadRepository` (list_by_user, find_by_id, find_by_ids, lock_accounts_for_update with SELECT FOR UPDATE)
  - `AccountWriteRepository` (create, update — 15+ params, soft-delete)
  - `TransactionReadRepository` / `TransactionWriteRepository` (list, find, create, update, delete with balance adjustments)
  - `AccountService` trait (list/get/create/update/delete) — `AccountServiceImpl` in `src/modules/accounts/service.rs`
  - `TransactionService` struct in `src/modules/transactions.rs` (list/get/create/update/delete with AccountBalanceUpdate)
- **Compatibility layer (`src/core/compatibility/`)**:
  - `FireflyListEnvelope<T>` / `FireflySingleEnvelope<T>` (envelope.rs) — JSON:API-like envelope with meta.pagination
  - `FireflyErrorResponse` (error_response.rs) — {message, errors: HashMap<String, Vec<String>>}
  - `Paginated<T>` + `PaginationMeta` (pagination.rs) — DEFAULT_PAGE=1, DEFAULT_LIMIT=50, MAX_LIMIT=100, compute_pagination with div_ceil
  - `DecimalAmount` (decimal_amount.rs) — string-serialized Decimal with normalize + 2-decimal formatting

### Frameworks & Libraries
- `axum 0.7.9` — HTTP framework (router, extractors, middleware)
- `tokio 1.43.0` (rt-multi-thread, macros, signal) — async runtime
- `tower 0.5.2` + `tower-http 0.6.2` (trace, request-id, set-header) — middleware stack
- `sqlx 0.8.3` (postgres, runtime-tokio-rustls, uuid, chrono, rust_decimal) — DB access + migrations
- `serde 1.0.217` + `serde_json 1.0.138` — serialization
- `chrono 0.4.39` (serde) — DateTime<Utc>
- `uuid 1.12.1` (v4, serde) — primary keys
- `rust_decimal 1.36.0` (serde) — monetary amounts (NUMERIC 20,8)
- `argon2 0.5.3` — token hashing (memory_cost >=65536, time_cost >=3)
- `sha2 0.10.8` + `base64 0.22.1` + `rand 0.8.5` + `subtle 2.6.1` — token generation, SHA256 lookup, constant-time bootstrap key compare
- `moka 0.12.10` (future) — token cache (positive + negative caches, TTL-based)
- `prometheus 0.13.4` — metrics (auth_validation_latency_ms histogram, counters for auth failures/cache hits/misses, token issue/revoke, rate-limited)
- `tracing 0.1.41` + `tracing-subscriber 0.3.19` (env-filter, json) — structured JSON logging + audit logger (target: "audit")
- `anyhow 1.0.95` + `thiserror 2.0.11` — error handling
- `async-trait 0.1.86` — async repository traits
- `http 1.2.0` — header types
- `dotenvy 0.15.7` — env loading
- Dev: `proptest 1.6.0` (property tests), `testcontainers 0.23.3` + `testcontainers-modules 0.11.4` (postgres 17-alpine integration tests)
- JS dev: `oxfmt 0.44.0`, `oxlint 1.59.0` — lint/format (no runtime JS)

### Test Coverage
- **Test Directories**: `./tests/` (integration) + `./tests/core/` (unit) + inline `#[cfg(test)]` in modules
- **Test Frameworks**: `cargo test` (tokio::test async), `testcontainers` (real Postgres), `proptest` (property-based)
- **Coverage Config**: absent (no tarpaulin/llvm-cov config found)
- **Test Files Found**:
  - `tests/accounts_api_test.rs` — Firefly envelope, pagination, type filter, 401 without bearer, CRUD
  - `tests/transactions_api_test.rs` — transaction CRUD, list with filters, account-scoped listing
  - `tests/auth_integration_test.rs` — bearer validation, bootstrap flow, token lifecycle
  - `tests/db_integration_test.rs` — repository integration
  - `tests/core_tests.rs` + `tests/core/*.rs` — pagination_test, decimal_serialization_test, error_mapper_test, firefly_error_contract_test, accounts_query_test, auth_validator_test, token_lifecycle_test
  - `tests/core/support.rs` — shared test helpers
- **Notable**: Most integration tests are `#[ignore = "requires Docker daemon"]` — require running Docker for Postgres container; CI must have Docker available. No unit test for `budgets.rs` (placeholder).

### Code Quality Indicators
- **Linting**: `oxlint` with strict config (`oxlint.config.ts` — eqeqeq, no-implicit-coercion, typescript strict rules, unicorn) + `oxfmt` for formatting. Rust: `cargo clippy -- -D warnings` per README (no clippy.toml found, relies on default).
- **CI/CD**: No `.github/workflows/` found in snapshot; `scripts/ci/setup-env.ts` exists but not analyzed deeply. `docker/docker-compose.yml` + `Dockerfile` provide local build; no explicit CI pipeline file detected.
- **Documentation**: `README.md` present (overview, supported API high-level, quick start, config, repo layout). `openapi.yaml` is comprehensive (3.0.3, ~1500 lines, all endpoints + schemas). Inline doc comments sparse — service traits have brief docs (e.g., AccountService), but most handlers lack rustdoc. `aidlc/` holds AI-DLC design artifacts (not code docs).
- **Config Validation**: `src/config.rs` validates Argon2 params, bootstrap key length >=16, strict SSL (DATABASE_URL must contain sslmode=require when APP_STRICT_SSL=true), rate limit defaults (5 req/60s).
- **Observability**: Prometheus metrics + JSON tracing + audit logger (security events with actor, event_type, outcome, source_ip, reason_code, request_id). `docker/prometheus/`, `docker/grafana/`, `docker/loki/`, `docker/promtail/` configs present (skimmed).

### Technical Debt Signals
- `src/modules/budgets.rs` is a placeholder (`// Placeholder for UOW-04.`) — Firefly budgets API entirely unimplemented; openapi.yaml has no budget paths.
- Static currency table (`src/modules/metadata.rs` — 20 hardcoded entries, JPY default, no DB table) — Firefly has dynamic currencies with CRUD; Utopia only lists.
- `src/modules/accounts/types.rs` — `ALLOWED_ACCOUNT_TYPES` includes 21 variants but `normalize_account_type` logic not fully visible; `UpdateAccountRequest` has `Option<Option<T>>` for nullable fields (double Option pattern) which is error-prone and lacks clear Firefly parity for PATCH semantics.
- `src/modules/transactions.rs` — `TransactionView.user` is empty string (handler does not resolve principal email into resource); `source_name`/`destination_name` are None (no join to accounts table at query time despite `find_by_ids` existing).
- `src/core/persistence/repository.rs` — `ACCOUNT_COLUMNS` constant duplicated across queries; `AccountWriteRepository::create` takes 15+ positional args (clippy allow too_many_arguments) — builder pattern would be safer.
- `src/api/handlers/metadata.rs` — `parse_page_param`/`parse_limit_param` duplicate pagination logic already in `src/modules/accounts/types.rs` and `src/modules/transactions.rs` (three copies of page/limit parsing).
- `src/api/middleware/rate_limiter.rs` — in-memory HashMap with RwLock, no distributed rate limiting; `fail_open_check_and_count` silently allows requests on non-rate-limit errors (fail-open may hide bugs).
- `src/core/auth/validator.rs` — `tokio::spawn` for `update_last_used_at` is fire-and-forget; failures only increment metric, no retry.
- `src/core/compatibility/decimal_amount.rs` — `format_amount` normalizes then pads to 2 decimals, but Firefly expects variable decimal_places per currency (JPY 0, USD 2) — mismatch for JPY amounts (e.g., "100.00" vs "100").
- Missing Firefly surface: categories, tags, bills, piggy banks, attachments, search, bulk operations, webhooks, data export/import, recurring transactions — none in openapi.yaml or router.
- No pagination `Link` header or `X-Total-Count`; only meta.pagination (Firefly also supports Link headers in some endpoints).
- `openapi.yaml` defines `UpdateAccountRequest` schema twice (duplicate `type: object` block at line ~700) — likely copy-paste error, second block overwrites first.
- Tests require Docker (`#[ignore]`) — no in-memory or mock DB alternative, so `cargo test` without Docker runs only a subset.

## Handoff Summary
- **Intent-relevant finding**: The Firefly III-compatible API is partially implemented — Accounts (CRUD + list with type filter, soft-delete, 21 account types, extended attributes via migration 0003), Transactions (CRUD + list with date/type filters, group_id, balance updates with SELECT FOR UPDATE), Metadata (currencies static 20, about, about/user), and Auth (bearer Argon2+SHA256, moka cache, bootstrap single-use, rate-limited) are all implemented. `openapi.yaml` (~1500 lines) and `src/api/router.rs` (16 routes) are the single source of truth for the spec. Unimplemented features include Budgets (placeholder only), Categories, Tags, Bills, Piggy Banks, Attachments, Search, Bulk operations, Recurring transactions — the majority of Firefly's surface. `src/modules/budgets.rs:1` is empty, currencies have no backing DB table, and transaction user/source_name resolution is incomplete. Prioritization requires a "implemented vs Firefly upstream diff" table, and fixing minor inconsistencies such as the duplicate UpdateAccountRequest schema in openapi.yaml as a starting point. Evidence: `src/modules/budgets.rs:1`, `src/modules/metadata.rs:20-40` (CURRENCY_TABLE), `src/modules/transactions.rs:180-220` (FireflyTransactionResource::from_view with empty user), `openapi.yaml:1-1500` (no budget/category paths), `migrations/0004_transactions_schema.sql:1-50` (transaction_journals table).
- **Risks / follow-up**: Architect must preserve: (1) Firefly envelope contract — `FireflyListEnvelope {data, meta:{pagination:{total,count,per_page,current_page,total_pages}}}` and `FireflyErrorResponse {message, errors}` — validated in `tests/core/firefly_error_contract_test.rs` and `src/core/compatibility/envelope.rs:1-40`; (2) Decimal string serialization via `DecimalAmount` (src/core/compatibility/decimal_amount.rs) — changing format breaks Firefly clients; (3) Auth cache TTLs and Argon2 params from `src/config.rs` (security-sensitive, validated at startup); (4) Soft-delete semantics (deleted_at IS NULL partial unique index in migration 0003) — hard delete would violate Firefly expectations; (5) Transaction balance atomicity via `lock_accounts_for_update` (SELECT FOR UPDATE) in repository.rs — must not be removed. Follow-up: confirm Firefly version to diff against (openapi.yaml claims api_version 1.1.0 but Firefly III is at 6.x), and decide whether to fix openapi.yaml duplicate UpdateAccountRequest schema before inventory report.
