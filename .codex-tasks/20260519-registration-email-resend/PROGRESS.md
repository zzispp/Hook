# Progress Log

## Context Recovery Block

- **Current milestone**: Complete
- **Current status**: DONE
- **Goal**: Add frontend 60s resend cooldown and backend anti-abuse behavior for registration email codes.
- **Contract**: Same email cannot trigger a send more than once per 60s. After 60s, if the previous code is still unexpired, backend re-sends the same code. Successful registration consumes the Redis code so it cannot be reused.
- **Next action**: Implement backend storage/service changes.

## 2026-05-19

- Read current backend/frontend registration email flow.
- User redirected storage strategy from DB to Redis. Registration email code is temporary state, so the code source of truth will be Redis.
- Semantic constants: verification code TTL remains 10 minutes; resend cooldown is 60 seconds.
- Implemented Redis registration email code store with `SET NX EX` cooldown and atomic compare/delete consume.
- Removed registration email verification DB storage path from storage/user and backend baseline creation; baseline apply explicitly drops the obsolete table if present.
- Added frontend 60 second cooldown after successful code request.
- Validation passed: `cargo test -p user`, `cargo check -p backend`, `pnpm --filter hook_frontend lint`, `pnpm --filter hook_frontend build`.
