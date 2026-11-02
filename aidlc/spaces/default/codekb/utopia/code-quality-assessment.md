# Code Quality Assessment — Utopia

> **Snapshot note:** This assessment reflects the codebase state at the time of reverse-engineering (see `reverse-engineering-timestamp` in this `codekb/` directory). Statements such as "no CI pipeline detected" describe the snapshot, not necessarily the current repository. Re-validate against the live tree before acting on gaps.

## Test Coverage
- **Test directories:** `tests/` (integration) + `tests/core/` (unit) + inline `#[cfg(test)]` in modules.
- **Frameworks:** `cargo test` with `#[tokio::test]` async, `testcontainers` (postgres:17-alpine), `proptest` (property-based).
- **Coverage config:** Absent — no `tarpaulin`, `llvm-cov`, or `cargo-llvm-cov` config found.
- **Test files:**
  - `tests/accounts_api_test.rs` — Firefly envelope, pagination, type filter, 401 without bearer, CRUD
  - `tests/transactions_api_test.rs` — transaction CRUD, list with filters, account-scoped listing
  - `tests/auth_integration_test.rs` — bearer validation, bootstrap flow, token lifecycle
  - `tests/db_integration_test.rs` — repository integration
  - `tests/core_tests.rs` — integration-test entry point that pulls in `tests/core/support.rs`, which declares `mod` for each unit test module below
  - `tests/core/support.rs` — shared helpers + module declarations: `accounts_query_test`, `auth_validator_test`, `decimal_serialization_test`, `error_mapper_test`, `firefly_error_contract_test`, `pagination_test`, `token_lifecycle_test`
  - `tests/core/support.rs` — shared helpers
- **Notable gaps:**
  - Most integration tests are `#[ignore = "requires Docker daemon"]` — require Docker; `cargo test` without Docker runs only a subset (unit + non-ignored).
  - No unit test for `src/modules/budgets.rs` (placeholder).
  - No coverage gate or report generation in CI.
  - No in-memory/mock DB alternative — cannot run full suite in environments without Docker.

## Linting & Formatting
- **JS:** `oxlint 1.59.0` (strict: eqeqeq, no-implicit-coercion, typescript strict, unicorn) + `oxfmt 0.44.0` — configs `oxlint.config.ts`, `oxc.config.ts`, `oxfmt.config.ts`. Scripts: `pnpm lint`, `lint:fix`, `fmt`, `fmt:check`, `lint-and-format`.
- **Rust:** `cargo clippy -- -D warnings` per README (no `clippy.toml`, relies on defaults). No `rustfmt.toml` — uses default `rustfmt`.
- **Gaps:** No CI enforcement file found (no `.github/workflows/` in snapshot); lint/format are manual or via `scripts/ci/setup-env.ts` (not deeply analyzed).

## CI/CD
- **Pipeline files:** None found in the reverse-engineering snapshot (no `.github/workflows/` at snapshot time). `scripts/ci/setup-env.ts` exists but not analyzed deeply. Re-check the live tree — a pipeline may have been added since the snapshot.
- **Docker:** `Dockerfile` (multi-stage: rust:1.88-alpine builder → alpine:3.21 runtime) + `docker/docker-compose.yml` + `docker-compose.override.yml` provide local build/run.
- **Migrations:** `sqlx::migrate!` embedded, run at startup via `create_pool`.
- **Gaps (snapshot):** No explicit CI pipeline detected at snapshot time; no automated test/lint/coverage gate; `cargo test` requires Docker for full suite — any CI must provision Docker.

## Documentation Quality
- **README.md:** Present — overview, supported API high-level, quick start (Docker + local), config, repo layout, build & test, license (BSD-3-Clause). Adequate for onboarding.
- **openapi.yaml:** Comprehensive (OpenAPI 3.0.3, ~1500 lines, all 16 business routes + schemas + security + error responses) — contract source of truth. Known issue: duplicate `UpdateAccountRequest` schema block (second `type: object` overwrites first).
- **Inline docs:** Sparse — service traits have brief docs (e.g., `AccountService`), but most handlers lack `rustdoc`. No crate-level or module-level documentation.
- **AI-DLC artifacts:** `aidlc/` holds design artifacts (not code docs) — out of scope for code quality.
- **Gaps:** No `CONTRIBUTING.md`, no `ARCHITECTURE.md` (prior to this inventory), no API changelog.

