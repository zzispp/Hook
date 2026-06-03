# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 让当前生效的首页落地页接入项目现有 `react-i18next` 多语言方案。
- 让首页 metadata、导航、Hero、功能区、演示区、部署区、生态区、CTA、Footer 等用户可见文案随语言切换。

## Non-Goals

- 不替换当前首页所使用的 `react-bits` 落地页架构。
- 不修改后台 `admin` 命名空间或后端 i18n 接口。
- 不新增静默降级、多余兼容层或第二套首页语言系统。

## Constraints

- 复用现有 `I18nProvider`、`useTranslate`、`getServerTranslations` 和 `common.json` 资源。
- 首页实现保持在现有 `apps/hook_frontend/src/react-bits/components/landingnew/**` 组件体系内。
- 变更后需通过前端静态校验，优先使用现有 lint / build 命令验证。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js App Router
- **Package manager**: pnpm
- **Test framework**: 前端当前以 lint / build 为主
- **Build command**: `pnpm build:frontend`

## Risk Assessment

- [x] Breaking changes to existing code — 仅改首页文案取词路径，影响范围已定位
- [x] Long-running tests — 使用前端 lint / build 验证

## Deliverables

- 首页落地页组件改为从现有翻译资源取文案
- 首页 metadata 改为按当前语言生成
- 中英文首页翻译资源补齐

## Done-When

- [ ] 首页所有用户可见核心文案都不再硬编码在组件里
- [ ] 中英文访问首页时文案和 metadata 可随语言切换
- [ ] 前端 lint 或 build 校验通过

## Final Validation Command

```bash
pnpm lint:frontend
```
