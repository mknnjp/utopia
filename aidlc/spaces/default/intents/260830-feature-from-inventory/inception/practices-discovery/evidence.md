# Evidence

> Finalized after the consolidated-summary confirmation.

## Sources Scanned

- `.github/workflows/ci-phase1.yml` — Rust fmt/clippy/build/test + cargo-audit (advisory).
- `.github/workflows/ci-phase2.yml` — k6 compatibility suite (auth/accounts/transactions) against a docker-compose stack on PR + main push.
- `package.json` — `oxfmt` / `oxlint` scripts (`fmt`, `lint`, `lint:fix`, `lint-and-format`).
- `oxlint.config.ts`, `oxfmt.config.ts` — JS/TS lint + format rules (strict type-safety, single quotes, printWidth 120).
- `Cargo.toml` — Rust edition 2021, `rust-version 1.73`; dev-deps include `proptest`, `testcontainers`.
- `Dockerfile` — multi-stage Rust 1.88 build → alpine runtime, exposes 3000.
- `docker/docker-compose.yml` — `utopia-api` + `postgres` + `caddy` (+ observability profile: prometheus/grafana/loki/pgbackrest).
- `git log` / `git branch -a` — `feature/*` branches merged to `main` via PR; trunk-based, no long-lived release branches.

## What the Lead Inferred

- **Branching strategy**: GitHub-Flow-like — short-lived feature branches, PR to `main`, CI must be green.
- **Deployment cadence**: deploy-on-merge to staging; production gated by manual approval (per org.md).
- **Environment topology**: single Dockerized service behind Caddy; Postgres datastore; observability stack available via compose profile.
- **Quality gates**: clippy deny-warnings + fmt check + cargo test in Phase 1; k6 compatibility in Phase 2. cargo-audit is advisory (non-blocking) in Phase 1.

## Interview Decisions

- **Q1 (Branching):** Confirmed — short-lived `feature/*` branches → `main` via PR, CI must be green (GitHub-Flow-like).
- **Q2 (Walking skeleton):** No — skip the skeleton; new features land as ordinary PRs into the running system.
- **Q3 (Coverage floor):** Enforce now — add a Rust coverage tool (cargo-llvm-cov) as a CI gate for the 80% floor.
- **Q4 (Scanning):** Promote `cargo-audit` to a blocking gate and add secret scanning (gitleaks) pre-merge. No SAST/DAST decided yet.
- **Q5 (Prod rollout):** Placeholder — no production deployment target exists yet, so the concrete rollout method is undecided and recorded as a placeholder.

## Independent Review Gaps (resolved)

- 80% Rust coverage floor: now enforced via CI coverage tool (was aspirational).
- cargo-audit: now blocking; secret scanning (gitleaks) added pre-merge.
- JS/TS unit-test framework: still none configured; lint/format only (no decision to add one).
- Production rollout strategy: placeholder until a target environment exists.

## Unresolved Uncertainty

- No explicit team-written `team.md` practices existed before this run; all baselines are now affirmed from config + interview.
- SAST/DAST tooling remains out of scope for now (human decision).
