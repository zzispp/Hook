# Aether / new-api Comparison

## Forced Usage Requesting

- `new-api/relay/compatible_handler.go` forces `stream_options.include_usage=true` when `ForceStreamOption` is enabled.
- `Aether/src/api/handlers/base/chat_handler_base.py` and `Aether/src/services/provider/stream_policy.py` force `stream_options.include_usage=true` for OpenAI Chat streams.
- Hook previously forwarded the user's OpenAI Chat stream body unchanged, so upstream did not return usage and Hook could not calculate billing.

## Streaming Usage Extraction

- Aether extracts usage from OpenAI Chat stream chunks and OpenAI Responses `response.completed.response.usage`.
- Hook now extracts OpenAI Chat stream usage incrementally across network chunks and extracts OpenAI Responses stream usage from `response.completed.response.usage`.

## Timeout Semantics

- Hook had `stream_first_byte_timeout_seconds=30` and `request_timeout_seconds=300` in the provider.
- The previous Hook request builder applied the first-byte timeout as reqwest's total request timeout for streams, so long streams could be cancelled around 30 seconds.
- Hook now keeps reqwest total timeout on `request_timeout_seconds` and applies `stream_first_byte_timeout_seconds` only to the initial body chunk prefetch.
