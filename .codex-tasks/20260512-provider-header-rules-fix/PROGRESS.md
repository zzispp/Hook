# Progress

## 2026-05-12

- Confirmed `header_rules` exists in provider endpoint API/storage and frontend uses an array of `{ action, key/value/from/to, condition? }`.
- Confirmed current proxy scheduling snapshot and `ProxyCandidate` do not carry `header_rules`, and upstream request construction only applies default auth headers.
- Added a failing backend unit test for header `set/drop/rename` behavior before changing request construction.
- Implemented `header_rules` propagation into the scheduling snapshot and proxy candidate.
- Added request header rule execution after default upstream authentication headers are set.
- Bumped the scheduling snapshot Redis key to `v2` because the serialized endpoint schema now includes `header_rules`.
- Added condition evaluation for header rules using current/original request body JSON paths and the existing frontend condition operators.
- Verified with a temporary backend on `127.0.0.1:3001` and an httpbin upstream. The upstream received `Authorization: Bearer header-rule-auth-conditions-20260512T160800Z`, `X-Provider-Overwrite-Probe`, and `X-Renamed-To`; the skipped/drop rules were absent.
- Restored the local DB endpoint and provider key after the real probe and cleared `hook:llm_proxy:scheduling:snapshot:v2`.
