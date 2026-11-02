# Requirements — Firefly III Compatibility Inventory

## Intent Analysis

**Initiative:** Systematic inventory of the current Firefly III-compatible API surface, mapping implemented specs against upstream Firefly III v6, and prioritizing remaining gaps [Q1][Q5].

**Business problem:** Two concrete risks motivate this work: (1) external Firefly III clients may encounter unexpected incompatibilities at integration time due to an unverified compatibility surface [Q1]; (2) without a visualized gap map, the roadmap for the next expansion cannot be prioritized on evidence [Q1]. Internal technical debt and spec ambiguities accumulated during core domain implementation (Accounts, Transactions) further motivate a structured inventory at this planning inflection point [Q4].

**Target customer:** The primary consumer is the internal development team — a solo developer who owns all scope and priority decisions [Q2][Q7]. Deliverables serve as direct input for future implementation planning. External integrators and product-level stakeholders are explicitly out of scope for this inventory's audience [Q2].

**Success definition (all three artifacts required) [Q3]:**
1. Endpoint-level compatibility matrix (Implemented / Partially Implemented / Not Implemented) [Q3][Q5].
2. Spec diff table (`openapi.yaml` vs upstream Firefly III OpenAPI spec) [Q3].
3. Prioritization framework with Top N candidates — criteria: compatibility importance + business value [Q6], ranked P0/P1/P2 [Q3].

**Initiative trigger:** Internal — resolve accumulated technical debt and spec ambiguities before committing to the next feature wave [Q4].

**Scope boundary:** `firefly-compat-inventory` (Minimal depth, 6 stages, inventory-only — no code implementation) [Q8]. Implementation of unimplemented APIs, if pursued, will be handled as a separate follow-up workflow [Q8].

**Comparison baseline:** All API endpoints of the latest stable upstream Firefly III (v6) [Q5].

**Prioritization weighting:** Compatibility importance (primary) + business value (primary) [Q6]; implementation effort is not a primary driver for this inventory.

---

## Functional Requirements

Every requirement carries a stable `FR{n}` ID for downstream traceability. Sub-requirements use `FR{n}.{m}`.

### FR1 — Compatibility Matrix (Endpoint-Level)
Every upstream Firefly III v6 endpoint must be classified against Utopia's current implementation.

- FR1.1 — For each endpoint (`/api/v1/accounts`, `/api/v1/accounts/{id}`, `/api/v1/transactions`, `/api/v1/transactions/{id}`, `/api/v1/currencies`, `/api/v1/about`, `/api/v1/about/user`, `/api/v1/bootstrap/tokens`, `/api/v1/tokens`, `/api/v1/tokens/{id}`, `/metrics`), record: method, path, status (Implemented / Partial / Not Implemented), and notes.
- FR1.2 — Status definitions: Implemented = full CRUD + pagination + envelope; Partial = some operations or fields missing; Not Implemented = no route or schema defined.
- FR1.3 — Cross-reference each endpoint to the corresponding `openapi.yaml` path definition and `src/api/handlers/*.rs` handler.

### FR2 — Spec Diff Table (Schema-Level)
Compare `openapi.yaml` schemas against upstream Firefly III OpenAPI spec.

- FR2.1 — For each resource (`FireflyAccountResource`, `FireflyTransactionResource`, `FireflyCurrencyResource`, `FireflySystemInfoResource`, `FireflyUserResource`), list fields present in upstream but missing or different in Utopia.
- FR2.2 — Note schema-level differences: `currency_decimal_places` (static 2-decimal vs per-currency), `DecimalAmount` formatting, `UpdateAccountRequest` duplicate schema block, pagination meta shape.
- FR2.3 — Note envelope-level differences: `FireflyListEnvelope*` and `FireflySingleEnvelope*` structures, pagination meta (`PaginationMeta`), link headers (`FireflyLink`).

### FR3 — Prioritization Framework
Define and apply evaluation criteria to rank unimplemented endpoints.

- FR3.1 — Criteria: Compatibility importance (frequency of use by Firefly III clients) [Q6] + Business value (alignment with Utopia product goals) [Q6].
- FR3.2 — Apply criteria to all Not Implemented / Partial endpoints from FR1.
- FR3.3 — Produce ranked output: P0 (high importance + high value), P1 (high on one axis), P2 (lower on both). Include Top 5 candidates with justification.

### FR4 — Technical Debt & Partial Implementation Registry
Document known quality issues in implemented endpoints.

- FR4.1 — `DecimalAmount` formats amounts with fixed 2-decimal formatting; JPY (0 decimal places) is not enforced per currency [developer-scan finding].
- FR4.2 — Currency table is static (20 hardcoded entries, no DB table, no CRUD) [developer-scan finding].
- FR4.3 — `Budgets` module (`src/modules/budgets.rs`) is a placeholder — no routes, no schemas [developer-scan finding].
- FR4.4 — `openapi.yaml` `UpdateAccountRequest` has a duplicate `type: object` schema block [developer-scan finding].
- FR4.5 — Pagination parsing duplicated across `metadata.rs`, `accounts/types.rs`, `transactions.rs` [developer-scan finding].
- FR4.6 — Rate limiter is in-memory (`HashMap` + `RwLock`), fail-open, resets on restart [developer-scan finding].
- FR4.7 — Tests require Docker (`testcontainers`) and some are `#[ignore]` [developer-scan finding].

---

## Non-Functional Requirements

