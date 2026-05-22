# System Settings Ticket Token

## Goal

- Add system setting for support ticket submission captcha, enabled by default.
- Add token quantity limit setting, default 5.
- Add token expiry cleanup/check interval setting, default 5 minutes.

## Scope

- Backend schema, seed, storage, setting API types.
- Frontend admin system settings form.
- Runtime consumers for tickets and token cleanup/limits.
