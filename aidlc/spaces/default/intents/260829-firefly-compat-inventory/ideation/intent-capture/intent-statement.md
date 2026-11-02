# Intent Statement

## Problem Statement

Utopia aims to provide a Firefly III-compatible household finance API, but there is no systematic inventory of how much of the upstream Firefly III API surface is already implemented, partially implemented, or missing [desc][Q1]. This gap creates two concrete risks: (1) external Firefly III clients may encounter unexpected incompatibilities at integration time [Q1], and (2) without a visualized map of unimplemented areas, the roadmap for the next expansion cannot be prioritized on evidence [Q1]. Accumulated technical debt and spec ambiguities within the codebase further motivate a structured inventory now [Q4].

## Target Customer

The primary consumer of this inventory is the internal development team — in this case a solo developer who owns all decisions and implementation [Q2][Q7]. The deliverables will be used as direct input for future implementation planning and prioritization [Q2]. External integrators and product-level stakeholders are explicitly out of scope for this inventory's audience [Q2].

## Success Metrics

Success is defined as the delivery of all three artifacts together [Q3]:

1. **Endpoint-level compatibility matrix** — every upstream Firefly III v6 endpoint classified as Implemented / Partially Implemented / Not Implemented [Q3][Q5].
2. **Spec diff table** — tabular comparison of `openapi.yaml` against the upstream Firefly III OpenAPI specification [Q3].
3. **Prioritization framework with Top N candidates** — agreed evaluation criteria (compatibility importance and business value [Q6]) applied to rank unimplemented endpoints, with the next implementation candidates explicitly named [Q3][Q6].

The inventory is considered complete when the matrix, diff table, and ranked backlog are documented and reviewable in English [Q3].

## Initiative Trigger

The trigger is internal: the need to resolve technical debt and spec ambiguities that have accumulated during the implementation of core domains such as Accounts and Transactions [Q4]. The inventory is timed as a planning inflection point before committing to the next wave of feature work [Q4][Q1].

## Initial Scope Signal

- **Workflow-selected scope:** `firefly-compat-inventory` (Minimal depth, 6 stages: workspace-scaffold, workspace-detection, state-init, intent-capture, reverse-engineering, requirements-analysis) [scope].
- **User-confirmed product boundary:** Confirmed to proceed with this inventory-only scope; no code implementation is included in this workflow [Q8]. Implementation of unimplemented APIs, if pursued, will be handled as a separate follow-up workflow [Q8].
- **Comparison baseline:** All API endpoints of the latest stable upstream Firefly III (v6) [Q5].
- **Prioritization weighting:** Compatibility importance and business value are the primary criteria [Q6]; implementation effort is not a primary driver for this inventory.

## Assumptions & Open Questions

None. [Q1][Q2][Q3][Q4][Q5][Q6][Q7][Q8]

## Sources

- [desc] Initial description: "Firefly III互換APIの現状を棚卸しして、実装済み仕様と本家との差分、今後の優先順位を整理したい。"
- [scope] Workflow-selected scope: `firefly-compat-inventory`.
- [Q1] Q1 Answer: A, B
- [Q2] Q2 Answer: A
- [Q3] Q3 Answer: D (updated from C on 2026-08-29 per consistency review)
- [Q4] Q4 Answer: D
- [Q5] Q5 Answer: A
- [Q6] Q6 Answer: A, C
- [Q7] Q7 Answer: C
- [Q8] Q8 Answer: A

## Review

**Verdict:** READY
**Reviewer:** aidlc-product-lead-agent
**Date:** 2026-08-29T07:08:57Z
**Iteration:** 1

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/260829-firefly-compat-inventory/ideation/intent-capture/intent-statement.md > Success Metrics | Success metrics list three artifacts but out-of-scope boundary is implicit | Optionally add explicit out-of-scope statement for clarity | New |

### Summary

Intent statement is well-grounded with all claims traceable to Q1-Q8 and scope. No assumptions remain. Stakeholder map correctly identifies solo developer as sole decision-maker. Ready for approval.
