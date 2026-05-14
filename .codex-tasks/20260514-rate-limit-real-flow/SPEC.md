# 2026-05-14 Rate Limit Real Flow

## Goal

核对并修复 Hook 代理里用户、用户令牌、Provider Key 三层速率限制的运行时生效链路，并用真实请求验证。

## Scope

- 检查当前代码里用户 `rate_limit_rpm`、用户令牌 `rate_limit_rpm`、Provider Key `rpm_limit` 的实际执行路径。
- 实现运行时速率限制，覆盖用户层、令牌层、用户+令牌叠加、单用户多令牌共享用户限制、Provider Key 限制。
- 把 Provider Key 表单里的 `RPM` 文案改成“速率限制(请求/分钟)”，默认值改为 `0`，含义为不限制。
- 复用现有真实代理夹具，跑真实 upstream 请求验证限制是否真的生效。

## Constraints

- 不写 mock 成功路径。
- 限制未生效或真实验证失败时必须显式暴露，不做静默 fallback。
- 不把会话里给出的上游密钥写入源码或任务文件。
