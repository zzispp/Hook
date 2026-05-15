# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape

- **Shape**: `single-full`

## Goals

- 为 Hook 增加 OpenAI 风格的 `GET /v1/models` 与 `GET /v1/models/{model}` 接口。
- 接口必须复用现有 API 令牌鉴权，不要求前台 JWT。
- 返回模型必须对齐 `new-api`，按令牌所在计费分组过滤，并叠加令牌自身模型限制；用户令牌还要叠加用户模型限制。

## Non-Goals

- 不实现 `/v1beta/models` 的列表接口。
- 不修改前端页面或 `/api/models/catalog` 的现有行为。
- 不引入兼容性 fallback 或放宽现有令牌校验。
- 不放松真实转发阶段对 provider candidate 的校验。

## Constraints

- 遵守仓库 Rust 代码结构约束，避免继续增大超限文件。
- 复用现有 `llm_proxy` 令牌与调度快照，不新增第二套鉴权来源。
- 先写失败用例，再修复到通过。
- `/v1/models` 的列表语义与真实转发语义允许不同，需与 `new-api` 一致。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: `Rust workspace + pnpm monorepo`
- **Package manager**: `cargo / pnpm`
- **Test framework**: `cargo test`
- **Build command**: `just check`
- **Existing test count**: `未单独统计；按目标 crate 运行验证`

## Risk Assessment

- [x] External dependencies (APIs, services) — availability confirmed?
- [x] Breaking changes to existing code — impact assessed?
- [x] Large file generation — disk space sufficient?
- [x] Long-running tests — timeout configured?

## Deliverables

- `llm_proxy` 新增 `/v1/models` 与 `/v1/models/{model}` 处理器与路由。
- 共享的 API 令牌模型可见性过滤逻辑。
- 覆盖过滤行为与接口行为的 Rust 测试。

## Done-When

- [ ] 带有效 API 令牌访问 `/v1/models` 返回该令牌在权限上可见的模型，即使当前没有 provider 绑定。
- [ ] `/v1/models/{model}` 对不可见或不存在模型返回明确错误。
- [ ] 相关 Rust 测试通过。

## Final Validation Command

```bash
cargo test -p backend llm_proxy -- --nocapture
```

## Demo Flow (optional)

1. 使用有效 API 令牌请求 `GET /v1/models`。
2. 确认响应只包含计费分组与令牌允许的模型。
3. 请求 `GET /v1/models/{model}` 验证允许与不允许两种情况。
