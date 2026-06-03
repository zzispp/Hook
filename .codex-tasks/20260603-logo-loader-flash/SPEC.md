# Fix Logo Landing Loader Flash

## Goal

Fix the visual bug where clicking the logo on `/auth/sign-in/?returnTo=%2Fdashboard%2F` while navigating to the landing page shows a black loader first and then a white loader.

## Scope

- Identify the logo navigation path and both loader sources.
- Implement the shortest root-cause fix for the duplicate/color-changing loader.
- Verify with frontend checks and browser reproduction when feasible.

## Non-Goals

- Do not redesign the landing page.
- Do not add fallback loading UI.
- Do not change unrelated auth redirect behavior.
