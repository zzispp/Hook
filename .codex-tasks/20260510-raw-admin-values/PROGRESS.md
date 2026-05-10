# Progress

## Recovery

- 2026-05-10: Task initialized. Current step is inspecting translated helper usages.
- 2026-05-10: Removed admin-side value translation helpers for API names, menu titles/sections, roles, and billing groups. Runtime dashboard nav translation remains in `layouts/dashboard/nav-translation.ts`.
- 2026-05-10: Deleted unused admin locale maps for API permission names, role names/descriptions, auth sources, and system billing group labels after confirming no code references.
- 2026-05-10: Validation passed with `pnpm lint:frontend`; residual `rg` checks for removed helper names and deleted locale keys returned no matches.
