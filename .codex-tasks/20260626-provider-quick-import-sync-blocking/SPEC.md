# Provider Quick Import Sync Blocking Fix

## Goal

修复快捷导入提供商定时同步中，单个 provider 因上游认证失败后阻塞同批其他 provider 执行/通知的问题。

## Scope

- 排查 provider quick import sync 定时任务的批处理控制流
- 先补失败用例，再修复实现
- 运行相关 Rust 测试与静态检查
