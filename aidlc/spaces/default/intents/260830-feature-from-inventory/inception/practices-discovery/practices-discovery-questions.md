# Practices Discovery — Interview

Evidence from CI/CD config, linter/formatter settings, and git history suggests
the answers below. Please confirm or correct them. Your intent is the source of
truth; the configuration only suggests, it does not decide.

## Q1 — Branching & merge flow
The repo uses short-lived `feature/*` branches merged to `main` via pull
request, with CI required green before merge. Is that your team's actual flow?

- A. Yes — feature branches to `main` via PR, CI must be green (GitHub-Flow-like)
- B. No — we use a different strategy (e.g. GitFlow with release branches, trunk-based direct pushes)
- X. Other (please specify)

[Answer]: A. Yes — feature branches to `main` via PR, CI must be green (GitHub-Flow-like)

## Q2 — Walking skeleton (thin end-to-end slice first?)
A walking skeleton is a minimal version that runs the whole way through, built
first to prove the pieces connect before real features go in. The existing
pipeline already boots the service end-to-end against Postgres, so a skeleton
ceremony may add little. Should new features still start from a thin
end-to-end slice?

- A. No — skip the skeleton; new features land as ordinary PRs into the running system
- B. Yes — build a thin end-to-end slice first for each significant feature
- X. Other (please specify)

[Answer]: A. No — skip the skeleton; new features land as ordinary PRs into the running system

## Q3 — Test coverage floor
The `classic` scope calls for an 80% line-coverage floor, but no coverage tool
is wired into CI today. How should we treat that floor?

- A. Enforce it now — add a Rust coverage tool (e.g. cargo-llvm-cov) as a CI gate
- B. Keep it as a target, not yet enforced — measure locally, don't block merge
- C. Drop the explicit floor — rely on cargo test + k6 compatibility instead
- X. Other (please specify)

[Answer]: A. Enforce it now — add a Rust coverage tool (e.g. cargo-llvm-cov) as a CI gate

## Q4 — Dependency & secret scanning
`cargo-audit` runs but is advisory-only (doesn't block merge), and there is no
secret-scanning or SAST tool today. How should security scanning behave?

- A. Promote cargo-audit to a blocking gate; add secret scanning (gitleaks) pre-merge
- B. Keep cargo-audit advisory; add no new scanning for now
- C. Blocking cargo-audit + add SAST (CodeQL/Semgrep) and secret scanning
- X. Other (please specify)

[Answer]: A. Promote cargo-audit to a blocking gate; add secret scanning (gitleaks) pre-merge

## Q5 — Production deploy strategy
Production deploys currently gate on manual approval (tech lead + product owner).
What rollout strategy should we use for production?

- A. Simple recreate / direct deploy after approval
- B. Blue-green (swap environments, instant rollback)
- C. Canary (gradual traffic shift with metric thresholds)
- X. Other (please specify)

[Answer]: X. Other — デプロイ先は現在存在しないため、本番へのロールアウトはプレースホルダーとする

## Consolidated Summary Confirmation

The five interview answers above resolve the team's working practices:

- **Branching:** short-lived `feature/*` → `main` via PR, CI must be green.
- **Walking skeleton:** skipped; new features land as ordinary PRs.
- **Testing:** test-after; 80% Rust coverage floor **enforced now** via a CI coverage tool.
- **Security scanning:** `cargo-audit` promoted to a **blocking** gate; add secret scanning (gitleaks) pre-merge.
- **Production rollout:** placeholder — no deploy target exists yet.

These feed `team-practices.md` (five sections) and `discovered-rules.md`
(two mandated hard constraints). Does this all look correct before the
artifacts are finalized?

- Looks correct — finalize the artifacts from these answers
- Request changes — revise one or more answers before finalization
- Other — describe what you want instead

[Answer]: Looks correct
