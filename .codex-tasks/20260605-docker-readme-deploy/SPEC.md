# Docker README Deployment

## Goal

Make Hook's Docker Compose deployment documentation and scripts match the first stable release deployment flow.

## Scope

- Initialize Docker deployment with administrator username, email, and password entered by the user.
- Generate PostgreSQL password, JWT secret, and provider key secret automatically.
- Add an update script for source-build Docker Compose deployments without deleting named volumes.
- Update README, README_CN, and landing quick-start commands to describe the real deployment path.

## Validation

- Shell syntax checks pass for deployment scripts.
- Docker Compose config resolves with generated env values.
- Frontend lint/build and backend cargo check pass.
- README and landing copy contain no Docker Run instructions.
