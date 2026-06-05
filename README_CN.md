# Hook

<p align="center">
  <img src="apps/hook_frontend/public/logo/logo.svg" width="240" alt="Hook Logo">
</p>

<p align="center">
  <strong>AI API 网关与运营管理平台</strong>
</p>

<p align="center">
  <a href="README.md">English</a> • 中文
</p>

---

## 简介

Hook 是一个 Rust 与 pnpm monorepo，用于构建 AI API 网关、用户控制台和运营管理后台。后端基于 Axum、SeaORM、Redis 和 PostgreSQL，负责代理、鉴权、调度、计费、监控和管理接口；前端基于 Next.js、React、MUI 和 TypeScript，提供用户与管理员界面。

用户通过 Hook 令牌访问 `/v1` 或 `/v1beta`，系统按模型、供应商、分组、钱包余额和权限策略完成路由、计费、记录和监控。

## 功能

- **统一 AI 代理入口**：提供 OpenAI 风格 `/v1` 和 Gemini 风格 `/v1beta`，覆盖聊天、Responses、Claude Messages、图片、Embedding、Rerank、音频、Moderations 和 Realtime。
- **供应商与模型管理**：维护全局模型、供应商、端点、上游 API Key、模型绑定、模型成本、冷却释放和模型连通性测试。
- **令牌与权限控制**：支持用户令牌、管理员令牌、RBAC 角色、菜单、API 权限、导航权限和系统令牌策略。
- **钱包与计费**：包含用户钱包、余额、流水、日模型用量、后台调账、后台充值、计费分组、用户分组和价格分组。
- **充值、卡密与返佣**：支持充值套餐、支付渠道、支付回调记录、卡密生成与兑换、返佣关系、佣金和报表导出。
- **请求记录与成本分析**：记录客户端和上游请求，提供使用记录、活跃请求、用户统计、成本预测、成本节省和聚合统计。
- **模型状态与运维监控**：提供模型状态检查、定时任务、缓存亲和性监控、系统性能监控和健康检查。
- **账户与运营后台**：支持注册、登录、刷新令牌、OAuth、钱包登录、密码重置、个人资料、公告、工单、通知和站点设置。
- **后台国际化资源**：Admin 文案由后端 `translation_languages` 与 `translation_entries` 提供，通过 `/api/i18n/resources` 加载。

## 项目结构

```text
.
├── apps/
│   ├── hook_backend/      # Axum 后端入口、LLM 代理、迁移、监控和调度
│   └── hook_frontend/     # Next.js 前端控制台和管理后台
├── crates/
│   ├── api_token/         # API 令牌
│   ├── model/             # 全局模型目录
│   ├── provider/          # 供应商、端点、Key、模型绑定
│   ├── wallet/            # 钱包和流水
│   ├── recharge/          # 充值、支付回调、返佣
│   ├── rbac/              # 角色、菜单、API 权限
│   └── ...                # 配置、存储、调度、监控、用户等共享模块
├── config/config.yaml     # 默认本地配置
├── Dockerfile             # 多阶段镜像，内嵌前端静态产物
├── docker-compose.yml     # PostgreSQL、Redis 和 Hook 源码构建部署
├── deploy.sh              # Docker Compose 源码构建部署的一键安装脚本
├── update.sh              # Docker Compose 源码构建部署的一键更新脚本
├── package.json           # pnpm 工作区脚本
├── Cargo.toml             # Rust workspace
└── justfile               # Rust 检查、构建、迁移命令
```

品牌资源：

- `apps/hook_frontend/public/logo/logo.svg`：完整 logo 和文字。
- `apps/hook_frontend/public/logo/logo-icon.svg`：icon 版本。

## 部署

### Docker Compose Source Build

当前稳定版本推荐使用 Docker Compose 部署。它会从源码构建 Hook，编译内嵌前端静态产物，启动 PostgreSQL 和 Redis，执行 `migration up`，然后在 `http://127.0.0.1:5555` 提供前端和 API。

