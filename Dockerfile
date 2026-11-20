# UniCloud Portal (Rust). cargo-chef caches the dependency build in its own layer, so a code change
# rebuilds only the portal itself. Migrations and the SQL are compiled in; no database at build time.

FROM rust:1-slim-bookworm AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /src

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
COPY config/ config/
RUN cargo build --release --locked && strip target/release/unicloud-portal

# Run stage: slim Debian, CA certificates (for TLS to the database), the binary and the assets
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/unicloud-portal /app/unicloud-portal
COPY assets/ /app/assets/
ARG GIT_SHA=dev
ENV GIT_SHA=$GIT_SHA \
    APP_ASSETS_DIR=/app/assets \
    PORT=8080 \
    RUST_LOG=info,sqlx=warn
EXPOSE 8080
USER 1000:1000
ENTRYPOINT ["/app/unicloud-portal"]
