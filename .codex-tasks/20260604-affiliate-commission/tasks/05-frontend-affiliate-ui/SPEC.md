# 前端返佣设置与邀请展示

## Goal

后台系统设置能配置 `affiliate_commission_percent`；注册入口能传递 URL aff；用户钱包中心能展示邀请链接和基础返佣摘要。

## Acceptance

- 后台设置表单提交 `affiliate_commission_percent`。
- 注册、OAuth、钱包注册 payload 携带 URL `aff`。
- 用户端展示 `/sign-up?aff=<affiliate_code>`、复制按钮、邀请人数、累计返佣。
- 管理员创建用户可显式填写 `referrer_aff_code`。
- 新增 admin/auth 文案写入后端 i18n seed JSON。
