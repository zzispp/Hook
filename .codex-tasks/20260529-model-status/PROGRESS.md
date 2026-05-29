# Progress

Started implementation on 2026-05-29.

2026-05-29:
- Added model status probe history retention cleanup storage method for raw runs and hourly stats.
- Registered `model_status_runs_cleanup` scheduled task with default 300s interval and 90 day retention.
- Split scheduled task implementations so touched files stay under the project file size limit.
- Validated JSON seeds, Rust formatting, storage model status tests, and backend check.
- Added selected-check batch fix API and UI for enabled state, interval, name prefix, and independent token.
- Validated batch fix with backend checks, frontend lint, and frontend production build.
- Replaced multi-model/multi-endpoint frontend create loop with one `batch-create` backend API request.
