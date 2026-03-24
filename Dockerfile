# Stage 1: Build frontend
FROM node:18-alpine AS frontend-builder

WORKDIR /app/web

# Install dependencies
COPY web/package.json web/package-lock.json* web/yarn.lock* ./
RUN npm install || yarn install

# Build frontend
COPY web/ ./
RUN npm run build || yarn build

# Stage 2: Build backend
FROM rust:1.94-alpine AS backend-builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build backend
RUN cargo build --release

# Stage 3: Runtime
FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/reader-rust /app/reader-rust

# Copy frontend dist
COPY --from=frontend-builder /app/web/dist /app/web/dist

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