# GitHub Pages Site

## Goal

Create a GitHub Pages landing site for Hook that follows the existing landing page visual language and add a GitHub Actions workflow to deploy it.

## Scope

- Add a standalone static Pages site that does not depend on the main Next.js app or backend APIs.
- Reuse Hook logo assets and copy aligned with the existing landing page.
- Add a Pages deployment workflow that runs on stable release tags and manual dispatch.
- Validate the static assets and workflow locally.

## Validation

- The static site can be served locally.
- The Pages workflow YAML parses.
- The deployment artifact directory contains `index.html`, `404.html`, CSS, JS, and logo assets.
