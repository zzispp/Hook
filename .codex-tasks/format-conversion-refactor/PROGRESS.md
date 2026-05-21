# Progress

## 2026-05-21

- Started read-only mapping of existing conversion code and CLI reference projects.
- Confirmed current conversion architecture already routes through an explicit internal model in `crates/proxy/src/format_conversion`.
- Confirmed OpenAI Responses request parsing silently loses structured `function_call_output.output` arrays by reading only `output.as_str().unwrap_or_default()`.
- Confirmed official Codex Responses supports string or structured content item array for `function_call_output.output` and `custom_tool_call_output.output`.
- Confirmed Claude and Gemini both represent tool results as structured content blocks (`tool_result` and `functionResponse`).
- Implementation Plan: first expose and fix OpenAI Responses structured tool output parsing, then add explicit errors or mappings for official item types that cannot be represented without loss.
- Implemented internal tool kind tracking so function and custom tool calls/results are not silently conflated.
- Implemented OpenAI Responses request parsing for structured `function_call_output.output` / `custom_tool_call_output.output`, including text and data URL image content items.
- Implemented OpenAI Responses response output item parsing and generation for reasoning, message text, function calls/results, and custom tool calls/results.
- Fixed Responses response parsing so `output` items take precedence over `output_text`; this prevents tool calls from being dropped when both fields exist.
- Implemented Responses stream input handling for `response.custom_tool_call_input.delta`, reasoning deltas, and explicit unsupported errors for official item types that the internal model cannot represent.
- Split conversion modules and stream tests so edited files stay under the project 300-line limit.
- Validation passed: `perl -e 'alarm 60; exec @ARGV' cargo test -p proxy format_conversion -- --nocapture`.
- Validation passed: `perl -e 'alarm 60; exec @ARGV' cargo test -p proxy -- --nocapture`.
