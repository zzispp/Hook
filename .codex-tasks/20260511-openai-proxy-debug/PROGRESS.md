# OpenAI Proxy Debug Progress

## 2026-05-11

- Started diagnosis from the local Hook workspace.
- Existing workspace contains many unrelated provider/proxy changes; these will be preserved.
- Reproduced the failure path: `8082` is the Next frontend, `/v1/chat/completions` is redirected by trailing-slash handling and then returns the frontend 404 page; backend `3000` has no `/v1` route and treats the API token as JWT.
- Compared aether public OpenAI routing and candidate building. The Hook fix needs endpoint-signature routing plus Hook billing-group constraints: token group, group model/provider bindings, active provider endpoints/keys, and provider model binding.
- Implemented `/v1` public proxy routes with API-token middleware, exact OpenAI endpoint format selection (`openai_chat`, `openai_cli`, `openai_compact`), billing-group model/provider filtering, provider key decryption, and frontend `/v1` rewrite to the backend.
- Validation passed: `cargo test -p rbac authorize_api_allows_whitelisted_prefix_without_matching_sibling_prefixes`, `cargo test -p provider decrypt_provider_key`, `cargo check -p proxy`, `cargo check -p backend`, and `pnpm --filter hook_frontend lint`.
- Runtime validation passed for `8082` non-stream chat completions, `8082` streaming chat completions, `8082` responses non-stream, `8082` responses stream, and `8082` responses compact. WebSocket `/v1/realtime?model=gpt-5.5` reached the new backend token route on `3000`, selected the provider, then failed explicitly because upstream `wss://www.hook.rs/v1/realtime` returned `404 Not Found`.
- Fixed request records that stayed `streaming`: HTTP stream completion now updates the selected attempt to `success`, websocket relay/connect failures write terminal `failed` attempts, and local historical stuck rows were marked terminal in PostgreSQL.
- Fixed independent token ownership: admin-created independent tokens are owned by the creator user id. The local `sk-KrYgz8w...` token now has `user_id=00000000-0000-7000-8000-000000000000`, and the obsolete `fk_api_tokens_user` constraint was dropped because the config system admin is not a row in `users`.
- Runtime admin API validation shows request records for the independent token now return `username=admin`; latest active `streaming/pending` count is `0`.
- Request detail trace now includes candidate `is_stream` alongside `client_api_format`, `provider_api_format`, and `needs_conversion`; verified on request `019e175b-c878-7663-a932-3b5a81fcce34`.
