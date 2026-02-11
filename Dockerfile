# Dockerfile for Battlestar Server - Fly.io deployment
# Build context must be the repo root (fly.toml sets this)

# Build stage
FROM rust:1.93.0-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace root files
COPY Cargo.toml ./Cargo.toml
COPY game-constants.toml ./game-constants.toml

# Copy shared crate (server depends on it)
COPY shared ./shared

# Copy server crate
COPY server/Cargo.toml ./server/Cargo.toml
COPY server/build.rs ./server/build.rs
COPY server/src ./server/src

# Create a dummy client crate so workspace resolves
RUN mkdir -p client/src && \
    printf '[package]\nname = "battlestar-client"\nversion = "0.1.0"\nedition = "2021"\n' > client/Cargo.toml && \
    touch client/src/lib.rs

# Build server with release optimizations
RUN cargo build --release -p battlestar-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/battlestar-server /app/battlestar-server

# Expose port
EXPOSE 3000

# Run the binary
CMD ["/app/battlestar-server"]
