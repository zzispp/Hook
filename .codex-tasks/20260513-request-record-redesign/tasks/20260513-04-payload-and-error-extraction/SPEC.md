# Single Task Spec

## Goal

- 区分 client/provider 双视角 payload，并为失败记录保留更具体的错误片段。

## Scope

- `apps/hook_backend/src/llm_proxy/proxy/transport.rs`
- `apps/hook_backend/src/llm_proxy/proxy/stream_transport.rs`
- `crates/storage/src/provider/**`
- `crates/types/src/provider/request_record.rs`

## Done-When

- `request_records` 承载 client payload
- `request_candidates` 承载 provider payload
- 非 2xx 失败不再只写死通用错误文案