```bash
git clone https://github.com/zzispp/Hook.git
cd Hook
./deploy.sh
```

`./deploy.sh` 首次运行会创建 `.env`，要求输入管理员用户名、邮箱和密码，然后启动 Docker Compose。PostgreSQL 密码、JWT secret、provider key 加密密钥会自动生成。PostgreSQL 和 Redis 的运行数据保存在 `hook-postgres` 与 `hook-redis` Docker named volumes。

常用部署命令：

```bash
docker compose logs -f hook
docker compose ps
docker compose down
```

`docker compose down` 只停止容器，不删除 named volumes。只有明确要删除部署数据时才执行 `docker compose down -v`。

### 一键更新

Docker Compose 部署后，可在部署目录直接执行：

```bash
./update.sh
```

`update.sh` 会执行 `git pull --ff-only`，拉取 PostgreSQL 和 Redis 基础镜像，使用当前源码重新构建 Hook 镜像并重建容器。它不会删除 Docker named volumes。

### 不使用 Docker 的源码构建

源码构建部署使用同一套内嵌前端路径，不依赖 Docker。执行迁移前，需要先启动 PostgreSQL 和 Redis，并让 `config.yaml` 指向它们。

```bash
pnpm install
cp config/config.yaml config.yaml
scripts/generate-password-hash.sh "your-password"
# 将输出的密码 hash、数据库、Redis 和密钥配置写入 config.yaml。
cargo run -p hook_backend -- migration up
pnpm build:frontend:embedded
cargo run -p hook_backend
```

## 本地开发

依赖：Rust edition 2024 toolchain、Node.js `>=22.12.0`、pnpm `10.33.4`、PostgreSQL 和 Redis。默认配置连接 `localhost:5433` 与 `localhost:6380`。

安装依赖、准备配置、初始化数据库、生成嵌入式前端静态产物并启动开发服务：

```bash
pnpm install
cp config/config.yaml config.yaml
scripts/generate-password-hash.sh "your-password"
cargo run -p hook_backend -- migration up
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm build:frontend:embedded
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm dev
```

开发服务地址：后端 `http://127.0.0.1:5555`，前端 `http://127.0.0.1:8082`。

## 构建、配置与数据库

```bash
pnpm build:frontend
pnpm build:backend
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm build:backend:embedded
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm start:embedded
```

主要配置项包括 `server.*`、`database.*`、`redis.*`、`jwt.secret`、`security.provider_key_secret`、`admin.*`、`auth.*`、`tracing.log_level`、`NEXT_PUBLIC_SERVER_URL` 和 `HOOK_BACKEND_URL`。

迁移命令：

```bash
cargo run -p hook_backend -- migration up       # 应用 baseline，不清空已有完整表
cargo run -p hook_backend -- migration status   # 查看 baseline 表状态
cargo run -p hook_backend -- migration down     # 删除 baseline 表和迁移标记
cargo run -p hook_backend -- migration fresh    # 删除后重建 baseline
cargo run -p hook_backend -- migration refresh  # 删除后重建 baseline
cargo run -p hook_backend -- migration reset    # 删除 baseline 表和迁移标记
```

## API 入口

- `GET /health`：健康检查。
- `/api/*`：控制台、管理后台、鉴权、钱包、计费、供应商、模型、监控等业务接口。
- `/v1/*`：OpenAI、Claude、Jina 兼容代理入口。
- `/v1beta/*`：Gemini 兼容代理入口。

常见代理路由包括 `/v1/models`、`/v1/chat/completions`、`/v1/responses`、`/v1/messages`、`/v1/images/generations`、`/v1/embeddings`、`/v1/rerank`、`/v1/realtime`、`/v1beta/models/{model}:generateContent` 和 `/v1beta/models/{model}/embedContent`。

## 测试与校验

```bash
just check
just lint
just test
pnpm lint:frontend
pnpm build:frontend
```

## 许可证

当前仓库未包含许可证文件，授权范围未声明。
