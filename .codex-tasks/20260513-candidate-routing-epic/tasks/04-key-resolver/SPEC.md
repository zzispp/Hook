# 04 Key Resolver

Move provider key choice from candidate construction into a key resolver.

Acceptance:
- Multiple active keys produce one route rather than many initial candidates.
- Selected key is stable for cache affinity when possible.
- Failed key attempts can move to the next eligible key with an auditable record.
