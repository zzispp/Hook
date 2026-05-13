# 用户 Provider 与模型限制

## Goal

在 Hook 项目中为编辑用户 modal 增加允许的 provider 与允许的模型配置，默认不限制；按现有目录、代码、多语言规范实现，并检查 LLM proxy 与其他调用链是否需要同步调整。Aether 项目作为参考实现。

## Scope

- 读取 Hook 当前用户编辑、provider/model、i18n、proxy 代码路径。
- 对照 `/Users/bubu/ZwjProjects/Aether` 的相关实现。
- 实现前端编辑用户 modal 带搜索选择框。
- 若后端或 proxy 缺少数据结构、持久化或运行时约束，同步补齐。
- 运行可行验证命令。

## Non-goals

- 不引入 mock 成功路径、静默降级或兼容性补丁。
- 不改动与该需求无关的业务配置。
