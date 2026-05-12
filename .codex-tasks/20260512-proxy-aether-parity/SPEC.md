# LLM Proxy Aether Parity

## Goal

Bring Hook LLM proxy scheduling semantics closer to aether while preserving Hook's billing-group layer.

## Scope

- Rename the backend app proxy module away from `openai` because it serves OpenAI-compatible, Claude, Gemini, and realtime proxy traffic.
- Route all LLM proxy traffic through the scheduler crate instead of the local one-shot selector.
- Add failover/retry across candidate keys/providers.
- Add key-level filtering for active flags and time windows where Hook has data.
- Add fixed-order, cache-affinity, and load-balance scheduling based on backend system settings.
- Avoid provider pagination order distortion by collecting all active provider candidates before global ordering.
- Keep conversion demotion as the current degradation strategy.
- Preserve billing group model/provider restrictions and multiplier billing.
- Do not implement health probing or health-based circuit breaking in this stage.

## Acceptance

- Candidates respect token permissions, billing group permissions, provider/model/endpoint/key active flags, and endpoint format compatibility.
- Failed upstream attempts advance to the next retry/candidate and are visible in request trace.
- Cache-affinity and load-balance ordering are explicit and test-covered.
- Existing OpenAI/Claude/Gemini conversion tests remain green.
- Backend checks and focused tests pass.
