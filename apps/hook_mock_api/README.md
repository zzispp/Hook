# 🚀 Mock Server Setup Guide

📖 **Official Documentation:** [Minimal Docs - Quick Start](https://docs.minimals.cc/quick-start)

## 📌 Prerequisites

- Node.js >=20

## Installation

```sh
pnpm install
pnpm dev
```

## Default port

Mock server runs on: [http://localhost:7272](http://localhost:7272)

## Deploy on Vercel

- Step 1: Push code to GitHub
- Step 2: Create a new project on Vercel
- Step 3: Connect with GitHub
- Step 4: Select Framework preset: Next.js
- Step 5: Deploy

## Deploy on Cloudflare

- Step 1: Push code to GitHub
- Step 2: Create a new project on Cloudflare Pages
- Step 3: Connect with GitHub
- Step 4: Select Framework preset: Next.js

```sh
# Build command
npx @cloudflare/next-on-pages@1

# Build output directory
.vercel/output/static
```

- Step 5: Add Environment variables and deploy

```sh
NODE_VERSION=20
```

- Step 6: Configure Compatibility Flags: Workers & Pages > Project > Settings > Compatibility flags add `nodejs_compat`
- Step 7: Re-deploy
