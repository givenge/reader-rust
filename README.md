# reader-rust

基于reader重构 https://github.com/hectorqin/reader

Rust 版 `reader` 服务端，基于 axum + tokio + reqwest + sqlx(SQLite) + rquickjs。

## 接口文档

- 后端接口整理文档：[/Users/mac/project/reder/reader-rust/docs/backend-api.md](/Users/mac/project/reder/reader-rust/docs/backend-api.md)

## 运行环境

- Rust 1.75+（建议使用最新稳定版）
- SQLite

## 前端说明

项目当前包含两套前端：

### 旧前端：`web/`

- 原版Reader前端改UI
- 基于 Vue 2
- 属于较早版本的 Web 客户端
- 默认构建产物目录为 `web/dist`
- 后端默认通过 `WEB_ROOT=web/dist` 提供静态文件服务

### 新前端：`frontend/`

- 基于 Vue 3 + Vite + TypeScript
- 是当前正在持续开发的新前端
- 已完成阅读页、服务器备份、书架拖拽排序、PWA、构建体积优化等新能力
- 适合后续功能迭代与界面改进

### 当前使用建议

- 如果要保持与当前后端默认配置一致，可继续使用旧前端 `web/`
- 如果要体验和开发新界面，请使用新前端 `frontend/`
- 如需让后端直接服务新前端构建结果，请将 `WEB_ROOT` 改为 `frontend/dist`

## 打包与运行

### 运行（开发模式）

```bash
cargo run
```

## Docker 部署

### 使用预编译镜像（推荐）

```bash
# 基础运行
docker run -d --restart=always --name=reader-rust \
  -v $(pwd)/storage:/app/storage \
  -p 8080:8080 \
  givenge/reader-rust:latest

# 使用环境变量配置
docker run -d --restart=always --name=reader-rust \
  -e "LOG_LEVEL=info" \
  -e "SECURE=true" \
  -e "SECURE_KEY=your-admin-password" \
  -e "INVITE_CODE=your-invite-code" \
  -v $(pwd)/storage:/app/storage \
  -p 8080:8080 \
  givenge/reader-rust:latest
```

### 自行编译镜像

```bash
# 克隆仓库
git clone https://github.com/givenge/reader-rust.git
cd reader-rust

# 构建镜像
docker build -t reader-rust:latest .

# 运行
docker run -d --restart=always --name=reader \
  -v $(pwd)/storage:/app/storage \
  -p 8080:8080 \
  reader-rust:latest
```

### 跨平台镜像构建

```bash
# 新建构建器
docker buildx create --use --name mybuilder

# 启动构建器
docker buildx inspect mybuilder --bootstrap

# 查看构建器及其所支持的 CPU 架构
docker buildx ls

# 构建跨平台镜像并推送
docker buildx build -t givenge/reader-rust:latest --platform=linux/arm64,linux/amd64 . --push
```

### 更新镜像

```bash
# 拉取最新镜像
docker pull givenge/reader-rust:latest

# 通过 watchtower 自动更新
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --cleanup --run-once reader
```

### 访问地址

- Web 端：http://localhost:8080/
- API 接口：http://localhost:8080/reader3/


### 构建（Release）

```bash
cargo build --release
```

### musl 静态编译

生成的二进制文件不依赖 glibc，可在任何 x86_64 Linux 上运行。

#### Linux 环境

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl --release
```

#### macOS 交叉编译

macOS 需要安装 musl-cross 工具链：

```bash
# 1. 安装 musl-cross
brew install FiloSottile/musl-cross/musl-cross

# 2. 添加 Rust target
rustup target add x86_64-unknown-linux-musl

# 3. 配置链接器（项目已包含 .cargo/config.toml）
# 如果没有，创建 .cargo/config.toml 内容如下：
# [target.x86_64-unknown-linux-musl]
# linker = "x86_64-linux-musl-gcc"

# 4. 编译
cargo build --target x86_64-unknown-linux-musl --release
```

> 注意：确保使用 rustup 安装的 Rust，而非 Homebrew 的 Rust。
> 当前Homebrew 的 Rust 不支持交叉编译 target。
> 如有冲突，临时用 rustup 的 cargo ~/.cargo/bin/cargo build --release --target x86_64-unknown-linux-musl 
> 或者卸载Homebrew 的 Rust `brew uninstall rust` 后使用 rustup 版本。

### musl 静态编译后的二进制文件

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
