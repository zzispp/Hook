# 管理员成本分析页面

## Goal

在概览菜单分组新增管理员成本分析页面，参考 Aether 成本分析模块，排除月卡消耗进度。

## Constraints

- 查询路径不得扫描 `request_records`。
- 使用聚合桶支撑高数据量读取。
- 不添加 mock、静默 fallback 或兼容补丁。
- 管理员页面排序在用户统计之后。

