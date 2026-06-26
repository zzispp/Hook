# LLM 时延四指标口径改造

## 目标

把请求记录链路统一成 4 个明确指标：

- 响应头
- 首字
- 首 Token
- 总耗时

## 范围

本次只改请求记录层与 Admin 展示链路：

- 后端流式请求记录时机与 SSE 检测语义
- Admin 请求记录表 expanded timing
- 请求详情抽屉与 trace timeline
- usage records 中的首 Token 命名
- admin 中英文本地化文案

## 非范围

- 不新增数据库字段
- 不做 schema migration
- 不回填历史数据
- 不改 routing / performance 的 TTFB 统计源
- 不改成本分析 / 用户统计 / 成本预测 / 节省分析的统计口径

## 语义约束

- `response_headers_time_ms`：上游响应头可用时间
- `first_sse_event_time_ms`：首个有效 SSE `data:` 事件时间，忽略空行、keepalive、非 `data:`、`[DONE]`
- `first_output_time_ms`：首个真实可感知输出时间，继续复用现有输出检测器
- `total_latency_ms`：请求完成时间
- `first_byte_time_ms`：仅保留给旧统计和内部兼容，不作为本次 4 指标展示来源

## 验证

- Rust 单测覆盖首字检测与兼容 timing 行为
- 后端编译/测试通过
- 前端 lint 与构建通过
