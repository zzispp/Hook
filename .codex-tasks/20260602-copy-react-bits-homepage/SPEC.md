# Copy React Bits Homepage

## Goal

Replace the Hook public homepage with a 1:1 visual and behavioral copy of the homepage from `/Users/bubu/ZwjProjects/react-bits`.

## Scope

- Inspect the source React Bits landing page, assets, styles, and runtime dependencies.
- Inspect the Hook frontend routing and existing homepage implementation.
- Port only the homepage-facing files/assets required for the landing page.
- Preserve Hook app conventions where needed for Next.js compilation.
- Validate with automated frontend checks and browser rendering.

## Non-Goals

- Do not refactor unrelated dashboard/auth/admin flows.
- Do not add compatibility fallbacks or mock behavior.
- Do not add frontend locale JSON files.

## Acceptance Criteria

- `/` in `apps/hook_frontend` renders the React Bits landing page structure and styling.
- Required assets resolve from Hook public assets or local frontend source.
- Frontend lint/build checks pass or failures are documented with concrete evidence.
- Browser render is verified on the running local frontend.
