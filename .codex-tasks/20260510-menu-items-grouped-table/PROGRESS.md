# Progress

## 2026-05-10

- Current menu item table is flat and only displays section name in a normal table cell.
- `useMenuManagementData()` already fetches `allSections`, enough to group current page items by section.
- Menu item table now groups rows under raw menu section `subheader` values and indents menu item rows.
- Validation passed: `pnpm --filter hook_frontend exec tsc --noEmit --pretty false`.
