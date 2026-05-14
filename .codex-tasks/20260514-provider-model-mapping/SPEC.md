# 提供商详情模型映射与端点改写收口

## 背景

当前仓库的 Provider 详情里已经接入了模型映射基础链路，但还有三个关键问题：

- 端点管理里的 `body_rules` 只存在于配置与前端，运行时代理链路没有执行。
- 模型映射当前是 `1:N`，运行时会从多个映射名里再做一次选择，和“单个客户端模型稳定映射到单个上游模型”的目标不一致。
- 代理成功响应会把真实上游 `model` 名称返回给客户端，可能暴露 provider 侧模型名。

## 目标

在 Provider 详情的模型列表下方保留“模型映射”功能，但收口为更明确的单映射模型：

- 每个客户端模型最多绑定一个上游模型映射，改成严格 `1:1`。
- 模型映射支持编辑一个映射级 `reasoning_effort` 覆盖。
- 端点 `body_rules` 必须真正接入运行时请求改写链路。
- 成功响应与流式响应中的 `model` 字段改回客户端原始请求模型，避免把真实上游模型名暴露给客户端。
- 映射级覆盖先执行，端点 `body_rules/header_rules` 后执行；端点层作为更下游、更具体的 provider/endpoint 规则。

## 非目标

- 不引入 aether/new-api 那种通用多字段 override DSL。
- 不改造全局模型管理页面。
- 不新增静默 fallback、mock 返回或兼容层。

## 实现约束

- 复用现有 `provider_models.provider_model_mappings` 字段，不新增表。
- 保持 Provider 详情现有目录和交互风格。
- 管理端多语言文案写入后端 i18n seed。
- body_rules 运行时行为必须基于当前代码里已有的规则结构实现，不新增另一套规则语义。

## 验证

- Rust 单测覆盖 body_rules 运行时、单映射选择、reasoning_effort 覆盖与响应 model 回写。
- 前端 lint / 构建检查新增组件与类型。
- 后端编译、测试通过，且映射更新会触发 scheduling snapshot refresh。
