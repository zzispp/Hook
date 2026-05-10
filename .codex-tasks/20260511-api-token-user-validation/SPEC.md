# API Token User Validation

## Goal

Admin token creation must validate user ownership on the frontend before calling the backend.

## Scope

- Only the admin token creation dialog.
- When token type is `user`, `user_id` must be selected.
- Show a field-level validation message on the user selector.
- Keep backend validation unchanged.

