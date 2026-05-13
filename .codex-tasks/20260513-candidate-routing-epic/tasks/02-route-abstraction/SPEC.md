# 02 Route Abstraction

Replace flat `CandidateParts` as the main scheduling unit with a provider route that carries endpoint and key policies.

Acceptance:
- Scheduler can order routes without depending on a single concrete key.
- Proxy execution can resolve an actual endpoint and key for each attempt.
- Existing billing and trace metadata still have all required fields once an attempt is materialized.
