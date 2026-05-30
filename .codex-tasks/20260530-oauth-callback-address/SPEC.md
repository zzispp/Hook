# OAuth callback address display

## Goal
Show GitHub and Google OAuth callback addresses in the admin quick login provider settings.

## Boundary
- Use the configured public access address when present.
- Show "请先设置公网访问地址" when the public access address is missing.
- Keep changes scoped to the existing admin system settings UI and translation seeds.

## Validation
- Run focused frontend lint/build checks when feasible.
