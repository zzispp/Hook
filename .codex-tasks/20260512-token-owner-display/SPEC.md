# Token owner display

## Goal

Admin token management should show a readable owner identity in the table, preferring username or email instead of a raw user id.

## Scope

- Inspect the current admin token list data flow.
- Expose owner display data from the backend when the current API does not include it.
- Render the owner column with username or email while keeping raw ids out of the table's primary display.
- Validate Rust and frontend checks affected by the change.
