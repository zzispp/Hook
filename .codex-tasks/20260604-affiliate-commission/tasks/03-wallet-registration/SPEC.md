# 钱包注册链路

## Goal

未知钱包可以通过签名、邮箱验证码、用户名完成注册，创建无密码账号，绑定钱包身份，并支持 aff 邀请关系。

## Acceptance

- 新增 `/api/auth/wallet/register`。
- 请求校验复用钱包 challenge 和注册邮箱验证码。
- 创建用户无密码，绑定 wallet identity，返回登录 session。
- 支持 `aff_code` 绑定邀请人。
