# Build stage
FROM rust:1.75 AS builder

WORKDIR /app

# Install build essentials
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir backend/src && touch backend/src/lib.rs

# Download dependencies
RUN cargo fetch

# Copy source
COPY backend/src ./src

# Build release
RUN cargo build --release --bin lightweight-charts-backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/lightweight-charts-backend /app/

# Create data directory
RUN mkdir -p /app/data

# Expose port
EXPOSE 3000

# Environment
ENV RUST_LOG=info

# Run
CMD ["/app/lightweight-charts-backend"]