# Baseline Only Migration Rewrite

## Goal

- Remove incremental development migrations.
- Keep one baseline migration that creates the current final schema and seeds current default data.
- Do not preserve compatibility with already-applied development migrations.
