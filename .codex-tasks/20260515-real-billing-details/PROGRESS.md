# Progress

## Recovery

任务: 真实验证请求记录计费细则和服务档位
形态: single-full
进度: 5/5
当前: Completed
文件: .codex-tasks/20260515-real-billing-details/TODO.csv
结果: 已用真实 provider 请求通过本地 backend，直接查本地 Postgres 验证 `request_records` 和 `request_candidates` 都记录了 `service_tier=standard`、成本拆分、单价、倍率和最终计费。

## Evidence

- 上游模型解析:
  - MSUTools: `gpt-5.4-mini`
  - Ekan8: `gemini-3.1-pro-preview`
- 真实请求:
  - HTTP status: `200`
  - model: `hook-real-billing-details`
  - usage: `prompt_tokens=34`, `completion_tokens=26`, `total_tokens=60`
- DB 验证:
  - `input_cost=0.00008500`
  - `output_cost=0.00039000`
  - `request_cost=0.00000000`
  - `base_cost=0.00047500`
  - `billing_multiplier=0.15000000`
  - `total_cost=0.00007125`
  - `cost_currency=USD`
  - `input_price_per_million=2.50000000`
  - `output_price_per_million=15.00000000`
  - `cache_creation_price_per_million=1.25000000`
  - `cache_read_price_per_million=0.25000000`
- 详细结果文件: `.codex-tasks/20260515-real-billing-details/raw/results.json`

## Verification

- `cargo fmt --check`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo check -p backend`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p provider billing`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_records -- --nocapture`
- `node .codex-tasks/20260515-real-billing-details/real_billing_details.mjs`
