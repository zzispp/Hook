# Progress

- 已确认当前导航只通过 `NAV_ICONS` 映射硬编码 `icon.*`。
- 已改成 dashboard 字符串 icon 直接用 Iconify 渲染，不再传 `NAV_ICONS` 映射。
- 菜单项 modal 的 icon 字段改成 Iconify 图标选择框，保存值就是真实 icon code。
- baseline 默认菜单 icon 已改成真实 Iconify code，并已执行 `migration up` 重建本地 DB。
