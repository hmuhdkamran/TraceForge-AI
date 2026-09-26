# Multi-stage Dockerfile for TraceForge AI ContractGuard

# --- Frontend build ---
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package*.json ./
RUN npm ci --quiet

COPY frontend/ .
RUN npm run build

# --- Rust backend build ---
FROM rust:1.75-slim-bookworm AS backend-builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/migrations ./migrations

RUN cargo build --release

# --- Final image ---
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 contractguard

# Copy backend binary
COPY --from=backend-builder /app/target/release/contractguard /app/contractguard

# Copy migrations
COPY --from=backend-builder /app/migrations /app/migrations

# Copy frontend build output
COPY --from=frontend-builder /app/frontend/dist /app/static

# Copy sample project
COPY sample_project /app/sample_project

# Create workspace and data directories
RUN mkdir -p /app/runs /app/data && chown -R contractguard:contractguard /app

USER contractguard

ENV DATABASE_URL=sqlite:///app/data/contractguard.db
ENV BIND_ADDR=0.0.0.0:8080
ENV WORKSPACE_ROOT=/app/runs
ENV SAMPLE_PROJECT_ROOT=/app/sample_project
ENV ALLOWED_ORIGINS=*

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s \
  CMD wget -qO- http://localhost:8080/api/v1/health || exit 1

CMD ["/app/contractguard"]
