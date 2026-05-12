# Progress

- Started: 2026-05-12
- Status: In progress
- Scope: Request detail trace timeline.

## Notes

- Candidate list must remain billing-group scoped.
- Aether uses large provider dots and small provider-key attempt dots.
- Local DB translation_entries was missing the newly seeded trace keys; synced the requestRecords trace keys for cn/en.
- Fixed not-started attempts to render only "未开始" instead of "未开始 -> 进行中".
- Fixed selected key-dot clipping by centering the key-dot strip and reserving enough width/padding for the first selected dot.
- Real request through http://127.0.0.1:8082/v1/chat/completions returned success for gpt-5.5. New request 019e1a1d-d648-7583-a9ea-0bf7b0990a18 recorded 14 candidates across 2 providers: test and ikuncode.
- Validation: cargo check -p backend, pnpm build:frontend, targeted frontend eslint, and storage provider request tests passed.
