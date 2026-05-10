# Progress

## 2026-05-10

- API management currently renders a flat list from `useApis()`.
- API rows do not include their bound menu IDs, while edit mode loads bindings through `getApiMenus(api.id)`.
- Grouped rendering should use real binding data from the list response, not per-row fallback requests.
- Backend API list responses now carry `menu_item_ids` so grouping uses real binding data from the list payload.
- Frontend API management table now groups rows by bound menu and shows unbound APIs under an explicit unbound group.
- Extracted grouped table rendering into `api-management-table.tsx`; `api-management-view.tsx` is back under the 300-line file limit.
- Validation passed: `cargo fmt --all --check`, `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p rbac page_apis`, `pnpm --filter hook_frontend exec tsc --noEmit --pretty false`.
