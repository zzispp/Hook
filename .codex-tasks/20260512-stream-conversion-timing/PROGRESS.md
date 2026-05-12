# Progress

## Recovery

任务: 对齐 Hook proxy 流式格式转换时机
形态: single-full
进度: 5/5
当前: 完成
文件: `.codex-tasks/20260512-stream-conversion-timing/TODO.csv`
下一步: 读 Hook transport/executor 依赖，改成功流式路径为增量 Body stream。

## Notes

- aether 的流式跨格式响应转换在 `create_response_stream()` 中按 SSE 行调用 `convert_stream_chunk()` 并立即 yield。
- Hook 的 `stream_response()` 当前在成功状态下调用 `response_bytes()`，这会等待上游流结束，再调用 `convert_stream_body()`，最后一次性 `Body::from(body)`。
- Hook 需要状态化 chunk 转换；直接对现有批量 `convert_stream(chunks)` 逐 chunk 调用会让 Gemini 累计文本格式重复输出。
- 已新增状态化 `convert_stream_chunk`，并用 Gemini 累计文本测试证明增量转换结果与批量转换一致。
- Hook 成功流式路径已改为 `bytes_stream()` + `Body::from_stream()`，首个可输出 chunk 前预读，预读阶段转换失败会返回错误给 executor 继续 failover。
- 验证通过：`cargo check -p proxy -p backend`、`cargo test -p proxy --test format_conversion -- --nocapture`、`cargo test -p backend llm_proxy -- --nocapture`。
