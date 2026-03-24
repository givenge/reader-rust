# reader-rust

基于reader重构 https://github.com/hectorqin/reader

Rust 版 `reader` 服务端，基于 axum + tokio + reqwest + sqlx(SQLite) + rquickjs。

## 运行环境

- Rust 1.75+（建议使用最新稳定版）
- SQLite

## 打包与运行

### 运行（开发模式）

```bash
cargo run
```

### 构建（Release）

```bash
cargo build --release
```

### 或者使用musl静态编译

> 需要在 Linux 环境或 Docker 中进行，macOS 不支持。

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl --release
```

### musl静态编译后的二进制文件

```bash
target/x86_64-unknown-linux-musl/release/reader-rust
```

### 运行（Release）

```bash
./target/release/reader-rust
```

## 配置

支持环境变量覆盖。

默认值：

### Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

### Database (SQLite)
DATABASE_URL=sqlite:storage/reader.db?mode=rwc

### Storage paths
STORAGE_DIR=storage
ASSETS_DIR=storage/assets

### Web frontend path (adjust if needed)
WEB_ROOT=web/dist

### Logging
LOG_LEVEL=info

### Request timeout in seconds
REQUEST_TIMEOUT_SECS=15

### Security settings
SECURE=false
SECURE_KEY=

### User registration
INVITE_CODE=
USER_LIMIT=50
USER_BOOK_LIMIT=2000