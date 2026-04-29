# syntax=docker/dockerfile:1.7

# Stage 1: chef base — installs cargo-chef once on top of the pinned Rust image.
# Using the same base for planner and builder keeps the toolchain identical
# across both, so the dep-cache layer stays valid.
FROM rust:1.95-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: planner — produces a recipe.json describing the dependency graph.
# Only Cargo.toml/Cargo.lock content shapes the recipe, so this stage's output
# is stable across pure source-code edits, which keeps the next stage cached.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: builder — cooks deps from recipe.json (the heavy work, cached
# whenever Cargo.lock is unchanged), then builds the binary.
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --bin blog

# Stage 4: runtime — minimal Debian with ca-certificates (TLS roots) and curl
# (HEALTHCHECK only). Runs as an unprivileged user.
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system blog \
    && useradd --system --gid blog --home-dir /app --shell /usr/sbin/nologin blog
WORKDIR /app
COPY --from=builder --chown=blog:blog /app/target/release/blog /usr/local/bin/blog
COPY --from=builder --chown=blog:blog /app/static ./static
COPY --from=builder --chown=blog:blog /app/content ./content
ENV PORT=3000 APP_ENV=production
USER blog:blog
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS "http://127.0.0.1:${PORT}/healthz" || exit 1
CMD ["/usr/local/bin/blog"]
