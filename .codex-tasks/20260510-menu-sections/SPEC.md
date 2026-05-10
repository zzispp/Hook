# Menu Section Binding And Defaults

## Goal

Make menu item grouping explicit through a menu section dropdown in the menu item modal, and align default admin navigation sections with the requested product grouping.

## Requested Navigation Sections

- Overview: dashboard only.
- System Management: user management, role management, API management, menu management, system settings, model management.
- A separate business/operations section: wallet management, wallet center, token management, model catalog, billing groups.

## Scope

- Inspect existing menu section/menu item frontend and backend shapes.
- Update menu item create/edit modal to select a menu section from existing sections instead of relying on loose text or hidden binding.
- Update default menu section assignments and an additive migration for existing DB rows.
- Verify backend and frontend checks.

## Validation

- `cargo fmt --all`
- `cargo check --workspace`
- `pnpm lint:frontend`
- `cargo run -p backend -- migration up`
- `perl -e 'alarm 60; exec @ARGV' cargo test --workspace`
