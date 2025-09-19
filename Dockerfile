# `cargo-chef` is a cargo-subcommand that provides
# enhanced Docker layer caching for Rust projects.
FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /usr/src/app

# Sync Rust toolchain. Ensure it builds with correct version.
COPY rust-toolchain.toml .
RUN rustup toolchain install

FROM chef AS planner
COPY . .
# Compute a lock-like file for project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json

# Build project's dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin itil-back-web

FROM debian:bookworm-slim AS runtime

# Create non-root user for security
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "itil"

USER itil:itil

COPY --from=builder --chown=itil:itil /usr/src/app/target/release/itil-back-web /usr/local/bin/itil-back-web

ENV APP_ENVIRONMENT=production
ENV APP_SERVER__PORT=3000
ENV APP_SERVER__IP=0.0.0.0

ENTRYPOINT ["/usr/local/bin/itil-back-web"]
EXPOSE 3000
