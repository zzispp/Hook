# 05 Audit Terminal

Finish audit state cleanup for all request terminal paths.

Acceptance:
- Success marks unattempted candidates as `unused`.
- Terminal failure marks remaining unattempted candidates as `unused` or an explicit terminal status.
- On-demand retry records are created only for real attempts.
