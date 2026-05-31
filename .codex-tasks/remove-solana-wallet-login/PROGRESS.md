# Progress

## Current State

- Solana wallet login has been removed from admin settings, frontend sign-in, frontend wallet signing, public auth config, backend wallet auth service, setting storage/types, migrations, and i18n seed copy.
- EVM wallet login remains configurable and enabled through the existing EVM chain selection and statement fields.
- Historical removed-provider identity parsing/display has been removed as requested, so unsupported provider values surface as invalid provider data.

## Recovery

任务: 移除 Solana 钱包登录，仅保留 EVM
形态: single-full
进度: 6/6
当前: Complete
文件: .codex-tasks/remove-solana-wallet-login/TODO.csv
验证:
- cargo fmt --check --all
- cargo check -p backend
- cargo test -p setting wallet --lib
- cargo test -p user wallet --lib
- pnpm lint:frontend
- pnpm build:frontend
- git diff --check
- rg -n "solana|Solana" apps crates Cargo.toml Cargo.lock --glob '!target/**'
