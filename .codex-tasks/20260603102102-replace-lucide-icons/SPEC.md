# Replace lucide-react with internal Iconify

## Goal

Remove frontend usage of `lucide-react` and use the internal icon component instead.

## Scope

- Replace `lucide-react` imports in `apps/hook_frontend/src/react-bits`.
- Remove the frontend dependency entry when no source imports remain.
- Run frontend build until it succeeds or a real blocker is exposed.

## Validation

- `rg -n "lucide-react" apps/hook_frontend`
- `pnpm build:frontend`
