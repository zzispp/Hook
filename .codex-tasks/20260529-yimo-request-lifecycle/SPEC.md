# 任务说明

## 背景

线上 `yimo` provider 的部分请求会长期停留在 `request_records.status = pending`，后备候选不会接管，只能依赖 `request_record_stale_sweep` 定时任务在 15 分钟后兜底回收。用户要求对照 `/Users/bubu/Downloads/Aether-main` 的实现，找出 Hook 的请求生命周期缺口，修复根因，并移除 `request_record_stale_sweep`。

## 目标

1. 完整分析 Aether 的请求执行、超时、流式收尾与失败切换逻辑。
2. 对照 Hook 找出导致 `request_records`/`request_candidates` 悬挂的根因。
3. 在 Hook 中实现根因修复，让请求在主流程里进入明确终态，不再依赖 stale sweep。
4. 移除 `request_record_stale_sweep` 的注册、定义与默认配置，并保持系统行为自洽。
5. 用可重复验证证明修复成立。

## 非目标

1. 不引入新的静默降级、mock、补丁式兜底。
2. 不修改无关 provider 选择策略或业务计费规则。

## 验证口径

1. 代码层确认 Hook 在请求发送失败、首字节超时、流读取超时、客户端取消、上游异常结束等路径都会落终态。
2. 相关测试通过，至少覆盖本次根因路径。
3. 移除 stale sweep 后，调度注册、文案、代码引用保持一致。
