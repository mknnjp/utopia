---
applyTo: "**"
---

# Branch naming conventions

## Rules

- Format: `<type>/<description>` (lowercase only).
- Allowed prefixes: `feature/` (or `feat/`), `bugfix/` (or `fix/`), `hotfix/`, `release/`, `chore/`, `ai/`, `copilot/`, `claude/`, `cursor/`, `codex/`.
- Trunk branches (`main`, `master`, `develop`) use no prefix.
- Description rules: lowercase alphanumerics (`a-z`, `0-9`), hyphens (`-`) only; no underscores, spaces, or special chars.
- No consecutive, leading, or trailing hyphens/dots in description.
- Keep concise and descriptive; include ticket number when applicable (e.g., `feature/issue-123-new-login`).
- Release branches may use dots for version (e.g., `release/v1.2.0`).
- Invalid: uppercase letters, consecutive hyphens (`feature/new--login`), leading/trailing hyphens, spaces, underscores.

## Examples

- `feature/add-login-page`
- `feat/add-login-page`
- `bugfix/fix-header-bug`
- `fix/header-bug`
- `hotfix/security-patch`
- `release/v1.2.0`
- `chore/update-dependencies`
- `copilot/add-login-page`
- `claude/security-patch`
- `feature/issue-123-new-login`
