# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 删除 `c4067752d4f8987b2bec3958041cc9aff923d37b` 引入、但当前首页重写后已不再使用的静态资源、组件、上下文和常量。
- 删除 `Navbar` 中当前没有任何入口会触发的旧 React Bits docs 分支及其依赖。

## Non-Goals

- 不改动当前首页 section 的业务文案和布局结构。
- 不清理与本次首页未使用资源无关的其他模块。
- 不删除 `.codex-tasks/**` 任务记录文件。

## Constraints

- 只删除已通过当前入口链路和全仓引用确认无用的文件与代码。
- 避免覆盖用户当前工作区中对首页 section 的既有改动。
- 删除后需通过前端静态校验。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js App Router
- **Package manager**: pnpm
- **Test framework**: 前端当前以 lint / build 为主
- **Build command**: `pnpm build:frontend`

## Risk Assessment

- [x] Breaking changes to existing code — 已按当前首页真实入口和全仓引用确认删除范围
- [x] Long-running tests — 使用前端 lint 校验

## Deliverables

- 首页不再引入旧 docs 分支相关上下文、常量、静态资源和死内容组件
- 相关未使用静态资源与文件从仓库中移除
- 前端 lint 校验结果记录完成

## Done-When

- [ ] 当前首页入口不再依赖旧 docs 分支上下文与常量
- [ ] 已确认无用的静态资源、组件、常量文件从仓库移除
- [ ] `pnpm lint:frontend` 通过

## Final Validation Command

```bash
pnpm lint:frontend
```
