# syntax=docker/dockerfile:1

FROM node:24-bookworm-slim AS frontend
WORKDIR /app
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_OPTIONS=--max-old-space-size=4096

RUN corepack enable && corepack prepare pnpm@10.33.4 --activate

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/hook_frontend/package.json apps/hook_frontend/package.json
RUN pnpm install --frozen-lockfile

COPY apps/hook_frontend apps/hook_frontend
COPY apps/hook_backend/src/migration/defaults/i18n apps/hook_backend/src/migration/defaults/i18n
RUN BUILD_STATIC_EXPORT=true pnpm --filter hook_frontend build

FROM rust:1-bookworm AS backend
WORKDIR /app
ARG TARGETOS
ARG TARGETARCH

COPY Cargo.toml Cargo.lock ./
COPY apps/hook_backend apps/hook_backend
COPY crates crates
COPY --from=frontend /app/apps/hook_frontend/out apps/hook_frontend/out

RUN --mount=type=cache,id=hook-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=hook-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=hook-target-${TARGETOS}-${TARGETARCH},target=/app/target,sharing=locked \
    cargo build --release -p hook_backend \
    && cargo build --release -p user --bin generate_password_hash \
    && mkdir -p /app/dist-bin \
    && cp target/release/hook_backend /app/dist-bin/hook_backend \
    && cp target/release/generate_password_hash /app/dist-bin/generate_password_hash

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend /app/dist-bin/hook_backend /usr/local/bin/hook_backend
COPY --from=backend /app/dist-bin/generate_password_hash /usr/local/bin/generate_password_hash
COPY scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

EXPOSE 5555
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
