# Performance Monitoring Currency Display

## Goal

性能监控成本展示必须复用后台管理系统的显示货币设置，而不是使用 `fCurrency` 的 locale 默认货币。

## Scope

- 读取系统显示货币设置。
- CNY 展示时使用现有 USD/CNY 汇率数据。
- 性能监控成本卡片使用现有 `CurrencyDisplay` 与 `formatMoneyCompact`。
- 保持后端聚合金额仍为 USD 基础金额，不改计费、钱包、审计语义。

## Evidence

- 请求记录页通过 `useSystemSettings` 与 `useUsdCnyExchangeRate` 构造 `CurrencyDisplay`。
- `currency-format.ts` 已支持 USD/CNY 展示和汇率不可用提示。
- 性能监控当前通过 `fCurrency(llm?.cost ?? 0)` 格式化，未接系统显示货币。

## Validation

- `pnpm lint:frontend`
- `pnpm build:frontend`
