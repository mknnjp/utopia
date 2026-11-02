# Intent Capture Questions

## Sources

- [desc] Initial description: "Firefly III互換APIの現状を棚卸しして、実装済み仕様と本家との差分、今後の優先順位を整理したい。"
- [scope] Workflow-selected scope: `firefly-compat-inventory`.

## Q1. What business problem does this inventory aim to solve?

What is the background for understanding the current Firefly III-compatible API and prioritizing future development?

A. It is unclear how compatible the existing implementation is with upstream Firefly III, posing a risk of failures when integrating with external clients
B. Before adding new features, we want to visualize unimplemented areas and build a roadmap
C. Documentation is scattered and we want a shared understanding of implementation status within the team
D. We need a diff-management foundation to keep up with upstream Firefly III version upgrades
E. Not yet defined

[Answer]: A, B

## Q2. Who will use the results of this inventory? (Customer / Audience)

A. Internal development team — as input for future implementation planning and prioritization
B. External integrators (Firefly III client apps, etc.) — for compatibility verification
C. Product owner / decision-maker — for investment and scope decisions
D. All of the above (dev team + external integrators + decision-makers)
E. Not identified

[Answer]: A

## Q3. What defines success and what deliverables are expected?

A. An endpoint-level compatibility matrix (implemented / not implemented / partially implemented) and a prioritized backlog are documented
B. Differences between openapi.yaml and the upstream Firefly III OpenAPI spec are organized in tabular form
C. Prioritization criteria (usage frequency / effort / compatibility importance) are agreed and the top N next candidates are decided
D. All of A + B + C
E. Not yet defined

[Answer]: D — Updated from C on 2026-08-29 per consistency review. A/B are prerequisite intermediate artifacts for C; all three are in scope.

## Q4. Why is this inventory needed now? (Trigger)

A. Core features such as Accounts/Transactions in Utopia are largely implemented and we are at a planning inflection point for the next expansion
B. Compatibility inquiries and bug reports from external parties have increased
C. We need to respond to upstream Firefly III spec changes or version upgrades
D. We want to resolve technical debt and spec ambiguities within the team
E. Not applicable

[Answer]: D

## Q5. Which upstream Firefly III version and scope should be the comparison baseline?

A. All API endpoints of the latest stable version (v6)
B. Limited to the domains Utopia currently covers (Accounts, Transactions, Budgets, Categories, etc.)
C. 1:1 comparison of the endpoint set already defined in openapi.yaml against the upstream spec
D. Manual inventory of endpoints listed in the official Firefly III docs (firefly-iii.org/api)
E. Not yet defined — please propose a recommended approach

[Answer]: A

## Q6. What criteria should drive prioritization?

A. Compatibility importance — prioritize endpoints most frequently used by Firefly III clients
B. Implementation effort — prioritize low-effort items that yield large compatibility gains
C. Business value — prioritize domains directly tied to Utopia's product goals
D. Balanced score across A + B + C (weighted scoring)
E. Not yet defined

[Answer]: A, C

## Q7. Who are the key stakeholders and decision-makers?

A. The development team (committers to this repository) leads, with the product owner making the final prioritization call
B. Development team and representative(s) of external integrators decide jointly
C. Solo project — I decide everything myself
D. Not identified — please help organize
E. Not applicable

[Answer]: C

## Q8. Does the workflow-selected scope `firefly-compat-inventory` match the intended boundary?

This workflow was started with `firefly-compat-inventory` (Minimal depth, 6 stages: intent-capture / reverse-engineering / requirements-analysis). This is an inventory-only scope with no code implementation — deliverables are documentation/analysis only.

A. Yes, it matches — proceed with this inventory-only scope
B. No, I want to include implementation of unimplemented APIs, so switch to a broader scope (feature/mvp, etc.)
C. I plan to start the implementation phase as a separate workflow after the inventory, so this scope is fine
D. I don't understand what the scope means — please explain
E. Other (please describe…)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
