# syntax=docker/dockerfile:1.7

# Rust version is configurable at build time, while defaulting to latest stable tag.
ARG RUST_VERSION=1

# =============================================================================
# STAGE 1: Chef base for dependency planning/caching
# =============================================================================
FROM rust:${RUST_VERSION}-slim-bookworm AS chef

# Install only build/runtime tooling needed during compile and health probes.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# cargo-chef lets Docker cache dependency compilation separately from app code.
RUN cargo install cargo-chef --locked

# Keep all following paths predictable for cargo, sqlx, and runtime assets.
WORKDIR /app

# =============================================================================
# STAGE 2: Planner creates the dependency recipe
# =============================================================================
FROM chef AS planner

# Copy project files; .dockerignore keeps target/, .env, and git data out.
COPY . .

# Generate a dependency recipe from Cargo.toml/Cargo.lock/source layout.
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# STAGE 3: Dependencies are built once and reused by dev/prod targets
# =============================================================================
FROM chef AS deps

# Copy only the dependency recipe first to maximize Docker layer caching.
COPY --from=planner /app/recipe.json recipe.json

# Compile dependencies without compiling the final backend binary.
RUN cargo chef cook --release --recipe-path recipe.json

# =============================================================================
# STAGE 4: Development image with hot-reload tools
# =============================================================================
FROM deps AS development

# cargo-watch restarts the API on code changes; sqlx-cli prepares/migrates local DB.
RUN cargo install cargo-watch --locked \
    && cargo install sqlx-cli --locked --no-default-features --features rustls,postgres

# Copy initial files so the image can start even before bind mounts are attached.
COPY . .

# Default development port and structured Rust logs.
ENV PORT=8080
ENV RUST_LOG=info

# Document the container port exposed by the Axum server.
EXPOSE 8080

# Compose overrides this with sqlx database setup + cargo watch.
CMD ["cargo", "watch", "-w", "src", "-w", "migrations", "-w", "Cargo.toml", "-x", "run --bin backend"]

# =============================================================================
# STAGE 5: Production builder
# =============================================================================
FROM deps AS builder

# SQLx query macros need either .sqlx offline cache or a DATABASE_URL at build time.
ARG SQLX_OFFLINE=false
ARG DATABASE_URL

# Apply SQLx build mode only inside the build stage; these values are not in runtime.
ENV SQLX_OFFLINE=${SQLX_OFFLINE}
ENV DATABASE_URL=${DATABASE_URL}

# Copy the full application source after dependency layers are cached.
COPY . .

# Build a size-optimized release binary and strip debug symbols.
RUN cargo build --release --bin backend \
    && strip target/release/backend

# =============================================================================
# STAGE 6: Minimal non-root production runtime
# =============================================================================
FROM debian:bookworm-slim AS production

# Install only certificates and curl for outbound TLS and Docker HEALTHCHECK.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Create a locked-down non-root user for the running API process.
RUN groupadd --system --gid 1001 app \
    && useradd --system --uid 1001 --gid app --home-dir /app --shell /usr/sbin/nologin app

# Runtime working directory; migrations and assets are resolved from here.
WORKDIR /app

# Copy the compiled backend binary with non-root ownership.
COPY --from=builder --chown=app:app /app/target/release/backend /app/backend

# Copy database migrations because the API runs them at startup.
COPY --from=builder --chown=app:app /app/migrations /app/migrations

# Copy runtime assets served by /api/assets/:filename.
COPY --from=builder --chown=app:app /app/src/Assets /app/src/Assets

# Run the backend as the non-root user created above.
USER app

# The app reads PORT from env and binds to 0.0.0.0 internally.
ENV PORT=8080
ENV RUST_LOG=info

# Expose the API port for Docker/Compose metadata.
EXPOSE 8080

# Mark the container unhealthy if the backend health endpoint stops responding.
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/system/health || exit 1

# Start the production binary.
CMD ["/app/backend"]
