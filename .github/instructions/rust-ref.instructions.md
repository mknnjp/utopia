---
applyTo: "**/*.rs"
---

# Rust references

- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types/traits, SCREAMING_SNAKE_CASE for constants).
- Prefer `Result` and `Option` over panics; use `?` for error propagation.
- Use `clippy` and `rustfmt` for formatting and linting.
- Keep modules focused; split large files into submodules.
- Document public APIs with doc comments (`///`).
- Use `#[derive(...)]` where appropriate; avoid manual `Clone`/`Debug` when possible.
