# Architecture — Utopia

## System Overview
Utopia is a **monolithic Rust API server** (Axum + Tokio + SQLx + Postgres) that exposes a Firefly III-compatible JSON API. All business logic lives in a single binary (`src/main.rs` → `src/app.rs` → `src/api/router.rs`). Persistence is Postgres via `sqlx` with compile-time checked queries and `sqlx::migrate!` migrations. There is no separate frontend, worker, or microservice — deployment is a single container behind Caddy (reverse proxy) with Prometheus/Loki/Grafana for observability.

The architecture is **layered + modular monolith**:
- **API layer** (Axum handlers + router) — HTTP boundary, request parsing, Firefly envelope shaping.
- **Core layer** (`src/core/*`) — cross-cutting concerns: auth, compatibility (envelope/pagination/decimal/error), persistence (repositories), error mapping.
- **Modules layer** (`src/modules/*`) — domain modules (accounts, transactions, metadata, budgets stub) owning business rules and service traits.
- **Infra layer** — config, DB pool, cache, metrics, audit logger, rate limiter.

## Architectural Style
- **Style:** Modular monolith, layered (API → Core/Modules → Persistence). Evidence: single `utopia` binary crate, single `build_router` in `src/api/router.rs`, shared `AppState` injected via `Arc`, no inter-service RPC.
- **Communication:** In-process function calls + async traits (`async-trait`); HTTP is the only external boundary. No message queue, no event bus.
- **Data access:** Repository pattern with async traits (`TokenReadRepository`, `AccountReadRepository`, etc.) backed by `PgPool`/`Transaction<Postgres>`. `Repositories` struct aggregates all `Pg*Repository` impls.
- **Auth:** Bearer token (SHA256 lookup + Argon2 verify) with `moka` cache (positive + negative) and `Principal` extension injection via `auth_middleware`.
- **Compatibility:** Explicit Firefly III contract layer (`src/core/compatibility/*`) — envelope, pagination, decimal string, error response — kept separate from domain types so Firefly parity can be reasoned about in one place.

## Component Relationships

```mermaid
graph TD
    Client[Firefly III Client / curl] --> Caddy[Caddy Reverse Proxy]
    Caddy --> Router[Axum Router<br/>src/api/router.rs]
    Router --> MW{Middleware Stack}
    MW --> Accept[Accept Negotiation]
    MW --> ReqId[Request ID<br/>Set/Propagate]
    MW --> SecHeaders[Security Headers<br/>CSP/HSTS/nosniff]
    MW --> AuthMW[Auth Middleware<br/>src/core/auth/middleware.rs]
    MW --> RateLimit[Rate Limiter<br/>bootstrap only]

    AuthMW --> TokenService[TokenService<br/>src/core/auth/service.rs]
    TokenService --> Cache[TokenCache<br/>moka]
    TokenService --> RepoAuth[PgToken/User/Bootstrap Repos]
    AuthMW --> Principal[Principal<br/>user_id + token_id]

    Router --> Handlers[Handlers<br/>accounts / transactions / tokens / metadata]
    Handlers --> Services[Domain Services<br/>AccountServiceImpl<br/>TransactionService]
    Services --> Repo[Repositories<br/>PgAccount / PgTransaction]
    Repo --> Pool[(Postgres<br/>PgPool)]

    Handlers --> Compat[Compatibility Layer<br/>Envelope / Pagination / Decimal / ErrorResponse]
    Handlers --> ErrorMap[Error Mapper<br/>DomainError → HTTP]

    AppState[AppState<br/>config + repos + cache + metrics + audit] -.-> Router
    AppState -.-> Handlers
    AppState -.-> TokenService

    Metrics[Prometheus /metrics] --> PrometheusScrape[Prometheus]
    Tracing[JSON Tracing + Audit Logger] --> Loki[Loki]
    PrometheusScrape --> Grafana[Grafana]
    Loki --> Grafana
```

```mermaid
graph LR
    subgraph Core
        Auth[auth]
        Compat[compatibility]
        Persist[persistence]
        ErrMap[error_mapping]
    end
    subgraph Modules
        Accounts[accounts]
        Transactions[transactions]
        Metadata[metadata]
        Budgets[budgets stub]
    end
    subgraph API
        Router[router]
        Handlers[handlers]
        Middleware[middleware]
    end
    Config[config] --> App[app.rs build_app]
    App --> Router
    Router --> Middleware
    Middleware --> Auth
    Handlers --> Accounts
    Handlers --> Transactions
    Handlers --> Metadata
    Accounts --> Persist
    Transactions --> Persist
    Auth --> Persist
    Auth --> Cache[moka]
    Handlers --> Compat
    Handlers --> ErrMap
```

