# Utopia

Lightweight, self-hostable personal finance API with partial
compatibility with Firefly-III.

<p align="center">
  <img src="./docs/images/logo.png" alt="logo" width="600">
</p>

## Overview

Utopia implements a subset of the Firefly-III API surface to provide accounts,
token management, transaction journals, and Prometheus metrics. This repository
contains the server implementation, database migrations, and supporting
infrastructure for local development and observability.

## Supported API (high level)

- Accounts API
- Token issuance and management (personal access tokens)
- Bootstrap token issuance (initial provisioning)
- Transactions / journals
- Metadata API (currencies, system info, user profile)
- Prometheus `/metrics` endpoint

See the full API contract in [openapi.yaml](openapi.yaml).

## Quick start

Prerequisites: Docker Engine and Docker Compose (or a Rust toolchain for local
builds).

Start with Docker Compose:

```bash
cp .env.example .env
docker compose -f docker/docker-compose.yml up --build
```

Run locally (development):

```bash
cp .env.example .env
# adjust DATABASE_URL as needed
cargo build
cargo run
```

Verify the service (examples):

```bash
# Check metrics endpoint
curl -fsS http://localhost:3000/metrics | head -n 20

# Check metadata endpoints (requires auth token)
curl -fsS http://localhost:3000/api/v1/currencies | head -n 20
curl -fsS http://localhost:3000/api/v1/about | head -n 20
```

## Configuration

Runtime configuration is provided via environment variables. See
[.env.example](.env.example) for required keys and sensible defaults used for
local development.

## Build & test

```bash
# Build
cargo build
# Run tests
cargo test
# Lint
cargo clippy -- -D warnings
```

## Repository layout (high level)

- `src/` — application source code (server, handlers, core logic)
- `migrations/` — database migrations
- `docker/` — docker-compose and container helpers
- `tests/` — Rust tests (`unit/` for logic, `integration/` for API/DB)
- `compatibility-tests/` — k6 Firefly-III compatibility verification suite
- `results/compatibility/` — k6 output artifacts (JSON reports, gitignored)
- `aidlc/` — AI-DLC workspace (design artifacts, decision records, and method memory)

## Documentation

Architecture, NFRs, and construction plans are generated and maintained under
`aidlc/` as part of the AI-DLC workflow. Run `/aidlc` to start or resume a
workflow; see `AGENTS.md` for harness setup.

## License

This project is licensed under the BSD-3-Clause license. See [LICENSE](LICENSE)
for details.
