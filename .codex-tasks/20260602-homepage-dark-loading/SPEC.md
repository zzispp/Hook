# SPEC

## Goal

Remove the white frame during hard refresh of the React Bits homepage.

## Evidence

The white frame appears while `body` is still empty and the root document has `data-color-scheme="light"`. The homepage needs a dark document background before React mounts and before route loading content is inserted.

## Scope

- Homepage route group loading boundary.
- React Bits homepage document background marker.
- Do not change dashboard/auth splash behavior.

## Validation

- `pnpm --filter hook_frontend lint`
- `pnpm --filter hook_frontend build`
- Browser refresh verification for `http://localhost:8082/`

