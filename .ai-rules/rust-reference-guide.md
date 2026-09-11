# Rust Reference Guide

> Authoritative Rust standards for this project. All `.rs` files should follow these conventions.

## Source of Truth

The following files contain the detailed Rust rules for this project:

- `.github/instructions/rust-ref.instructions.md` — Copilot/VS Code Rust instructions
- `.claude/rules/rust-ref.md` — Claude Rust rules
- `.cursor/rules/rust-ref.mdc` — Cursor Rust rules

## General Guidelines

- Follow standard Rust idioms and the Rust API Guidelines
- Use `Result<T, E>` for fallible operations; never use `unwrap()` in production code
- Prefer `thiserror` for library error types, `anyhow` for application-level errors
- Use `serde` with `#[derive(Serialize, Deserialize)]` for JSON serialization
- All public items must have `///` doc comments
- Prefer `clippy`-clean code; run `cargo clippy` before committing
- Use `rustfmt` for consistent formatting; config is in `rustfmt.toml` if present

## Project-Specific Patterns

- **Error handling**: Use the `FireflyError` type defined in `src/core/error_mapping/`
- **Database**: Diesel ORM with async via `diesel-async`; migrations in `migrations/`
- **API**: Axum handlers in `src/api/handlers/`; middleware in `src/api/middleware/`
- **Auth**: JWT-based auth in `src/core/auth/`; token lifecycle tests in `tests/unit/`
- **Testing**: Integration tests in `tests/`; unit tests in `src/` alongside source files

## Forbidden

- No `unwrap()` or `expect()` in non-test code
- No `println!()` for logging — use `tracing` macros
- No raw SQL strings — use Diesel query builder or `sql_query` with type-safe results

## Validation Checklist

Before committing Rust code:

- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `cargo test` passes
- [ ] All public items have doc comments
- [ ] Error types use `thiserror` derive
