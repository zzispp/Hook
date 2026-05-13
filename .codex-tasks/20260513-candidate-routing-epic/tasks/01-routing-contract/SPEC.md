# 01 Routing Contract

Write failing tests that lock the intended behavior before refactoring scheduler internals.

Acceptance:
- Multi-key provider does not multiply initial route count.
- Exact endpoint is preferred over conversion endpoint.
- Conversion endpoint is only used as fallback according to explicit route policy.
- Success and terminal failure leave no `available` audit rows.
