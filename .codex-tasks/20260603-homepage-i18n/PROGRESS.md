# Progress Log

## Session Start

- **Date**: 2026-06-03 15:00
- **Task name**: `homepage-i18n`
- **Task dir**: `.codex-tasks/20260603-homepage-i18n/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv`
- **Environment**: TypeScript / Next.js / pnpm

## Context Recovery Block

- **Current milestone**: #2 — 补充首页翻译资源并改造组件取词
- **Current status**: IN_PROGRESS
- **Last completed**: #1 — 盘点首页落地页路由与硬编码文案范围
- **Current artifact**: `apps/hook_frontend/src/react-bits/components/landingnew/**`
- **Key context**: 当前首页使用 `react-bits` 落地页，主要硬编码集中在 Navbar、Hero、Features、LiveDemo、QuickStart、Sponsors、CTA、Footer，以及 `(home)/page.tsx` metadata。
- **Known issues**: 现有 `common.json` 仅覆盖旧版首页 `home.*` 键，当前落地页需要补充新词条。
- **Next action**: 新增首页落地页翻译键并逐组件改造。

## Milestone 1: 盘点首页落地页路由与硬编码文案范围

- **Status**: DONE
- **Started**: 14:45
- **Completed**: 15:00
- **What was done**:
  - 确认首页入口位于 `apps/hook_frontend/src/app/(home)/page.tsx`
  - 确认实际渲染 `src/react-bits/pages/LandingPage.tsx`
  - 盘点主要硬编码组件与现有 i18n provider / gate 接入情况
- **Validation**: `rg -n "LandingPage|landingnew|generateMetadata|useTranslation|t\(" apps/hook_frontend/src/app/\(home\) apps/hook_frontend/src/react-bits/components/landingnew apps/hook_frontend/src/react-bits/pages -S` → exit 0
- **Files changed**:
  - `.codex-tasks/20260603-homepage-i18n/SPEC.md`
  - `.codex-tasks/20260603-homepage-i18n/TODO.csv`
  - `.codex-tasks/20260603-homepage-i18n/PROGRESS.md`
- **Next step**: Milestone 2 — 补充首页翻译资源并改造组件取词

## Milestone 2: 补充首页翻译资源并改造组件取词

- **Status**: DONE
- **Started**: 15:00
- **Completed**: 15:41
- **What was done**:
  - 新增 `landing` namespace，并在 `I18nProvider` 中注册 `cn/en` 资源
  - 为首页落地页新增 `landing.json` 中英文文案，覆盖 metadata、导航、Hero、功能卡片、演示区、部署区、渠道区、CTA、Footer
  - 首页路由 `apps/hook_frontend/src/app/(home)/page.tsx` 改为通过 `getServerTranslations('landing')` 生成 metadata
  - `LiveDemo.tsx` 重写为统一从 `landing` namespace 取词，日志、按钮、输入提示、预设问题、模拟响应全部去掉硬编码
  - `Features.tsx` 修正监控帧字段使用，完成与 `features-content.ts` 的一致接线
- **Validation**: `rg -n "useTranslate\('landing'\)|getServerTranslations\('landing'\)" apps/hook_frontend/src/app/\(home\) apps/hook_frontend/src/locales apps/hook_frontend/src/react-bits/components/landingnew apps/hook_frontend/src/react-bits/pages -S` → exit 0
- **Files changed**:
  - `apps/hook_frontend/src/app/(home)/page.tsx`
  - `apps/hook_frontend/src/locales/i18n-provider.tsx`
  - `apps/hook_frontend/src/locales/langs/cn/landing.json`
  - `apps/hook_frontend/src/locales/langs/en/landing.json`
  - `apps/hook_frontend/src/react-bits/pages/LandingPage.tsx`
  - `apps/hook_frontend/src/react-bits/components/landingnew/**`
- **Next step**: Milestone 3 — 运行前端校验并确认结果

## Milestone 3: 运行前端校验并确认结果

- **Status**: DONE
- **Started**: 15:32
- **Completed**: 15:41
- **What was done**:
  - 运行 `pnpm lint:frontend`
  - 根据 ESLint 输出修复首页任务相关的导入顺序、局部变量遮蔽和命名导入排序问题
  - 重新运行 `pnpm lint:frontend` 并确认通过
- **Validation**: `pnpm lint:frontend` → exit 0
- **Files changed**:
  - `apps/hook_frontend/src/app/(home)/page.tsx`
  - `apps/hook_frontend/src/locales/i18n-provider.tsx`
  - `apps/hook_frontend/src/react-bits/components/landingnew/Features/Features.tsx`
  - `apps/hook_frontend/src/react-bits/components/landingnew/Hero/Hero.tsx`
  - `apps/hook_frontend/src/react-bits/components/landingnew/LiveDemo/LiveDemo.tsx`
  - `apps/hook_frontend/src/react-bits/components/landingnew/Navbar/Navbar.tsx`
  - `apps/hook_frontend/src/react-bits/components/landingnew/Sponsors/Sponsors.tsx`
