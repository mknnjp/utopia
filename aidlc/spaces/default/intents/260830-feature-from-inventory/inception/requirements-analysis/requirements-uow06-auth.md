# Requirements — User Authentication & Authorization Construction (UOW-06)

## Intent Analysis

- **User request**: "ユーザー認証・認可の Construction を開始する" — extend the
  auth domain of `utopia` (Rust/Axum, Firefly-III partial-compatible household
  finance API).
- **Request type**: Enhancement / New unit of work (UOW-06 candidate) extending
  the existing auth domain.
- **Scope estimate**: Multiple components (auth core, middleware, cache, config,
  docker topology, tests).
- **Complexity estimate**: Moderate (external IdP delegation lowers
  credential-handling risk, but JWT validation, user sync, and Security Baseline
  compliance add surface).

### Decisions (from `auth-construction-questions.md`)

| Q                                   | Answer                  | Summary                                                                                                                                                                                                        |
| ----------------------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1 Primary scope                    | **C**                   | Adopt **GoTrue** as external authorization server (IdP). It owns registration, password login, OAuth/social, and issues access + refresh JWTs. utopia becomes a resource server validating GoTrue-issued JWTs. |
| Q2 Relation to existing token model | **C (Layered)**         | GoTrue JWTs are a separate credential type mapping to the existing `Principal`, coexisting with PAT + bootstrap key during migration.                                                                          |
| Q3 Firefly-III compatibility        | **A (Hard constraint)** | Data endpoints keep `FireflySingleEnvelope` / `FireflyErrorResponse`. utopia adds no own auth endpoints — only validates Bearer JWTs.                                                                          |
| Q4 Requirements depth               | **A (Standard)**        | Bounded scope; Security Baseline (SEC-001..007) remains blocking.                                                                                                                                              |

---

## Functional Requirements

- **FR-01 (External IdP delegation)**: utopia SHALL NOT issue or store user
  credentials. Authentication is delegated to GoTrue.
- **FR-02 (JWT validation)**: For requests carrying a GoTrue-issued Bearer JWT,
  utopia SHALL validate:
  - signature (via GoTrue JWKS, cached locally),
  - `exp` (expiration),
  - `aud` (audience — must match configured value),
  - `iss` (issuer — must match configured GoTrue issuer URL).
- **FR-03 (Principal mapping)**: On successful validation, utopia SHALL build a
  `Principal` from JWT claims (`sub`, `email`, `role`/`app_metadata`).
- **FR-04 (User sync / JIT provisioning)**: On first authenticated request for
  an unknown `sub`, utopia SHALL create/link a local user record keyed by the
  GoTrue `sub` (external ID). Existing `users` table is extended with an
  external subject column; no password columns are required for GoTrue-sourced
  users.
- **FR-05 (Layered coexistence)**: Existing PAT + bootstrap-key flows SHALL
  remain functional and unchanged. Both credential types resolve to the same
  `Principal`.
- **FR-06 (Firefly compatibility)**: All data endpoints SHALL keep existing
  Firefly-compatible JSON contracts. No new auth endpoints are added to utopia;
  clients obtain tokens directly from GoTrue.
- **FR-07 (JWKS caching)**: utopia SHALL cache GoTrue JWKS (public keys) with a
  bounded TTL to avoid per-request network calls to GoTrue.

---

## Non-Functional Requirements

- **NFR-SEC-01 (Token validation, SEC-08)**: JWT validation SHALL occur
  server-side on every request; invalid/expired/aud/iss-mismatched tokens SHALL
  be rejected with `401` + `FireflyErrorResponse`.
- **NFR-SEC-02 (Constant-time comparison)**: Any local claim/signature
  comparison SHALL use constant-time primitives (consistent with existing
  `subtle` usage).
- **NFR-SEC-03 (Audit logging, SEC-03/SEC-08)**: Authentication failures,
  authorization denials, and JIT provisioning events SHALL be written to the
  existing audit logger (`audit_logger.rs`) with timestamp, request ID, and
  reason — no secrets/PII in logs.
- **NFR-SEC-04 (Rate limiting, SEC-11)**: Public-facing endpoints (including the
  existing bootstrap token route) SHALL retain rate limiting
  (`rate_limiter.rs`). GoTrue itself rate-limits its own login endpoints.
