# Deploy Hook To 50.16.57.26

## Goal

Deploy the Hook Rust backend with embedded static frontend to server `50.16.57.26`, without Docker.

## Boundary

- Install and configure system Nginx, PostgreSQL, and Redis.
- Build the app from the current repository and run it as a systemd service.
- Use Nginx as the public reverse proxy.
- Validate the deployed HTTP health endpoint and public domain path.

## Known Inputs

- SSH key: `/Users/bubu/Downloads/hook.pem`
- Server IP: `50.16.57.26`
- Cloudflare DNS: `api` A record to `50.16.57.26`, proxied.
- Backend bind target: `127.0.0.1:5555`.
- Production service should use embedded frontend assets.

## Assumptions To Verify

- SSH username is discoverable by probing common cloud users.
- Server OS package manager is discoverable after SSH login.
- The Cloudflare hostname is likely `api.<zone>`, but the root zone is not provided locally.
