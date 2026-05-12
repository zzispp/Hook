# Move Authenticated Base APIs To Config

## Goal

Move the runtime list of authenticated-only base APIs from a hard-coded backend constant into `auth.authenticated` under the YAML auth configuration, next to `auth.whitelist`.

## Scope

- Add `authenticated` to config parsing.
- Populate `config/config.yaml` with the existing authenticated base API paths.
- Wire backend authorization setup to use the configured rules.
- Validate config parsing and backend/frontend checks affected by the change.

## Constraints

- Do not put authenticated-only endpoints in `auth.whitelist`.
- Do not add default fallback rules in code; missing config should fail visibly.
- Preserve unrelated dirty workspace changes.
