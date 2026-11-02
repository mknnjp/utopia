---
applyTo: "**"
---

# Commit message conventions

## Rules

- Use Conventional Commits format in English.
- Format: `<type>(<scope>): <subject>`
- Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
- Scope: infer from changed files (e.g., `auth`, `api`, `ui`). Omit if unclear.
- Subject: imperative mood, lowercase, no period, max 72 chars.
- No vague words like `update` or `fix bug`.
- Body: optional; explain WHAT and WHY, not HOW. Wrap at 100 chars.
- Breaking change: add `!` after type/scope and footer `BREAKING CHANGE: ...`
- Always write in English even if diff contains Japanese.

## Examples

- `feat(auth): add OAuth2 login with PKCE`
- `fix(api): handle null pointer in user lookup`
- `refactor(ui): extract Button component`