Every NFR carries a stable `NFR{n}` ID.

### NFR1 — Documentation Language
All repository-persisted inventory artifacts (`intent-statement.md`, `stakeholder-map.md`, `requirements.md`, and any downstream artifacts) must be written in English [language.md].

### NFR2 — Source Traceability
Every substantive claim in `intent-statement.md` and `stakeholder-map.md` must carry inline source tags (`[desc]`, `[scope]`, `[Q<n>]`, `[memory:M<n>]`) linking back to the questions file or scope definition [intent-capture protocol].

### NFR3 — Artifact Completeness (Pre-Generation Gate)
Before `requirements.md` is generated, the `Consolidated Summary Confirmation` section in the questions file must contain exactly one `[Answer]: Looks correct` or `[Answer]: Request changes` entry, with a matching `SUMMARY_CONFIRMATION_RECORDED` audit receipt [stage-protocol.md §3a].

### NFR4 — Scope Boundary Preservation
The inventory scope (`firefly-compat-inventory`, Minimal depth, 6 stages) excludes code implementation, design ceremony, and operations phases [scope definition]. Any proposal to expand scope must be handled as a separate workflow [Q8].

---

## Constraints

- **Technical:** Brownfield Rust monolith (`Cargo.toml` + `pnpm-workspace.yaml`); single binary (`src/main.rs`); Postgres persistence (`sqlx`); Axum HTTP framework; `openapi.yaml` as contract source of truth.
- **Business:** Solo developer owns all decisions [Q7]; no external stakeholder reporting cadence required [Q2].
- **Organizational:** Trunk-based development (`main`); squash-merge; no long-lived branches [org.md].
- **Scope:** Inventory-only — no code generation, no CI changes, no deployment [scope definition, Q8].

---

## Assumptions

None confirmed. [Q1][Q2][Q3][Q4][Q5][Q6][Q7][Q8]

---

## Out of Scope

- Code implementation of unimplemented APIs (Budgets, Categories, Tags, Bills, etc.) — to be handled in a separate `feature` or `mvp` workflow [Q8].
- Design ceremony (domain-design, units-generation, contract-design, delivery-planning) — excluded by Minimal depth [scope definition].
- Non-functional design (NFR requirements/design) — excluded by scope [scope definition].
- Infrastructure changes, CI pipeline updates, deployment execution, observability setup — excluded by scope [scope definition].
- Performance validation, incident response, feedback optimization — excluded by scope [scope definition].

---

## Open Questions

None remaining after Q1-Q8 confirmation. All eight clarifying questions have been answered with explicit choices; Q3 was updated from C to D (all three artifacts: matrix + diff table + prioritization framework) per consistency review on 2026-08-29.

---

## Sources

- [desc] Initial description: "Take inventory of the current state of the Firefly III compatible API, organize the differences between the implemented specifications and the original, and prioritize what comes next."
- [scope] Workflow-selected scope: `firefly-compat-inventory`.
- [Q1] Q1 Answer: A, B — Compatibility risk + roadmap visibility
- [Q2] Q2 Answer: A — Internal development team
- [Q3] Q3 Answer: D (updated from C) — All three artifacts (matrix + diff + prioritization)
- [Q4] Q4 Answer: D — Technical debt / spec ambiguity resolution
- [Q5] Q5 Answer: A — Full v6 upstream surface
- [Q6] Q6 Answer: A, C — Compatibility importance + business value
- [Q7] Q7 Answer: C — Solo developer (self-decision)
- [Q8] Q8 Answer: A — Scope matches inventory-only boundary
- [memory:org.md] `org.md` > Way of Working, Testing Posture, Deployment
- [memory:team.md] `team.md` > Way of Working, Overconfidence Prevention
- [memory:project.md] `project.md` > Scope Overrides (empty — no overrides)
- [memory:phases/ideation.md] `phases/ideation.md` (framework default)
- [memory:phases/inception.md] `phases/inception.md` (framework default)
- [codekb] `aidlc/spaces/default/codekb/utopia/` — 9 artifacts from reverse-engineering (business-overview, architecture, code-structure, api-documentation, component-inventory, technology-stack, dependencies, code-quality-assessment, reverse-engineering-timestamp)

## Review

**Verdict:** READY
**Reviewer:** aidlc-product-lead-agent
**Date:** 2026-08-29T12:05:37Z
**Iteration:** 1
**Request Challenge:** review:e9964a9f635eeebcf6296c178a241fb5

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | requirements.md > FR1.1 | Endpoint list is hand-picked from `openapi.yaml` rather than auto-derived; should add a note that the matrix tracks the 11 currently-declared routes plus `/metrics` | Optionally mark the endpoint set as "current `openapi.yaml` snapshot" to make scope clear | New |
| R-02 | Minor | requirements.md > FR3.3 | Top 5 P0 candidates are not yet enumerated in the requirements (FR3 mandates the framework, not the actual ranking) | Optionally add a short Top 5 stub now, or defer to a follow-up inventory workflow | New |

### Summary

The requirements cleanly operationalize the intent-capture answers. FR1–FR4 cover the matrix, diff table, prioritization framework, and tech-debt registry. NFR1–NFR4 lock in language, traceability, pre-generation gate, and scope boundary. The Out-of-Scope section makes the inventory-only boundary explicit. All eight clarifying answers are traceable, and the codekb + memory sources are registered. Minor findings only; the artifact is ready for approval.
