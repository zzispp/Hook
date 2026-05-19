# Progress

## Recovery

任务: Deploy Hook to 50.16.57.26
形态: single-full
进度: 0/6
当前: Step 1, local build and artifact verification
文件: `.codex-tasks/20260519-deploy-hook/TODO.csv`
下一步: Run local embedded frontend build and release backend build.

## Log

- Created task artifacts after confirming backend embeds `apps/hook_frontend/out` through `rust-embed` and serves on `server.host:server.port`.
- Step 1 done: `pnpm build:frontend:embedded` passed and `cargo build -p backend --release` passed locally. Local binary is Mach-O arm64, so server deployment must build or receive a Linux binary.
- Step 2 done: SSH works as `ubuntu` using `/tmp/hook-key-normalized.pem`. Server is Ubuntu 26.04 LTS x86_64 with 3.8 GiB RAM and 75 GiB free disk.
- Step 3 done: Installed and enabled Nginx 1.28.3, PostgreSQL 18.3, Redis 8.0.5, and Linux build dependencies.
- Step 4 done: Built Linux backend binary, created production PostgreSQL database/user, wrote `/etc/hook/config.yaml` for `api.hook.rs`, generated bcrypt hash for the supplied admin password, and applied migrations. Migration status reports 37/37 baseline tables present.
- Step 5 done: Installed `/etc/systemd/system/hook-backend.service`, enabled it, and verified `http://127.0.0.1:5555/health` returns `status: ok`.
- Step 6 done: Configured HTTPS for `api.hook.rs`, Cloudflare-only origin access in Nginx and UFW, and 600s proxy timeouts. Verified `https://api.hook.rs/health` through Cloudflare returns `status: ok`; direct origin HTTP/HTTPS is blocked.
