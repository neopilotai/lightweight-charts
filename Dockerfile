# Build stage (using pre-built binary for speed)
FROM debian:bookworm-slim AS builder

WORKDIR /app

# Copy pre-built binary
COPY backend/target/release/lightweight-charts-backend /app/

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/lightweight-charts-backend /app/

# Create data directory
RUN mkdir -p /app/data

# Expose port
EXPOSE 3000

# Environment
ENV RUST_LOG=info

# Run
CMD ["/app/lightweight-charts-backend"]