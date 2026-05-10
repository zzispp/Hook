# Progress

## 2026-05-10

- Root cause: `admin_menu_codes()` returned every default menu item, so the admin role received user-facing `dashboard_models`, `wallet_center`, and `api_tokens`.
- Token labels collide in Chinese because `apiTokens` and `adminApiTokens` both resolve to `令牌管理`.
