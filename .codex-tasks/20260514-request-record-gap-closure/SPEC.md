# Request Record Gap Closure

## Goal

补齐请求记录体系剩余的 4 个能力缺口：

1. `request_candidates` 增加 `skip_reason`、`error_code`、`error_param`
2. candidate 状态从 `available/unused` 迁移为 `scheduled/skipped`
3. payload retention 从“过期清空”升级为“过期压缩保留，最终删除”
4. 增加 stale `pending/streaming` sweep，避免僵尸请求长期残留

## Constraints

- 不引入静默 fallback
- 保持 `request_records` 为权威主记录
- 前后端、storage、cleanup、测试一起对齐
- 现有工作区是脏的，不回滚无关改动

## Deliverables

- schema / entity / type / audit / frontend / cleanup / tests 全链路对齐
- 关键验证命令通过
