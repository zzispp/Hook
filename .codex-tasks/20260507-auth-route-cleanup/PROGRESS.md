# Progress

## Recovery

- 任务: Clean unused frontend auth UI and route JWT sign-in to `/auth/sign-in`.
- 形态: single-full
- 进度: 4/4
- 当前: Complete.
- 文件: `.codex-tasks/20260507-auth-route-cleanup/TODO.csv`
- 下一步: Inspect route tree and references.

## Log

- Created task tracking artifacts.
- Mapped active JWT dependencies and unused provider/demo surfaces.
- Removed non-JWT provider/demo routes, views, contexts, unused provider helpers, provider libs, and provider package dependencies.
- Moved JWT route constants and route files so protected redirects target `/auth/sign-in`.
- Validation exposed a missing direct dependency for existing editor imports: `@tiptap/core`.
- Fixed build failures by declaring missing direct deps (`@tiptap/core`, `@types/geojson`) and using Zod input types for nullable form default values.
- Final validation passed: `pnpm lint:frontend`, `pnpm --filter hook_frontend exec tsc --noEmit --pretty false`, and `pnpm build:frontend`.
