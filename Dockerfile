# Multi-stage build for Rust application
# Build argument to specify which binary to build
ARG BINARY=api-server

FROM rust:1.83 AS builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml ./
# Copy Cargo.lock if it exists (for reproducible builds)
COPY Cargo.lock* ./

# Copy source code
COPY src ./src

# Build argument to specify which binary to build
ARG BINARY=api-server

# Build the specified binary
RUN cargo build --release --bin ${BINARY}

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build argument to specify which binary to copy
ARG BINARY=api-server

# Copy binary from builder
COPY --from=builder /app/target/release/${BINARY} /app/${BINARY}

# Set binary name as environment variable for CMD
ENV BINARY=${BINARY}

# Create a non-root user
RUN useradd -m -u 1000 perptrix && chown -R perptrix:perptrix /app
USER perptrix

# Default command (can be overridden in docker-compose)
CMD ["sh", "-c", "./${BINARY}"]
