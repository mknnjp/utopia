# Discovered Rules

> Hard constraints stated by the human during the interview, in
> `ALWAYS ...` / `NEVER ...` form. Finalized after the consolidated-summary
> confirmation.

## Mandated

- ALWAYS run `cargo-audit` as a **blocking** gate in CI — a known-vulnerable Rust dependency must not be merged.
- ALWAYS run a secret-scanning step (e.g. gitleaks) pre-merge so committed secrets are caught before they reach `main`.

## Forbidden

<!-- No human-stated hard constraints discovered yet. -->
