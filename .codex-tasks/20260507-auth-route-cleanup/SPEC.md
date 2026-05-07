# Auth Route Cleanup

## Goal

Remove unused non-JWT auth UI and demo auth surface from the Hook frontend, keep the JWT implementation, and expose JWT sign-in at `/auth/sign-in/?returnTo=%2Fdashboard%2F` instead of `/auth/jwt/sign-in/?returnTo=%2Fdashboard%2F`.

## Scope

- Inspect `apps/hook_frontend/src/auth`, `apps/hook_frontend/src/app/auth-demo`, and related frontend references.
- Remove unused auth/demo modules that are no longer part of the active JWT path.
- Preserve real JWT sign-in behavior.
- Update route references and validation so the dashboard redirect targets `/auth/sign-in/`.

## Constraints

- Do not add fallback or compatibility routes unless required by existing framework mechanics.
- Keep failures explicit through lint/build/test output.
- Keep changes focused on frontend auth cleanup.
