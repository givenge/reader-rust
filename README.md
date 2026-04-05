# Reader-Rust

基于 [reader](https://github.com/hectorqin/reader) 重构的 Rust 版书源阅读服务端。

## 文档

完整文档见：https://givenge.github.io/reader-rust/

## 快速开始

```bash
# 克隆项目
git clone https://github.com/givenge/reader-rust.git
cd reader-rust

# 运行后端
cargo run

# 或运行前端开发服务器
cd frontend
npm install
npm run serve
```

## 技术栈

- 后端：Rust + axum + tokio + reqwest + sqlx (SQLite) + rquickjs
- 前端：Vue 3 + Vite + TypeScript + Pinia

## 功能特性

- 自定义书源支持
- CSS 选择器、JSONPath、XPath、正则、JavaScript 多种解析方式
- 自动化规则引擎
- 书籍搜索与目录获取
- 章节内容缓存
- RSS 订阅支持
- TTS 语音朗读