# Technology Stack — Utopia

> **Snapshot note:** Versions and technology choices reflect the reverse-engineering snapshot (`f92e948e`, 2026-08-29). Re-check `Cargo.toml` / `package.json` / `Dockerfile` for current versions before upgrading.

## Languages
| Language | Version | Usage |
|---|---|---|
| Rust | 1.73+ (edition 2021), Dockerfile uses 1.88-alpine | Entire API server (`src/*`) |
| TypeScript | (via pnpm, no runtime JS) | `scripts/seed` helpers, `scripts/ci/setup-env.ts`, k6 scripts |
| SQL | Postgres dialect | `migrations/*.sql` (4 migrations) |
| YAML | — | `openapi.yaml`, `docker-compose.yml`, prometheus/loki/promtail configs |

## Runtime & Frameworks
| Technology | Version | Purpose |
|---|---|---|
| Tokio | 1.43.0 (rt-multi-thread, macros, signal) | Async runtime |
| Axum | 0.7.9 | HTTP framework (router, extractors, middleware) |
| Tower | 0.5.2 (util) | Middleware composition |
| Tower-HTTP | 0.6.2 (trace, request-id, set-header) | Request ID, security headers, tracing |

## Data & Persistence
| Technology | Version | Purpose |
|---|---|---|
| Postgres | 17-alpine (testcontainers), production via docker-compose | Primary database |
| SQLx | 0.8.3 (postgres, runtime-tokio-rustls, uuid, chrono, rust_decimal) | DB access, compile-time checked queries, migrations (`sqlx::migrate!`) |
| rust_decimal | 1.36.0 (serde) | Monetary amounts (NUMERIC 20,8) |
| uuid | 1.12.1 (v4, serde) | Primary keys |
| chrono | 0.4.39 (serde) | DateTime<Utc> |

## Security & Auth
| Technology | Version | Purpose |
|---|---|---|
| argon2 | 0.5.3 | Token hashing (memory_cost >=65536, time_cost >=3) |
| sha2 | 0.10.8 | SHA256 for token lookup |
| base64 | 0.22.1 | Token encoding (URL_SAFE_NO_PAD) |
| rand | 0.8.5 | Token generation (OsRng) |
| subtle | 2.6.1 | Constant-time bootstrap key compare |

## Caching & Observability
| Technology | Version | Purpose |
|---|---|---|
| moka | 0.12.10 (future) | Token cache (positive + negative, TTL-based) |
| prometheus | 0.13.4 | Metrics (histograms, counters) + `/metrics` handler |
| tracing | 0.1.41 | Structured logging |
| tracing-subscriber | 0.3.19 (env-filter, json) | JSON log formatting, env filter |

## Serialization & Utilities
| Technology | Version | Purpose |
|---|---|---|
| serde | 1.0.217 (derive) | Serialization |
| serde_json | 1.0.138 | JSON |
| http | 1.2.0 | Header types |
| anyhow | 1.0.95 | Error context |
| thiserror | 2.0.11 | Typed errors |
| async-trait | 0.1.86 | Async repository traits |
| dotenvy | 0.15.7 | Env loading |

## JS Tooling (Dev Only, No Runtime JS)
| Technology | Version | Purpose |
|---|---|---|
| pnpm | (workspace) | Package manager (`pnpm-workspace.yaml`) |
| oxfmt | 0.44.0 | Formatter (`oxfmt.config.ts`, scripts `fmt`, `fmt:check`) |
| oxlint | 1.59.0 | Linter (`oxlint.config.ts`, scripts `lint`, `lint:fix`) |
| oxc | (via oxc.config.ts) | Underlying parser for oxlint/oxfmt |

## Testing
| Technology | Version | Purpose |
|---|---|---|
| cargo test + tokio::test | — | Test runner (async) |
| testcontainers | 0.23.3 | Real Postgres integration tests |
| testcontainers-modules | 0.11.4 (postgres) | Postgres 17-alpine container |
| proptest | 1.6.0 | Property-based tests (decimal, pagination) |

## Infrastructure & Deployment
| Technology | Version | Purpose |
|---|---|---|
| Docker | — | Containerization (`Dockerfile` multi-stage: rust:1.88-alpine → alpine:3.21) |
| Docker Compose | — | Local orchestration (`docker/docker-compose.yml`) |
| Caddy | — | Reverse proxy (`docker/caddy/Caddyfile`) |
| Prometheus | — | Metrics scraping (`docker/prometheus/prometheus.yml`, alert.rules.yml) |
| Grafana | — | Dashboards (`docker/grafana/provisioning/*`) |
| Loki | — | Log aggregation (`docker/loki/loki-config.yml`) |
| Promtail | — | Log shipping (`docker/promtail/promtail-config.yml`) |
| k6 | — | Load testing (`k6/*.ts`, fixtures) |

## Build System
- **Rust:** Cargo (single binary crate `utopia`, edition 2021, rust-version 1.73)
- **JS:** pnpm workspaces (`package.json` + `pnpm-workspace.yaml` + `scripts/seed/package.json`)
- **Migrations:** `sqlx::migrate!` macro (compile-time embedded, 4 files)
- **Docker:** Multi-stage build (builder → runtime), `docker-compose.override.yml` for local dev

## Version Pinning
- All Rust deps pinned via `Cargo.lock` (not shown but present); `Cargo.toml` uses exact or caret versions.
- JS dev deps pinned via `pnpm-lock.yaml` (`oxfmt ^0.44.0`, `oxlint ^1.59.0`).
- No `rust-toolchain.toml` — relies on `rust-version` in Cargo.toml and Dockerfile base image.
