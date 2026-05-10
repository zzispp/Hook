# Progress

## 2026-05-10

- Root cause: backend auth middleware returns `RbacApiError` with HTTP 200 and `{ success: false, message: "unauthorized" }`; frontend `resolveCurrentUser()` then throws a plain `Error` from `requireApiData()`.
- Because `AuthProvider` rethrows `state.error`, the route guard never gets a chance to see `unauthenticated` and redirect.
