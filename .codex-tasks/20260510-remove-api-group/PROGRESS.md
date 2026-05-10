# Progress

## Recovery

- 2026-05-10: Task initialized. Current step is mapping API group field usage.
- 2026-05-10: Completed removal of active API permission group field and added drop-column migration. Updated admin API UI and menu tree indentation. Validation passed: `cargo fmt --all`, `cargo check --workspace`, `pnpm lint:frontend`, `cargo run -p backend -- migration up`, `perl -e 'alarm 60; exec @ARGV' cargo test --workspace`.
