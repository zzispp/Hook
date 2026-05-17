# CLI Stream Request Records Diagnostics

## Goal

- Compare Hook and Aether behavior for CLI streaming request records.
- Use the real DB record prefix 019e36d7 as evidence.
- Identify why Hook writes fewer request_records and fix if the Hook link is missing a required record boundary.
