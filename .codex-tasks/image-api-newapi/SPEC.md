# 图片接口对齐 new-api

## Task Shape

- **Shape**: `single-full`

## Goals

- 为 Hook 增加 new-api 风格的 OpenAI 图片生成/编辑接口。
- 图片生成走 `/v1/images/generations`，图片编辑走 `/v1/images/edits` 和 `/v1/edits`。
- 图片编辑支持 multipart/form-data 透传，保留原始字段与文件，不做格式转换。
- 保持现有 provider 调度、失败降级和审计链路。

## Non-Goals

- 不扩展其他厂商专用图片协议。
- 不改动非图片文本/对话接口的语义。

## Constraints

- 遵守当前仓库的 Rust 代码规范、分层规范与依赖层级规范。
- 图片请求不做跨格式转换。
- 保留 keepalive / SSE 的现有流式处理行为。

## Deliverables

- 图片生成/编辑路由与处理逻辑。
- multipart 编辑请求构造与上游透传。
- 必要的测试覆盖。

## Done-When

- 图片生成/编辑请求可正常通过 Hook 代理到上游。
- 首个 provider 失败后可继续降级到下一个 provider。
- 相关测试通过。

## Final Validation Command

```bash
cargo fmt --all && cargo check && cargo clippy -p backend --all-targets -- -D warnings && just test
```
