# Usage Record Duration Display

## Goal

用户侧使用记录表格复用管理员侧请求记录的首字节与总耗时文字颜色、进行中滚动计时效果，但用户侧行仍不可点击打开抽屉详情。

## Scope

- Inspect admin request record duration rendering.
- Apply the same duration display to `apps/hook_frontend/src/sections/usage-records`.
- Validate with frontend static checks where feasible.

## Out Of Scope

- Adding user-side request detail drawers.
- Changing backend APIs or request record polling semantics.
