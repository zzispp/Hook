# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Fix the post-login `/api/auth/me` forbidden failure for normal users without bypassing RBAC.
- Add a normal-user Dashboard model catalog similar in spirit to Aether: users can see active global models and their capabilities/pricing, but cannot manage admin model data.
- Add any required database migration changes and run the repository migration command.

## Non-Goals

- Do not give normal users admin model management permissions.
- Do not add mock model data or silent fallback success paths.
- Do not redesign unrelated auth, RBAC, or admin model pages.

## Done-When

- Normal users have the minimum default RBAC API/menu permissions needed to log in and view the model catalog.
- Hook exposes a user-facing read-only model catalog endpoint and dashboard page.
- Toasty migration command is run and validation failures are explicit.
