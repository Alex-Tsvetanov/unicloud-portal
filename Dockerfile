# UniCloud Portal (Rust). cargo-chef caches the dependency build in its own layer, so a code change
# rebuilds only the portal itself. Migrations and the SQL are compiled in; no database at build time.
# A Node stage builds the React app (served at /app) and packs @unicloud/ui (served at /ui/v1).

FROM node:22-bookworm-slim AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
COPY web/packages/ui/package.json packages/ui/
COPY web/apps/portal/package.json apps/portal/
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build && npm run pack:ui && cp packages/ui/dist/unicloud.css dist-ui/unicloud.css

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
COPY api/ api/
RUN cargo build --release --locked && strip target/release/unicloud-portal

# Run stage: slim Debian, CA certificates (for TLS to the database), the binary and the assets
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /src/target/release/unicloud-portal /app/unicloud-portal
COPY assets/ /app/assets/
COPY --from=web /src/web/apps/portal/dist/ /app/web/
COPY --from=web /src/web/dist-ui/ /app/ui/
ARG GIT_SHA=dev
ENV GIT_SHA=$GIT_SHA \
    APP_ASSETS_DIR=/app/assets     APP_WEB_DIR=/app/web     APP_UI_DIR=/app/ui \
    PORT=8080 \
    RUST_LOG=info,sqlx=warn
EXPOSE 8080
USER 1000:1000
ENTRYPOINT ["/app/unicloud-portal"]
