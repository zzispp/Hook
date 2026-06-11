# Progress

2026-06-10
- Local Postgres shows provider `老尼` has `group_changed_action = disable_key`.
- The single imported key `codex` / upstream token `1209` is inactive and has `["upstream_group_changed"]`.
- `provider_quick_import_sync_events` contains two `upstream_group_changed` events for the same provider/source/key within milliseconds, so the duplicate came from one sync pass.
- Fixed notification generation to emit "上游分组已同步" only when the sync outcome includes an accepted upstream group patch.
- Added regression tests for disable-key group changes and sync-accepted group changes.
- Validation passed: `cargo fmt --check`, `timeout 60 cargo test -p provider quick_import_sync_policy_tests -- --nocapture`, `timeout 60 cargo test -p provider quick_import_sync -- --nocapture`.