- **NFR-SEC-05 (CORS, SEC-08)**: CORS on authenticated endpoints SHALL be
  restricted to explicitly allowed origins (no wildcard on authenticated
  routes).
- **NFR-SEC-06 (No hardcoded secrets, SEC-12)**: GoTrue JWKS URL, `aud`, `iss`,
  and any shared secrets SHALL be supplied via environment/config (`AppConfig`),
  never hardcoded.
- **NFR-REL-01 (Availability)**: JWKS caching SHALL ensure utopia can validate
  tokens without a synchronous call to GoTrue per request (resilience to GoTrue
  brief outages within cache TTL).
- **NFR-OBS-01 (Metrics, SEC-14)**: JWT validation outcomes (success/failure
  reason) SHALL be emitted via the existing Prometheus metrics (`metrics.rs`).
- **NFR-TST-01 (Property-Based Testing, Partial extension)**: Pure functions —
  notably claim→`Principal` mapping and JWT claim validation logic — SHALL have
  property-based tests.
- **NFR-TST-02 (Integration testing)**: Integration tests SHALL cover the JWT
  validation path (GoTrue-emitted token, or a test JWT signed with a known key /
  mock JWKS) against the running app + PostgreSQL (testcontainers).

---

## Architectural Considerations

- **Resource server pattern**: utopia = OAuth2 Resource Server; GoTrue =
  Authorization Server. Clients (Waterfly-iii) obtain JWTs from GoTrue, send
  `Bearer` to utopia.
- **Minimal change surface**: Reuse existing `auth_middleware`, `Principal`,
  `FireflyErrorResponse`. Introduce a JWT validator adapter alongside the
  existing PAT validator; `TokenCache` is repurposed/extended for JWKS caching.
- **Docker topology**: `docker/docker-compose.yml` gains a GoTrue service (or
  Supabase Auth) with PostgreSQL; utopia's `AppConfig` gains GoTrue JWKS/aud/iss
  settings.
- **Migration**: `0005_*` migration adds external subject column to `users` (and
  any index needed for JIT lookup).

---

## Security Baseline Compliance Summary (Extension ENABLED — blocking)

| Rule                                | Status                                   | Rationale                                                                              |
| ----------------------------------- | ---------------------------------------- | -------------------------------------------------------------------------------------- |
| SEC-01 Encryption at rest/transit   | N/A (no new store; uses existing PG/TLS) | Reuses existing encrypted PostgreSQL + TLS                                             |
| SEC-02 Network intermediary logging | N/A (no new LB/API GW introduced)        | Existing Caddy/Prometheus topology unchanged                                           |
| SEC-03 App-level logging            | Compliant                                | Reuses `audit_logger.rs` + tracing JSON subscriber                                     |
| SEC-04 HTTP security headers        | N/A (utopia serves JSON API, not HTML)   | No HTML-serving endpoints                                                              |
| SEC-05 Input validation             | Compliant                                | JWT parsed/validated before use; existing validators retained                          |
| SEC-06 Least-privilege              | Compliant                                | GoTrue-scoped config; no wildcard grants introduced                                    |
| SEC-07 Restrictive network          | N/A (no new network rules in this unit)  | Docker network scoped as before                                                        |
| SEC-08 App-level access control     | **Compliant (key constraint)**           | Server-side JWT validation every request; CORS restricted; deny-by-default             |
| SEC-09 Hardening                    | Compliant                                | No default creds; generic error responses retained                                     |
| SEC-10 Supply chain                 | Compliant                                | New deps pinned in `Cargo.toml` lockfile                                               |
| SEC-11 Secure design                | **Compliant (key constraint)**           | Auth logic isolated in `core/auth`; rate limiting retained; misuse (bad token) handled |
| SEC-12 Auth & credential mgmt       | Compliant (delegated)                    | Credential storage/MFA/lockout owned by GoTrue; utopia validates only                  |
| SEC-13 Integrity                    | Compliant                                | JWT signature verified via JWKS; no unsafe deserialization                             |
| SEC-14 Alerting/monitoring          | Compliant                                | Validation metrics via existing Prometheus                                             |

No blocking findings. Security Baseline satisfied for the utopia-side scope of
UOW-06.
