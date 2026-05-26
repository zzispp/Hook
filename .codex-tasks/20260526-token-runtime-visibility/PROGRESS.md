# Progress

## 2026-05-26

- Started runtime visibility enforcement for existing user tokens.
- Confirmed proxy model listing and candidate selection both depend on `model_access` and `SchedulingSnapshot`.
- Added `group_code` to cached users, `visible_user_group_codes` to cached billing groups, and `active_user_group_codes` to the scheduling snapshot.
- Bumped the Redis scheduling snapshot key from `v3` to `v4` so new code rebuilds with the expanded schema.
- Runtime token billing group visibility is now checked from the scheduling snapshot before listing models or selecting provider candidates.
- Added model access tests for a user moved to a non-visible group and for an inactive owner user group.
