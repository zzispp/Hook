# 用户创建邀请码与邀请关系绑定

## Goal

普通注册、OAuth 首次创建、管理员创建用户都生成 `affiliate_code`。普通注册/OAuth 首次创建/管理员显式填写时绑定邀请人，管理员默认创建不绑定。

## Acceptance

- 新用户返回自己的 `affiliate_code`。
- 有效 `aff_code` 绑定 `referred_by_user_id`。
- 无效 `aff_code` 返回明确错误。
- 替换用户不会修改邀请归属。
