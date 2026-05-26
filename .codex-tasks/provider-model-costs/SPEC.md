# Provider Model Costs

Implement key-scoped upstream model costs, remove provider-model pricing overrides from customer billing, and split request cost details into upstream cost, customer billing, and service tier sections.

## Constraints

- Do not migrate old provider model price data; this repo is still in baseline-development mode.
- Store upstream costs by provider key and provider model binding.
- A cost row is either per request or per token, never both.
- Missing upstream cost config uses the request-time global model price as an explicit snapshot source.
- Customer billing remains global model base price multiplied by billing group multiplier.

