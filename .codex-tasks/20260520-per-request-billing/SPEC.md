# 目标
让全局模型的按次计费在 provider 没有返回 token usage 时也能正常结算并写入请求记录。

# 范围
- 移除成功请求在计费阶段对 `usage` 的硬依赖。
- 保持 token 计费在有 usage 时的现有行为。
- 补充无 usage 的按次计费回归测试。

# 验收
- 配置了 `default_price_per_request` 的模型，在无 usage 的成功请求下会进入 `settled`。
- `BillingService` 可以只依赖 `price_per_request` 计算 request cost。
- 自动化测试通过。
