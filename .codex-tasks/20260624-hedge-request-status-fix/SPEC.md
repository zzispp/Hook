# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- 移除流式请求的 hedged request 并发执行路径，避免 backup candidate 额外消耗上游 token。
- 保留串行 candidate failover、retry、stream watchdog、请求 timing 记录等非 hedge 行为。
- 删除或调整 hedge-only 的取消原因传播和测试，避免死代码留存。

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- 不回滚 timing 字段、首字/首输出统计和 request record 展示。
- 不移除非流式请求处理。
- 不移除串行 failover 和 retry。
- 不引入模拟成功、静默 fallback 或兼容性补丁。
- 不修改线上数据。

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Rust 后端保持现有审计与 storage 分层。
- 流式请求最多一次只向一个上游 candidate 发起请求。
- 后端测试使用 60 秒超时。

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024 / Next.js
- **Package manager**: pnpm
- **Test framework**: cargo test
- **Build command**: `just check`
- **Existing test count**: not enumerated

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [ ] External dependencies (APIs, services) — availability confirmed?
- [ ] Breaking changes to existing code — impact assessed?
- [ ] Large file generation — disk space sufficient?
- [ ] Long-running tests — timeout configured?

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Backend executor 移除流式 hedged request 并发执行。
- 删除 hedge-only 的取消原因传播测试或代码，确保没有未使用路径。
- Conventional commit on feature branch, pushed to GitHub, CI verified, merged into main.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Focused Rust tests pass within 60 seconds.
- [ ] Relevant formatting/checks pass or any blocker is explicitly recorded.
- [ ] Branch is pushed and GitHub Actions pass.
- [ ] Pull request is merged into `main`.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
timeout 60s cargo test -p hook_backend llm_proxy::proxy
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
1. Trigger a stream request with multiple candidates configured.
2. Confirm only one candidate is attempted at a time.
3. Confirm fallback to the next candidate only happens after the current candidate fails before commit.
