# Requirements Analysis Questions

## Sources

- [desc] Initial description: "Firefly III互換APIの現状を棚卸しして、実装済み仕様と本家との差分、今後の優先順位を整理したい。"
- [scope] Workflow-selected scope: `firefly-compat-inventory`.

## Q1. How should the compatibility matrix be structured and what granularity is expected?

A. Endpoint-level matrix only (method + path → Implemented / Partial / Not Implemented)
B. Endpoint + schema-level matrix (endpoint status plus field-level gaps per resource)
C. Endpoint + schema + behavior matrix (also covering pagination, envelope, error shape, auth)
D. Minimal — endpoint list with status only, no field-level detail
X. Other (please specify)

[Answer]: B

## Q2. Which upstream Firefly III API surface should be the baseline for the gap analysis?

A. All endpoints from the latest stable Firefly III v6 OpenAPI spec (full surface)
B. Only the domains Utopia currently touches (Accounts, Transactions, Currencies, Budgets, Auth)
C. Full v6 surface, but with explicit priority tiers so unimplemented domains are ranked
D. Manually curated subset based on most-used Firefly III client endpoints
X. Other (please specify)

[Answer]: C

## Q3. How should prioritization be scored and what should the Top N output look like?

A. Simple ranked list (P0/P1/P2) based on compatibility importance + business value (from Q6: A,C)
B. Weighted scoring (compatibility importance × business value) with numeric scores and Top 5/10 candidates
C. Effort-aware scoring (compatibility × value ÷ effort) even though effort was not a primary criterion
D. No scoring — just group unimplemented endpoints by domain and let the solo developer decide ad hoc
X. Other (please specify)

[Answer]: A

## Q4. What is the expected handling of known technical debt and partial implementations in the requirements?

A. Document them as explicit requirements / constraints (e.g., DecimalAmount JPY formatting, static currencies, Budgets placeholder)
B. List them only as open questions / tech debt appendix, not as requirements
C. Ignore — focus only on missing endpoints, not on quality of implemented ones
D. Create separate NFRs for each debt item with measurable acceptance criteria
X. Other (please specify)

[Answer]: A

## Q5. What should be explicitly out of scope for this inventory's requirements?

A. Code implementation, DB migrations, and deployment — inventory is documentation only (per Q8)
B. Code implementation plus any new Firefly III v6 endpoints released after the inventory date
C. Only code implementation is out of scope; everything else (including future v6 deltas) is in scope
D. Nothing is out of scope — all gaps should be treated as in-scope requirements
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the requirements artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
