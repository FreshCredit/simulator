# Testnet Agent Bridge - Dockerfile
# Bridges agent requests to testnet nodes

ARG DEPS_IMAGE=us-central1-docker.pkg.dev/${PROJECT_ID}/websites/freshcredit-deps:latest
FROM ${DEPS_IMAGE} AS builder

USER root
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get upgrade -y && apt-get install -y pkg-config libssl-dev clang lld && rm -rf /var/lib/apt/lists/* || true

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY crates/ ./crates/
COPY apps/ ./apps/
COPY packages/ ./packages/
COPY workspace-hack/ ./workspace-hack/
COPY xtask/ ./xtask/
COPY migrations/ ./migrations/

# Build the testnet-agent-bridge binary
ENV CARGO_TARGET_DIR=/app/target-testnet-agent-bridge
RUN mkdir -p /app/target-testnet-agent-bridge/release
RUN cargo build --release --locked -p testnet-agent-bridge

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get upgrade -y && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r freshcredit && useradd -r -g freshcredit freshcredit

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target-testnet-agent-bridge/release/testnet-agent-bridge /app/testnet-agent-bridge

# Set ownership
RUN mkdir -p /app/data && chown -R freshcredit:freshcredit /app

USER freshcredit

ENV PORT=8080
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/testnet-agent-bridge"]
