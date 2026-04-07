# Dockerfile for aarch64 - using pre-built binary
# Build binary first: cargo build --release --target aarch64-unknown-linux-musl
# Build frontend first: cd web && npm install && npm run build

FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# Copy pre-built binary
COPY target/aarch64-unknown-linux-musl/release/reader-rust /app/reader-rust

# Copy frontend dist
COPY frontend/dist /app/web/dist

# Create storage directory
RUN mkdir -p /app/storage/assets

# Environment defaults
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV DATABASE_URL=sqlite:/app/storage/reader.db?mode=rwc
ENV STORAGE_DIR=/app/storage
ENV ASSETS_DIR=/app/storage/assets
ENV WEB_ROOT=/app/web/dist
ENV LOG_LEVEL=info

EXPOSE 8080

VOLUME ["/app/storage"]

CMD ["./reader-rust"]
