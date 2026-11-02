# Team Practices

> Drafted by the release engineer (lead) for Practices Discovery. Reflects
> evidence from the existing workspace configuration, CI/CD pipelines, and git
> history, finalized after the consolidated-summary confirmation.

## Way of Working

We work on short-lived `feature/*` branches and merge into `main` through pull
requests reviewed on GitHub. `main` is the single long-lived trunk; releases
happen from trunk, not from long-lived release branches. CI runs automatically
on every PR and on every push to `main`, so a branch is expected to be
green before it is merged.

## Walking Skeleton

We do not run a walking-skeleton ceremony for this project. The deployment
pipeline (Docker + docker-compose + Caddy) is already mature and the service
boots end-to-end against Postgres today, so the cost of a thin first slice
outweighs its value at our current maturity. New features land as ordinary PRs
into the existing running system.

## Testing Posture

- **Methodology**: test-after
- **Ordering**: implement each applicable testable layer, then write and run that layer's tests.

The `classic` scope adds an 80% line-coverage floor, **enforced now** via a Rust
coverage tool (e.g. cargo-llvm-cov) wired into CI as a gate. Today the Rust side
uses `cargo test` (unit + integration via testcontainers) and the API surface is
verified with k6 compatibility scripts in CI Phase 2. The JS/TS tooling
(oxlint, oxfmt) is lint/format only — there is no JS unit-test framework
configured yet.

## Deployment

We deploy on merge to staging environments. Production deploys gate on a
separate manual approval — tech lead + product owner sign-off. The application
ships as a Rust binary in a Docker image, orchestrated by docker-compose behind
a Caddy reverse proxy, with Postgres as the datastore. Database migrations live
in `migrations/` and run forward-only.

**Production rollout strategy (placeholder):** no production deployment target
exists yet, so the concrete rollout method (recreate / blue-green / canary) is
undecided and recorded as a placeholder until a target environment is stood up.

## Code Style

We defer to project-level configurations:
- Rust: `cargo fmt` (format) and `cargo clippy -- -D warnings` (lint), enforced in CI Phase 1.
- JS/TS: `oxfmt` (format) and `oxlint` (lint), configured in `oxlint.config.ts` / `oxfmt.config.ts` at repo root, run in CI before merge; failure blocks the PR.
- Naming: language idiomatic (snake_case for Rust, camelCase for JS/TS). No project-wide rename rules beyond the linter.
- **Security scanning**: `cargo-audit` is promoted from advisory to a **blocking** gate in CI, and a secret-scanning step (e.g. gitleaks) runs pre-merge.
