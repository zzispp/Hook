# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 编写一套真实脚本，直接连接本地 Postgres/Redis 与本地 Hook backend，验证 `global_models.usage_count` 会在真实模型调用后递增。
- 覆盖一条直连上游请求和一条 Ekan8 模型映射请求。
- 记录本地数据库中的前后计数、请求记录、候选记录和映射命中证据。

## Non-Goals

- 不改动前端页面。
- 不把上游 API key 或本次生成的 bearer token 写入源码或任务文件。
- 不扩展为完整 request-record 全矩阵回归。

## Constraints

- 直接使用本地 Docker 容器提供的 Postgres 与 Redis。
- 真实请求必须经过本地 Hook backend `/v1/*` 代理链路。
- 模型名与映射 alias 以当前上游实时返回结果为准，不用记忆值硬猜。

## Deliverables

- 任务目录下可执行的真实验证脚本。
- 运行产出的 `raw/results.json` 证据文件。
- 修复后真实环境下 `usage_count` 递增的结论。

## Done-When

- [ ] 脚本能自动探测上游模型、写入本地夹具、启动或复用本地 backend、发起真实请求并查询本地 DB。
- [ ] 直连模型请求后，对应 `global_models.usage_count` 明确递增。
- [ ] 映射模型请求后，对应 `global_models.usage_count` 明确递增，并保留映射命中证据。
