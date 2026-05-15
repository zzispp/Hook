# Progress

## Recovery

任务: 在 Hook 请求详情中补齐与 sub2api 一致的计费详情展示
形态: single-full
进度: 4/4
当前: Completed
文件: .codex-tasks/request-billing-details/TODO.csv
结果: 后端记录服务档位、成本拆分和单价字段；前端请求详情按现有 CurrencyDisplay 货币规则展示计费详情；backend seed i18n 已补齐。

## Verification

- cargo test -p provider billing
- cargo test -p storage --test provider_request_records
- cargo test -p storage --test provider_request_candidates
- cargo test -p storage --test provider_request_housekeeping
- cargo check -p backend
- pnpm lint:frontend
- pnpm build:frontend

备注: pnpm build:frontend 期间输出了既有的非致命 `Axios error: Something went wrong!`，命令退出码为 0。
