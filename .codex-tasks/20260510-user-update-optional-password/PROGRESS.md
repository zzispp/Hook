# Progress

## 2026-05-10

- User reported `PUT /api/users/:id` with `"password": ""` fails with `password must be between 8 and 128 characters`.
- Root cause: `UserPayload` is converted into `ReplaceUser`, then `validate_replace_user` always validates `password` with create-user rules; service also always builds a new `password_hash` for replace.
- Desired behavior from first principles: password is a credential, not a required profile field on every user edit. Blank password on edit means no credential change; a nonblank password is an explicit credential change.
- Added failing coverage for blank-password user replacement, then changed replace semantics to use `Option<String>` password after sanitization.
- `replace_user` now loads the current auth record and preserves its password hash when the sanitized update password is absent.
- Added coverage that a nonblank update password is still validated, hashed, and persisted.
- Updated admin edit helper text to say blank password keeps the current password.
- Validation passed: `cargo fmt --all --check`, `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p user replace_user`, `pnpm --filter hook_frontend exec tsc --noEmit --pretty false`.
