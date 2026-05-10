# User Update Optional Password

## Goal

Editing a user without entering a new password must not reject the request and must preserve the existing password hash.

## Scope

- Keep create-user and sign-up password validation unchanged.
- Treat blank `password` on user update as "do not change password".
- Validate and hash password on update only when a nonblank new password is provided.
- Update the admin edit dialog helper text so the UI matches backend behavior.
- Verify with focused backend tests and frontend type check.

## Done When

- `PUT /api/users/:id` accepts a blank password field for normal user profile edits.
- Existing password hash is preserved on blank password updates.
- Nonblank update password still passes normal password validation and updates the hash.
