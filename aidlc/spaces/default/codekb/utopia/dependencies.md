# Dependencies — Utopia

> **Snapshot note:** Dependency versions and usage reflect the reverse-engineering snapshot (`f92e948e`, 2026-08-29). Re-check `Cargo.toml` / `package.json` for current versions before acting on upgrade recommendations.

## External Dependencies (Cargo — `Cargo.toml`)

### Runtime
| Crate | Version | Purpose | Used In |
|---|---|---|---|
| `axum` | 0.7.9 | HTTP framework | `api/router.rs`, `api/handlers/*`, `app.rs` |
| `tokio` | 1.43.0 (rt-multi-thread, macros, signal) | Async runtime | `main.rs`, `app.rs`, `core/auth/*` |
| `tower` | 0.5.2 (util) | Middleware composition | `api/router.rs` |
| `tower-http` | 0.6.2 (trace, request-id, set-header) | Request ID, security headers | `api/router.rs` |
| `sqlx` | 0.8.3 (postgres, runtime-tokio-rustls, uuid, chrono, rust_decimal) | DB access + migrations | `core/persistence/*`, `modules/*`, `app.rs` |
| `serde` | 1.0.217 (derive) | Serialization | All modules |
| `serde_json` | 1.0.138 | JSON | `api/handlers/*`, `core/compatibility/*` |
| `chrono` | 0.4.39 (serde) | DateTime<Utc> | `modules/*`, `core/persistence/*`, `core/compatibility/*` |
| `uuid` | 1.12.1 (v4, serde) | Primary keys | `core/persistence/*`, `modules/*` |
| `rust_decimal` | 1.36.0 (serde) | Monetary amounts | `core/compatibility/decimal_amount.rs`, `modules/*` |
| `argon2` | 0.5.3 | Token hashing | `core/auth/service.rs`, `core/auth/validator.rs` |
| `sha2` | 0.10.8 | SHA256 lookup | `core/auth/service.rs` |
| `base64` | 0.22.1 | Token encoding | `core/auth/service.rs` |
| `rand` | 0.8.5 | Token generation | `core/auth/service.rs` |
| `subtle` | 2.6.1 | Constant-time compare | `core/auth/service.rs` (bootstrap) |
| `moka` | 0.12.10 (future) | Token cache | `core/auth/cache.rs` |
| `prometheus` | 0.13.4 | Metrics | `core/auth/metrics.rs` |
| `tracing` | 0.1.41 | Structured logging | `core/auth/*`, `app.rs` |
| `tracing-subscriber` | 0.3.19 (env-filter, json) | JSON log formatting | `main.rs`, `app.rs` |
| `anyhow` | 1.0.95 | Error context | `config.rs`, `app.rs` |
| `thiserror` | 2.0.11 | Typed errors | `core/auth/error.rs`, `core/persistence/repository.rs` |
| `async-trait` | 0.1.86 | Async traits | `core/persistence/repository.rs` |
| `http` | 1.2.0 | Header types | `api/middleware/*`, `core/auth/*` |
| `dotenvy` | 0.15.7 | Env loading | `main.rs`, `config.rs` |

### Dev
| Crate | Version | Purpose |
|---|---|---|
| `proptest` | 1.6.0 | Property-based tests |
| `testcontainers` | 0.23.3 | Postgres integration tests |
| `testcontainers-modules` | 0.11.4 (postgres) | Postgres 17-alpine container |

### JS Dev (pnpm — `package.json`)
| Package | Version | Purpose |
|---|---|---|
| `oxfmt` | 0.44.0 | Formatter |
| `oxlint` | 1.59.0 | Linter |

## External Services & Infrastructure
| Dependency | Type | Usage | Required |
|---|---|---|---|
| Postgres | Database | Primary store (users, tokens, accounts, journals) | Yes (DATABASE_URL) |
| Caddy | Reverse proxy | TLS termination, forwarding to Axum | Docker compose only |
| Prometheus | Metrics | Scrapes `/metrics` | Optional (observability) |
| Grafana | Dashboards | Visualizes Prometheus/Loki | Optional |
| Loki + Promtail | Log aggregation | Ships JSON logs | Optional |

## Internal Cross-Package / Cross-Module Dependencies

```
app.rs
├── core/persistence (create_pool, Repositories)
├── core/auth (TokenService, TokenCache, AuditLogger, PrometheusMetrics)
├── modules/accounts (AccountServiceImpl)
├── api/router (build_router)
└── config (AppConfig)

api/router.rs
├── api/handlers/accounts, transactions, tokens, metadata
├── api/middleware (accept_negotiation, rate_limiter)
├── core/auth/middleware (auth_middleware)
└── core/auth/metrics (metrics_handler)

api/handlers/accounts.rs
├── modules/accounts (AccountService)
├── modules/transactions (TransactionService — for account-scoped list)
├── core/compatibility (envelope, pagination, decimal)
└── core/error_mapping (DomainError → HTTP)

api/handlers/transactions.rs
├── modules/transactions (TransactionService)
├── core/compatibility
└── core/error_mapping

api/handlers/tokens.rs
├── core/auth/service (TokenService)
└── core/error_mapping

api/handlers/metadata.rs
├── modules/metadata (CURRENCY_TABLE)
└── core/compatibility

core/auth/service.rs
├── core/persistence/repository (Token/User/Bootstrap repos)
├── core/auth/cache (TokenCache)
├── core/auth/metrics (PrometheusMetrics)
└── config (AppConfig — Argon2 params, bootstrap key)

core/auth/middleware.rs
├── core/auth/service + cache (validate)
└── core/auth/models (Principal)

modules/accounts/service.rs
├── core/persistence/repository (AccountRead/Write)
├── core/compatibility (pagination, decimal)
└── core/error_mapping

modules/transactions.rs
├── core/persistence/repository (TransactionRead/Write, AccountRead for locking)
├── core/compatibility
└── core/error_mapping

core/compatibility/* — no internal deps except serde/rust_decimal/chrono
core/error_mapping — depends on core/compatibility/error_response
core/persistence — depends only on sqlx/async-trait/uuid/chrono/rust_decimal
```

## Dependency Health & Risks
- **No Cargo workspace** — single crate, so no inter-crate version conflicts; all deps resolved together.
- **sqlx compile-time checks** require `DATABASE_URL` at build time (or `sqlx prepare`); CI must provide it or use `SQLX_OFFLINE=true` with `sqlx-data.json` (not present — potential CI gap).
- **moka** (0.12.10) is pre-1.0 — API may change; pinned via Cargo.lock.
- **testcontainers** requires Docker daemon — `cargo test` without Docker only runs non-ignored unit tests.
- **JS tooling** has no runtime dependency on Rust — isolated via pnpm workspaces.
- **No direct Firefly III dependency** — compatibility is contract-level (envelope/pagination), not library-level.
