# Progress

## 2026-06-05

- Started release packaging task after user asked to add Aether-style GitHub Releases with shell scripts and platform packages.
- Verified Aether v0.7.8 assets through GitHub API: four platform tarballs, `install.sh`, and `SHA256SUMS`.
- Confirmed Hook release package boundary: `hook_backend` embeds `apps/hook_frontend/out`; binary packages still require externally managed PostgreSQL and Redis.
- Added release workflow for stable tags and beta/rc tags. Stable tags create normal GitHub Releases; beta/rc tags create prereleases.
- Added release assets: root `install.sh`, `scripts/package-release.sh`, `README_RELEASE.md`, and `packaging/config.example.yaml`.
- Added README and docs-site sections for GitHub Release binary packages while keeping Docker Compose Source Build as the recommended full deployment path.
- Validation completed: workflow YAML parse, shell syntax checks, actionlint for all workflows, release package dry run with expected tar layout, and checksum matching for installer lookup.
