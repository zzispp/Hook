# Password UI Template Migration

## Goal

Migrate the forgot-password and profile password-change UI to match the referenced split auth template while keeping existing Hook API flows unchanged.

## Boundaries

- Keep `/auth/forgot-password` on the reset-email request flow.
- Keep profile password changes on the account email-code plus new-password flow.
- Do not add mocks, fallbacks, or backend API changes.

## Validation

- `pnpm lint:frontend`
- `pnpm build:frontend`
