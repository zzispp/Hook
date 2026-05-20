# 同 Provider 多 Key 降级修复

## Goal

排查 Hook 调度中同一个 provider 的不同 key 是否不会继续降级调用；对比 /Users/bubu/ZwjProjects/Aether 的实现，并按 Aether 语义修复。

## Constraints

- 不添加静默 fallback 或 mock 成功路径。
- 失败需要显式暴露。
- 优先写可复现测试验证同 provider 多 key 尝试。
