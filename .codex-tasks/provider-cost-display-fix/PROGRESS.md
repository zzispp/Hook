# Progress

## 2026-06-02

- Created task tracking artifacts.
- Confirmed `ProviderCostCard` donut total used ApexCharts default numeric display, while Provider model costs were built from every API key and every binding.
- Confirmed backend/provider selection treats empty `allowed_model_ids` as all models and non-empty values as an allowlist.
- Updated dashboard provider cost donut value/total formatting to use existing dashboard money formatters.
- Updated provider model costs to render one section per API key and filter each section to models allowed by that key.
- Updated provider model cost dialog so model choices are filtered by the selected key and key changes clear selection/drafts.
- Validation passed: `pnpm --filter hook_frontend lint`; `pnpm --filter hook_frontend exec tsc --noEmit`.
