# Progress

## Recovery

- Task: Fix black-to-white double loader flash when logo navigates from sign-in to landing page.
- Shape: single-full
- Progress: 2/4
- Current: Run static/build validation.
- Latest validation: Diff review confirmed only two loader entry points changed.
- Truth file: `.codex-tasks/20260603-logo-loader-flash/TODO.csv`

## Log

- 2026-06-03: Created task record and started source inspection.
- 2026-06-03: Located root cause: `/auth/sign-in` logo navigates to `/`; `(home)/loading.tsx` renders `ReactBitsLoader mode="dark"`, then `LandingPage` renders `LandingLoader` with default auto/light mode.
- 2026-06-03: Updated `(home)/loading.tsx` and `LandingLoader.tsx` to use the global `SplashScreen` loader.
