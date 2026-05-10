# Progress

## 2026-05-10

- 用户要求 baseline 改为中文展示值，后续外语部署由管理员在后台自行调整。
- 已确认需要改动点：默认角色 seed、默认 API definitions、默认 menu definitions、前端 dashboard raw fallback。
- 已将内置角色名称和描述改为中文：`管理员`、`用户`。
- 已将默认 API 权限 `name` 改为中文展示值；权限 `code`、HTTP 方法、路径不变。
- 已将默认菜单分组 `subheader` 和菜单 `title` 改为中文展示值；菜单 `code`、路径、绑定不变。
- 已同步 `dashboard-menu-values.ts` 和 breadcrumb 根标题 fallback。
- 验证通过：`cargo fmt --all --check`、`perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend migration::defaults`、`pnpm --filter hook_frontend exec tsc --noEmit --pretty false`。
