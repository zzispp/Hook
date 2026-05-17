# Progress

2026-05-15
- Started implementation from approved plan.
- Added global provider cooldown policy types, settings persistence, validation, baseline schema, provider cooldown storage, admin APIs, RBAC seed API definitions, and backend i18n seed keys.
- Added Redis fixed-window status failure counting and provider cooldown TTL keys. Scheduling snapshots carry only the policy; active cooldown state is read from Redis during candidate selection.
- Wired HTTP upstream status failures to cooldown recording. Retryable failures that trigger cooldown stop the current provider candidate and continue to the next provider. Non-retryable status responses keep the existing client response behavior.
- Added startup Redis cooldown restoration from active DB records. Manual release updates DB and deletes the Redis cooldown key. Provider deletion clears the Redis cooldown key and rebuilds the scheduling snapshot.
- Added provider management tabs, cooldown policy dialog, cooldown list table, release action, frontend types/actions/endpoints, and split provider page state to keep files under project limits.
- Validation passed: jq empty on admin.cn/en seed JSON; cargo fmt --check; cargo check --workspace; just test; pnpm lint:frontend; pnpm build:frontend. The frontend build still logs an existing `Axios error: Something went wrong!` during static generation but exits 0.