## Data Flow

### Request lifecycle (authenticated route)
1. Caddy terminates TLS, forwards to Axum.
2. Global layers: `accept_header_middleware` (content negotiation), `SetRequestId`/`PropagateRequestId`, security headers.
3. `auth_middleware` extracts `Authorization: Bearer <token>`, SHA256-hashes, checks `TokenCache` (positive/negative), falls back to `TokenService::validate` (DB lookup + Argon2 verify), injects `Principal` into request extensions, spawns fire-and-forget `update_last_used_at`.
4. Handler parses query/body, delegates to domain service (`AccountServiceImpl`, `TransactionService`), which uses repository traits against `PgPool`/`Transaction`.
5. Service returns domain result or `DomainError`; handler maps via `error_mapping::mapper` to `FireflyErrorResponse` with appropriate HTTP status.
6. Response is wrapped in `FireflyListEnvelope` or `FireflySingleEnvelope` with `meta.pagination` where applicable; `DecimalAmount` serializes amounts as strings.

### Transaction balance flow
`create/update/delete` transaction → `TransactionService` opens DB transaction → `lock_accounts_for_update` (`SELECT ... FOR UPDATE` on `accounts`) → insert/update/delete `transaction_journals` → apply `AccountBalanceUpdate` deltas → commit. Atomicity is DB-transaction scoped.

### Bootstrap flow
`POST /api/v1/bootstrap/tokens` with `X-Bootstrap-Key` → `rate_limit_middleware` (in-memory `HashMap` + `RwLock`, fail-open) → `TokenService::bootstrap_issue` → constant-time compare of key hash → `claim_bootstrap_key` (atomic single-use via `bootstrap_key_usage` PK) → create user if needed → issue token.

## Key Design Decisions

| Decision | Rationale | Consequence / Trade-off |
|---|---|---|
| Single binary monolith (Axum) | Minimal ops, Firefly compat surface is small, team size favors monolith | Easy to reason about; no distributed transactions; scaling is vertical + replica behind Caddy |
| Repository traits with `async-trait` + `Executor` generic | Testability (mock repos), compile-time SQL via `sqlx`, transaction vs pool flexibility | Verbose trait surface (15+ methods), `create` with 15+ positional args — builder would be safer |
| `moka` cache for tokens (positive + negative) | Avoid Argon2 on every request (expensive) | Cache invalidation on revoke is TTL-based; negative cache prevents DB hammer on invalid tokens |
| Firefly envelope/pagination/decimal as separate `core/compatibility` | Isolate Firefly contract so drift is visible and testable | Duplication of pagination parsing (3 copies) — should be unified |
| Soft-delete for accounts (`deleted_at` + partial unique index) | Firefly expects soft-delete; preserves name uniqueness for active only | Queries must always filter `deleted_at IS NULL`; hard delete would break Firefly clients |
| `SELECT FOR UPDATE` for balance updates | Correctness under concurrent transaction writes | Holds row locks for duration of DB transaction — contention under high write concurrency |
| In-memory rate limiter for bootstrap | Simplicity, no Redis dependency | Not distributed; fail-open on errors may hide bugs; resets on restart |
| Static currency table (20 entries) | Fast, no migration needed for MVP | No CRUD, no per-currency decimal_places enforcement (JPY 0 vs USD 2 mismatch) |
| `openapi.yaml` as contract source of truth | Single place for Firefly compat surface, ~1500 lines | Duplicate `UpdateAccountRequest` schema block — needs fix; budgets etc. absent |

## Interaction Diagrams

### Sequence: List Accounts (authenticated)

```mermaid
sequenceDiagram
    participant C as Client
    participant R as Router + Middleware
    participant A as auth_middleware
    participant TC as TokenCache (moka)
    participant TS as TokenService
    participant DB as Postgres
    participant H as list_accounts_handler
    participant S as AccountServiceImpl
    participant Repo as PgAccountRepository
    participant Env as FireflyListEnvelope

    C->>R: GET /api/v1/accounts?page=1&limit=50&type=asset<br/>Authorization: Bearer <token>
    R->>A: auth_middleware
    A->>TC: get(token_sha256)
    alt cache hit (positive)
        TC-->>A: Principal
    else cache miss
        A->>TS: validate(token)
        TS->>DB: find_by_sha256 + Argon2 verify
        DB-->>TS: TokenRecord + UserRecord
        TS->>TC: insert positive
        TS-->>A: Principal
    end
    A->>H: inject Principal, next()
    H->>S: list(query, principal.user_id)
    S->>Repo: list_by_user(pool, user_id, filter, pagination)
    Repo->>DB: SELECT ... FROM accounts WHERE user_id=$1 AND deleted_at IS NULL ...
    DB-->>Repo: rows
    Repo-->>S: Vec<AccountRecord> + total
    S-->>H: Paginated<Account>
    H->>Env: FireflyListEnvelope::from_paginated
    Env-->>C: 200 {data:[...], meta:{pagination:{total,count,per_page,current_page,total_pages}}}
```

