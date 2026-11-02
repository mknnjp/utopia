# Stakeholder Map

| Stakeholder | Role | Interest / Concern | Authority | Source |
|---|---|---|---|---|
| Solo developer (repository owner) | Developer, product owner, decision-maker | Wants a clear picture of Firefly III compatibility gaps and a prioritized backlog to guide the next implementation wave; owns all scope and priority decisions | Decides scope and priority [Q7] | [Q7][Q2] |
| Future contributors (if any) | Potential collaborators | Will rely on the compatibility matrix and diff table as onboarding / contribution guidance | Influences (no decision authority in this inventory) | [Q2][Q7] |

## Decision-Makers vs. Influencers

| Person / Group | Type | Scope of Authority | Source |
|---|---|---|---|
| Solo developer | Decision-maker | All decisions: inventory scope, comparison baseline (v6 full surface), prioritization criteria, and Top N selection | [Q7] |
| (none) | Influencer | No external influencer is in scope for this inventory | [Q2][Q7] |

## Communication Requirements

| Audience | Channel / Cadence | Content | Source |
|---|---|---|---|
| Solo developer (self) | Repository artifacts (this intent record) | Compatibility matrix, spec diff table, prioritization framework | [Q2][Q3] |
| Future readers / reviewers | Pull request / docs | English-language inventory documents reviewable on GitHub | [Q3] |

No external reporting cadence or cross-team communication is required for this inventory [Q2][Q7].

## Assumptions & Open Questions

None. [Q2][Q7][Q8]

## Sources

- [desc] Initial description: "Firefly III互換APIの現状を棚卸しして、実装済み仕様と本家との差分、今後の優先順位を整理したい。"
- [scope] Workflow-selected scope: `firefly-compat-inventory`.
- [Q2] Q2 Answer: A
- [Q3] Q3 Answer: D (updated from C on 2026-08-29 per consistency review)
- [Q7] Q7 Answer: C
- [Q8] Q8 Answer: A
