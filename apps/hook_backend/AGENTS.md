# Backend Agent Instructions

## Testing Policy

- Use Test Driven Development (TDD) for backend changes.
- Write or update a failing unit test before changing production behavior.
- Keep tests close to the module they verify.
- Prefer deterministic unit tests over tests that require external services.
- Run `cargo test` after implementation, then `cargo fmt --all --check` and `cargo check`.
- Do not add fallback paths or fake success behavior to production code to make tests pass.
