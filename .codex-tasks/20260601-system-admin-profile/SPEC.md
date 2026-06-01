# 配置管理员账号自助能力限制

## 目标
配置文件级系统管理员（User.system = true）在用户侧 Profile 只保留只读账号信息，不允许改密码、绑定 provider、验证邮箱或解绑 provider。

## 边界
- 保留普通用户现有 profile 功能。
- 后端服务层必须拒绝系统用户相关自助变更，不能只依赖前端隐藏。
- 不触碰上一个 auth layout/profile RBAC 任务的 staged 改动。
