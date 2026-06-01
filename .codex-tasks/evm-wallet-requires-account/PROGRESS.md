# Progress

## Recovery

任务: 移除登录页的未绑定钱包邮箱绑定登录流程
形态: single-full
进度: 4/4
当前: Backend and frontend changes are implemented and focused validation passed.
文件: .codex-tasks/evm-wallet-requires-account/TODO.csv
下一步: Edit wallet sign-in result, API response, routes, frontend sign-in UI, then validate.

## Notes

- Profile page already exposes authenticated wallet binding through `linkAccountWallet`.
- The visible login bug comes from unauthenticated `email_required` wallet result and the wallet email/code completion UI.
- User clarified that the EVM sign-in slot should become a RainbowKit connect/account button once the wallet is connected.
- Fixed first-connect race by making `signWalletMessage` use the account returned by `connectWalletAccount` instead of the previous render's wagmi `address`.
- Added a shared `WalletConnectControl` for sign-in and profile provider linking.
