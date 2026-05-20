# Progress

## Recovery

任务: 新增普通用户“使用记录”
形态: single-full
进度: 5/5
当前: 已完成
文件: `.codex-tasks/20260520-user-usage-records/TODO.csv`
下一步: 无

## Log

- Created task artifacts.
- Inspected admin request record API, storage query, route guard, menu defaults, i18n seeds, and frontend table/toolbar. Current admin path returns provider and candidate details, so user path must use a dedicated DTO and endpoint.
- Added a dedicated user usage-record API at `GET /api/request-records` with current-user filtering and a restricted `UsageRecord` DTO.
- Added user dashboard route `/dashboard/usage-records`, menu code `usage_records`, API binding `usage_records_read`, and i18n nav/search strings. The page title is “使用记录”.
- Kept admin `/api/admin/request-records` and `/dashboard/admin/request-records` unchanged as “请求记录”.
- Added a storage test proving user usage records filter by `r.user_id_snapshot`, omit provider/request ID fields in JSON, and do not search/filter provider fields.
- Validation passed: `pnpm --filter hook_frontend lint`; `perl -e 'alarm 60; exec @ARGV' cargo check -p provider -p storage -p types`; `perl -e 'alarm 60; exec @ARGV' cargo test -p storage --test provider_request_records`; `perl -e 'alarm 60; exec @ARGV' cargo test -p backend migration::defaults`; `git diff --check`.
