# Progress

## 2026-06-05

- Confirmed the worktree is clean before starting.
- Confirmed the main frontend landing page uses dark glass panels, purple accent lighting, terminal/code surfaces, and `/logo/logo.svg` plus `/logo/logo-icon.svg`.
- Chose a standalone static `docs-site` instead of exporting the full Next.js app so GitHub Pages deployment stays independent from backend/API routes.
- Added `docs-site` as a static GitHub Pages site with product hero, project overview, features, Docker Compose Source Build install docs, one-command update docs, local development commands, API entrypoints, configuration/data notes, ecosystem, and acknowledgments.
- Added `.github/workflows/deploy-pages.yml` with stable `vX.Y.Z` tag filtering plus manual dispatch, and artifact upload from `docs-site`.
- Split static styling into `styles.css`, `docs.css`, and `responsive.css`; each file stays below the repository file-size limit.
- Validated workflow YAML, required static files, curl responses for HTML/CSS/JS/logo assets, `git diff --check`, desktop/mobile browser layout, logo masks, no mobile page overflow, and copy button DOM behavior.
- Restyled `docs-site` away from the landing-page aesthetic into a VitePress-style documentation site with white background, sticky topbar, left sidebar, constrained content column, tables, code blocks, and docs cards.
- Updated `site.js` so scroll highlighting targets the new top navigation and sidebar links.
- Revalidated workflow YAML, static file presence, served assets, `git diff --check`, desktop docs layout, mobile no-overflow behavior, logo switching, and visible copy-button error state when clipboard permission is denied.
- Added bilingual documentation support with English default `index.html`, Chinese `zh.html`, visible EN/中文 switch links, and localized copy-button idle/success/failure labels.
- Updated the Pages workflow artifact check to require `docs-site/zh.html`.
- Validated both language pages by static checks, curl responses, and browser checks for page language, active language state, mobile layout, and icon logo switching.
- Started adapting the bilingual docs site toward the provided New API/Fumadocs reference: compact fixed navigation tools, search entry, theme toggle, and a bordered gradient hero panel while preserving the existing documentation content.
- Updated `docs-site` to use a compact Fumadocs-like topbar with search, theme toggle, language switch, and GitHub icon; added a real static search dialog generated from `.doc-section` content and persistent light/dark theme switching.
- Reworked the hero panel into a bordered rounded gradient surface while replacing New API-inspired wording with Hook-specific positioning around model routing, access policy, billing, and observability.
- Added `docs-site/search.css` and updated the Pages workflow artifact preflight to require it.
- Validated workflow YAML, required static files, no trailing whitespace/conflict markers, no old New API-style hero copy, per-file line limits under 300, desktop English/Chinese hero copy, search result availability, theme state change, and mobile no-overflow layout.
- Verified search filtering in browser with `Docker`, which narrowed results to Installation, Update, and Configuration and Data.
- Started fixing GitHub Actions annotations: confirmed current CI used `actions/checkout@v4`, `actions/setup-node@v4`, `pnpm/action-setup@v4`, `extractions/setup-just@v2`, and `docker/build-push-action@v6`; verified newer `checkout@v6`, `setup-node@v6`, `pnpm/action-setup@v6`, and `docker/build-push-action@v7` run on Node 24, while `setup-just@v4` is composite.
- Updated CI to `actions/checkout@v6`, `actions/setup-node@v6`, `pnpm/action-setup@v6`, `extractions/setup-just@v4`, and `docker/build-push-action@v7`, with explicit `fetch-depth: 1`.
- Updated Pages deployment to `actions/checkout@v6`, `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5`, and `actions/deploy-pages@v5`.
- Added `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` to CI and Pages workflow env so GitHub Actions runs JavaScript actions under Node 24 immediately.
- Investigated the successful CI run annotations with `gh`: the `git exit code 128` warning came from checkout post cleanup running `git submodule foreach`, which failed because `.codex-tasks/20260512-login-register-captcha/raw/cap` was committed as a `160000` gitlink without a `.gitmodules` entry.
- Removed the bad task raw gitlink from the index and ignored `.codex-tasks/**/raw/*/` scratch repositories to prevent the same submodule cleanup warning from returning.