### Sequence: Create Transaction (with balance locking)

```mermaid
sequenceDiagram
    participant C as Client
    participant H as create_transaction_handler
    participant S as TransactionService
    participant DB as Postgres
    participant AR as PgAccountRepository
    participant TR as PgTransactionRepository

    C->>H: POST /api/v1/transactions {group_id, type, amount, source_id, dest_id...}
    H->>S: create(req, principal)
    S->>DB: BEGIN
    S->>AR: lock_accounts_for_update(tx, [source_id, dest_id])
    AR->>DB: SELECT ... FOR UPDATE
    DB-->>AR: locked rows
    S->>TR: create(tx, journal)
    TR->>DB: INSERT INTO transaction_journals ...
    S->>AR: apply balance deltas (AccountBalanceUpdate)
    AR->>DB: UPDATE accounts SET current_balance = current_balance + $delta ...
    S->>DB: COMMIT
    S-->>H: TransactionRecord
    H-->>C: 201 {data: FireflyTransactionResource}
```

### Sequence: Bootstrap Token Issuance (rate-limited, single-use)

```mermaid
sequenceDiagram
    participant C as Client
    participant RL as rate_limit_middleware
    participant H as bootstrap_issue_token_handler
    participant TS as TokenService
    participant DB as Postgres

    C->>RL: POST /api/v1/bootstrap/tokens<br/>X-Bootstrap-Key: <key>
    RL->>RL: check_and_count(ip/key) HashMap+RwLock
    alt over limit
        RL-->>C: 429 Too Many Requests
    else allowed
        RL->>H: next()
        H->>TS: bootstrap_issue(key, label)
        TS->>DB: claim_bootstrap_key(tx, key_hash) — PK insert, single-use
        alt already claimed
            DB-->>TS: false
            TS-->>C: 401/409
        else claimed
            TS->>DB: create_user if not exists + create_token
            DB-->>TS: TokenRecord
            TS-->>C: 201 {token: <raw>}
        end
    end
```

### Flow: Error Mapping

```mermaid
flowchart TD
    Handler[Handler] --> Service[Service / Repo]
    Service -->|Ok| Envelope[Firefly Envelope<br/>200/201]
    Service -->|DomainError| Mapper[error_mapping::mapper]
    Mapper -->|Validation| E422[422 FireflyErrorResponse<br/>message + errors map]
    Mapper -->|NotFound| E404[404]
    Mapper -->|Unauthorized| E401[401]
    Mapper -->|Conflict| E409[409]
    Mapper -->|Internal| E500[500]
    Envelope --> Client
    E422 --> Client
    E404 --> Client
    E401 --> Client
    E409 --> Client
    E500 --> Client
```

## Improvement Opportunities
- **Unify pagination parsing** — three copies (`metadata.rs`, `accounts/types.rs`, `transactions.rs`) → single `core/compatibility::pagination::parse` helper.
- **Builder for `AccountWriteRepository::create`** — 15+ positional args is error-prone; introduce `CreateAccountParams` struct.
- **Distributed rate limiting** — replace in-memory `HashMap` with Redis or Postgres-backed limiter if multi-replica deploys are planned; fix fail-open to fail-closed for non-rate-limit errors.
- **Currency model** — move `CURRENCY_TABLE` to DB table with CRUD and enforce `decimal_places` per currency in `DecimalAmount::format_amount`.
- **Transaction resource enrichment** — resolve `user` (principal email) and `source_name`/`destination_name` via join or `find_by_ids` instead of empty/None.
- **OpenAPI fix** — deduplicate `UpdateAccountRequest` schema (second `type: object` block overwrites first).
- **Budgets module** — implement or explicitly mark as out-of-scope in `openapi.yaml` with 501.
- **Cache invalidation** — on `revoke_token`, evict positive cache entry immediately instead of waiting for TTL.
