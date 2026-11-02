# Requirements Clarification — User Authentication & Authorization Construction

Please answer the following questions to help clarify the scope of the new
"ユーザー認証・認可" (user authentication & authorization) Construction effort.

> **Context**: The project `utopia` is a Rust/Axum backend implementing a
> Firefly-III partially-compatible household finance API. All prior units of
> work (UOW-01..05) are COMPLETE. The existing auth is **token-based only**:
> personal access tokens (PAT) + a one-time bootstrap key. There is currently
> **no** user registration, **no** password/login flow, **no** OAuth2/OIDC, and
> **no** session management. The target capabilities for this Construction are
> "OAuth2 / OpenID Connect Integration" and "Session Management（JWT）". This
> Construction will be a new unit of work (UOW-06 candidate) extending the auth
> domain.

---

## Question 1

What is the primary scope of this Construction effort? (Select all that apply —
but pick the single best primary focus if unsure.)

A) Session management with JWT (issue/refresh/revoke JWT sessions on top of the
existing token model; no password login yet)

B) User registration + password login (username/email + Argon2 password, issue
tokens/sessions) — local credential auth

C) OAuth2 / OpenID Connect integration (delegate auth to an external IdP such as
Google/Auth0; no local passwords)

D) Full auth platform: registration + password login + OAuth2/OIDC + session
management (JWT) together

E) Other (please describe after [Answer]: tag below)

[Answer]: C

> **Decision**: Adopt **GoTrue** (Supabase Auth / Netlify GoTrue) as the
> external authorization server. GoTrue acts as the external IdP: it owns user
> registration, password login, OAuth/Social, and issues access + refresh JWTs.
> utopia becomes a resource server that validates GoTrue-issued JWTs (signature
> via JWKS, plus `exp`/`aud`/`iss`) and maps the `sub` claim to a local
> `Principal`. This covers the "OAuth2/OIDC Integration" and "Session Management
> (JWT)" capabilities without self-hosting credential logic.

## Question 2

How should the new auth relate to the existing token model (personal access
tokens + bootstrap key)?

A) Additive only — new auth mechanisms issue the SAME existing token type
(Principal/TokenRecord); existing PAT + bootstrap flow stays unchanged

B) Replace — new auth becomes the primary mechanism; existing PAT/bootstrap is
deprecated or removed

C) Layered — new sessions (JWT) are a separate credential type that maps to the
existing Principal, coexisting with PATs

D) Other (please describe after [Answer]: tag below)

[Answer]: C

> **Recommended**: Layered coexistence. GoTrue JWTs are validated as a separate
> credential type that resolves to the existing `Principal` model, so the
> current PAT + bootstrap-key flow stays intact during migration. Existing
> clients using PATs are unaffected; new clients use GoTrue-issued JWTs.

## Question 3

Is Firefly-III client compatibility (e.g., Waterfly-iii) a hard constraint for
the new endpoints?

A) Yes — new auth endpoints must follow the existing Firefly-compatible JSON
contracts (FireflySingleEnvelope / FireflyErrorResponse) and not break current
clients

B) No — new auth endpoints can use a clean, purpose-built JSON contract even if
it differs from Firefly conventions

C) Mixed — login/session endpoints can be custom, but token issuance should stay
Firefly-compatible

D) Other (please describe after [Answer]: tag below)

[Answer]: A

> **Recommended**: Firefly-III client compatibility (e.g., Waterfly-iii) is a
> hard constraint. Data endpoints keep the existing `FireflySingleEnvelope` /
> `FireflyErrorResponse` contracts. GoTrue itself exposes the login/registration
> UI and token endpoints, so utopia does not need to add its own auth endpoints
> — it only validates Bearer JWTs. Clients obtain tokens from GoTrue and send
> them to utopia transparently.

## Question 4

What depth of Requirements Analysis do you want before Construction begins?

A) Standard — gather functional + non-functional requirements, then plan and
build (recommended for a security-sensitive domain)

B) Comprehensive — detailed requirements with full traceability and threat
modeling (high-risk auth surface)

C) Minimal — just confirm scope above and start building against existing
patterns

D) Other (please describe after [Answer]: tag below)

[Answer]: A

> **Recommended**: Standard depth. Scope is now well-bounded (external IdP
> delegation + JWT validation adapter + user sync), which is lower-risk than
> self-hosting credential logic. Security Baseline (SEC-001..007) remains a
> blocking constraint and is satisfied by validating signatures, rate limiting,
> audit logging, and metrics on the utopia side. If threat modeling is desired
> later, B can be revisited.
