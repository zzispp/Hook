# Progress

## Current State

- Backend and frontend implementation is complete.
- EVM and Solana wallet sign-in settings are split into separate admin sections.
- EVM network selection is limited to ETH/BSC/ARB and persists as `auth_evm_chain_ids`.
- Wallet signature domain is derived from `public_base_url`; the old wallet domain field is no longer part of the settings shape.
- EVM/Solana enablement requires a configured public base URL in frontend validation and backend service validation.
- Admin i18n seed copy has been updated for the new labels and validation messages.

## Recovery

任务: 调整区块链快捷登录配置
形态: single-full
进度: 5/5
当前: Complete
文件: .codex-tasks/blockchain-auth-provider-settings/TODO.csv
验证:
- cargo fmt --check --all
- cargo check -p backend
- cargo test -p types public_base_url_domain --lib
- cargo test -p setting wallet --lib
- pnpm lint:frontend
- pnpm build:frontend
