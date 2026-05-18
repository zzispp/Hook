# Remove frontend template screens

## Goal

Remove Minimal UI template/demo frontend screens while preserving the public entry pages, auth/error pages, and Hook business dashboard routes backed by the backend navbar/menu.

## Boundary

- Delete component demo routes, dashboard mock business templates, and their private sections/actions/types.
- Keep real dashboard business routes: dashboard overview, models, groups, wallet, tokens, announcements, tickets, and admin pages bound by backend defaults.
- Keep public homepage/marketing pages for this pass.
- Do not add compatibility redirects or fallback behavior; removed routes should naturally 404.

## Validation

- Frontend lint passes.
- Frontend build passes.
- Searches for removed template references are either empty or limited to retained public pages.
