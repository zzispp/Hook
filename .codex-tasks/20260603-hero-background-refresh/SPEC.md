# Fix Home Hero Background Refresh Blank

## Goal

Fix the landing hero refresh issue where the animated color band and dot field can render blank while hero copy, navigation, code panel, and bottom fade remain visible.

## Scope

- Inspect the current landing hero background implementation.
- Reproduce or instrument the refresh behavior locally where feasible.
- Apply the smallest root-cause fix without silent fallback behavior.
- Validate with frontend checks and browser inspection.

## Constraints

- Preserve existing user changes in the dirty worktree.
- Do not add mock success paths or hide rendering failures.
- Keep changes focused on hero background initialization and rendering.
