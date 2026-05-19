# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- 修复注册邮件验证开启后，注册界面没有展示邮件 OTP 输入和发送入口的问题。
- 保持真实后端链路：需要验证码时由前端发送验证码并提交验证码字段，不添加 mock 成功或静默降级。

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- 不改变后台系统设置的业务语义。
- 不新增兼容旧接口的临时路径。

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- 前端：Next.js / React Hook Form / Zod / MUI。
- 后端：Rust workspace，注册邮件验证配置来自系统设置。
- i18n：认证页文案由后端 seed 资源提供，不恢复前端 auth locale JSON。

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript + Rust
- **Package manager**: pnpm / cargo
- **Test framework**: ESLint, Next build, Rust tests as needed
- **Build command**: `pnpm build:frontend`
- **Existing test count**: 未统计

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — 邮件服务不在本次验证中实发测试。
- [x] Breaking changes to existing code — 待排查后限定在注册链路。
- [x] Large file generation — 不涉及。
- [x] Long-running tests — 后端测试需使用仓库 60 秒包装命令。

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- 注册页在后端开启 `registration_email_verification_enabled` 时展示邮件验证码输入和发送按钮。
- 注册请求携带后端需要的验证码字段。
- 必要认证文案写入后端 i18n seed。

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] 已定位注册配置、验证码发送接口、注册请求字段。
- [ ] UI 和请求逻辑实现真实邮件验证码链路。
- [ ] 验证命令通过或失败原因明确。

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm --filter hook_frontend lint
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. 打开注册页。
2. 后端开启注册邮件验证时，输入邮箱后可以发送验证码。
3. 注册提交时包含邮件验证码。