## Code Style & Conventions
- **Rust:** `snake_case` files, `PascalCase` types, `SCREAMING_SNAKE` constants. Handler suffix `*_handler`, repo suffix `*Repository` + `Pg*Repository` impl. `anyhow` + `thiserror` for errors, `async-trait` for repos.
- **Config validation:** `AppConfig::validate()` enforces Argon2 minima, bootstrap key length, strict SSL — good practice.
- **Observability:** Prometheus metrics + JSON tracing + audit logger (target: "audit") with actor/event_type/outcome/source_ip/reason_code/request_id — well-structured.
- **Gaps:** Pagination parsing duplicated 3×, `AccountWriteRepository::create` with 15+ positional args, `TransactionService` lacks trait vs `AccountService` trait — inconsistencies.

## Technical Debt Register

> **Classification note:** Entries #1 and #12 are *unimplemented Firefly surface* (absent routes/schemas — tracked as FR1 "Not Implemented", out of this inventory's scope per NFR4), not quality defects in shipped code. The remaining entries are *technical debt* in implemented code. "High" severity on #1/#12 signals compatibility coverage gap, not a bug.

| # | Location | Debt | Severity | Impact |
|---|---|---|---|---|
| 1 | `src/modules/budgets.rs:1` | Placeholder — budgets API entirely unimplemented (**unimplemented surface**, not a code defect) | High | Firefly compat gap; openapi has no budget paths |
| 2 | `src/modules/metadata.rs:20-40` | Static 20-entry currency table, no DB, no CRUD | Medium | Firefly has dynamic currencies; JPY decimal_places mismatch |
| 3 | `src/core/compatibility/decimal_amount.rs` | `format_amount` pads to 2 decimals always — JPY should be 0 | Medium | Breaks JPY amounts ("100.00" vs "100") |
| 4 | `src/modules/transactions.rs:180-220` | `TransactionView.user` empty, `source_name`/`destination_name` None | Medium | Incomplete Firefly resource; no join to accounts (see `architecture.md` → Improvement Opportunities: "Transaction resource enrichment") |
| 5 | `src/core/persistence/repository.rs` | `ACCOUNT_COLUMNS` duplicated, `create` with 15+ args | Medium | Error-prone, clippy allow too_many_arguments |
| 6 | `src/api/handlers/metadata.rs` + `accounts/types.rs` + `transactions.rs` | Pagination parsing duplicated 3× | Low | Maintenance burden, drift risk |
| 7 | `src/api/middleware/rate_limiter.rs` | In-memory HashMap+RwLock, fail-open, no distribution | Medium | Not multi-replica safe; hides bugs |
| 8 | `src/core/auth/validator.rs` | `tokio::spawn` fire-and-forget `update_last_used_at` | Low | No retry, only metric on failure |
| 9 | `openapi.yaml:~700` | Duplicate `UpdateAccountRequest` schema block | Low | Second block overwrites first — contract error |
| 10 | `src/modules/accounts/types.rs` | `Option<Option<T>>` double Option for nullable fields | Low | Error-prone PATCH semantics |
| 11 | Tests | `#[ignore]` requires Docker — no mock/in-memory alternative | Medium | `cargo test` without Docker is partial; CI must have Docker |
| 12 | Missing surface | Categories, tags, bills, piggy banks, attachments, search, bulk, recurring, webhooks, import/export (**unimplemented surface**, not a code defect) | High | Majority of Firefly surface absent — prioritization needed |
| 13 | `src/api/handlers/metadata.rs` | No `Link` header or `X-Total-Count` — only `meta.pagination` | Low | Firefly clients may expect Link headers |

## Quality Gates (Observed)
- No coverage threshold, no clippy gate, no format gate in CI (not found).
- `cargo clippy -- -D warnings` and `pnpm lint-and-format` are documented but not enforced via pipeline file.
- `sqlx` compile-time checks require `DATABASE_URL` at build — no `sqlx-data.json` offline mode detected (relevant to Recommendation #6: a committed `sqlx-data.json` would let CI/tests run without a live Postgres).

## Recommendations (for prioritization)
1. Fix `openapi.yaml` duplicate schema (low effort, high correctness).
2. Unify pagination parsing into `core/compatibility/pagination.rs` (low effort).
3. Introduce `CreateAccountParams` builder for `AccountWriteRepository::create` (medium effort).
4. Move currencies to DB table with per-currency `decimal_places` and fix `DecimalAmount` (medium effort).
5. Resolve transaction resource enrichment (join accounts for names, principal email for user) (medium effort).
6. Add `cargo test` without Docker path (mock repos or `sqlx` offline) and add coverage config (medium effort).
7. Add CI pipeline (`.github/workflows/`) with clippy, fmt check, test (with Docker), and openapi lint (medium effort).
8. Decide budgets scope — implement or explicitly 501 in openapi (requires product decision).
9. Inventory remaining Firefly surface (categories/tags/bills/etc.) and prioritize per intent — this inventory is the input.
