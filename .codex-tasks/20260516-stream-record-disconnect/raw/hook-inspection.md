# Hook Inspection Evidence

## Runtime

- Backend listened on `127.0.0.1:5555` before restart as PID `45371`.
- Restarted backend from `/Users/bubu/ZwjProjects/Hook` with `cargo run -p backend`; new listener PID is `70033`.
- PostgreSQL is available through Docker container `hook-postgres`, mapped as host port `5433`.

## Before Fix

Latest streamed request before restart:

```text
request_id=019e2f5b-b60f-79d2-9289-4b0b41275216
status=success
billing_status=missing_usage
is_stream=true
client_status_code=200
prompt_tokens=NULL
completion_tokens=NULL
total_tokens=NULL
total_latency_ms=33163
```

`request_candidates.provider_request_body` did not include `stream_options.include_usage`.

## After Fix Runtime Test

Real streamed curl request created:

```text
request_id=019e2f6b-92d3-7440-a357-462979c4c4e7
status=success
billing_status=settled
is_stream=true
client_status_code=200
prompt_tokens=27
completion_tokens=1403
total_tokens=1430
total_cost=0.04222500
first_byte_time_ms=1807
total_latency_ms=27458
```

Recorded upstream body:

```json
{"messages":[{"content":"写一首关于秋天的长诗。","role":"user"}],"model":"gpt-5.5","stream":true,"stream_options":{"include_usage":true}}
```

The captured SSE response ended with a usage chunk and `data: [DONE]`.

## Wallet Observation

`wallet_transactions` stayed empty because the tested API token is `token_type=independent`.
The request cost was still recorded against the token:

```text
token_id=019e2f5a-73bc-73c3-8264-10e10eb6d0a0
used_quota=0.04222500
request_count=1
```
