# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- API 权限可以挂到 0..N 个菜单。
- 未挂菜单的 API 是非菜单 API。
- 角色授权时可以同时选择菜单和 API。
- 后端 API 授权使用角色菜单携带 API 与角色直授权 API 的并集。
- 开发阶段直接重写 baseline，不保留旧 DB 兼容迁移。

## Non-Goals

- 不做旧数据库兼容迁移。
- 不引入兜底权限逻辑。
- 不改变“前端路由由后端菜单返回”的原则。

## Constraints

- 管理员默认不应该有用户钱包中心。
- 用户角色默认拥有钱包中心和钱包 API。
- authenticated base API 仍不通过菜单或角色授权表达。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + Next.js
- **Package manager**: cargo / pnpm
- **Test framework**: cargo test / ESLint / Next build

## Deliverables

- `role_api_permissions` schema/entity/repository/service/API/frontend 回归为一等绑定。
- `menu_api_permissions` 保留为菜单自动携带 API。
- baseline fresh 后可直接创建最终 schema 和 seed。

## Done-When

- [ ] 后端授权快照包含菜单 API 与角色 API 并集。
- [ ] 角色授权界面能保存菜单和 API。
- [ ] API 管理界面能保存 API 所属菜单。
- [ ] 本地 DB fresh 成功。
- [ ] 验证命令通过。

## Final Validation Command

```bash
just test && pnpm lint:frontend && pnpm build:frontend
```
