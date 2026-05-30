# OAuth public base URL

## Goal
Use the configured public base URL for GitHub and Google OAuth callback URLs, and prevent enabling either provider unless the public base URL is present and valid.

## Boundary
- Keep OAuth redirect URI construction deterministic and backend-controlled by saved system settings.
- Add frontend validation matching existing system settings save checks.
- Add backend validation so API writes cannot enable Google/GitHub OAuth without a valid public base URL.
- Do not add fallback redirect behavior.

## Validation
- Frontend lint/type/build where feasible.
- Rust check/test for affected settings/user code where feasible.
