# Auth Unauthorized Redirect

## Goal

Expired or invalid browser sessions must clear stored JWT tokens and let the dashboard auth guard redirect to sign-in instead of throwing the root error page.

## Scope

- Return HTTP 401/403 for RBAC middleware authorization failures.
- Treat auth envelope `unauthorized` responses as session expiry in the JWT auth provider.
- Keep non-auth failures visible as real errors.
- Validate backend error mapping and frontend type checking.
