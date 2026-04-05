---
outline: deep
---

# 书源开发概述

书源是 Reader-Rust 的核心，定义了如何从网站获取书籍信息、目录和正文。

## 书源结构

书源是一个 JSON 对象，包含以下主要部分：

```json
{
  "bookSourceName": "书源名称",
  "bookSourceUrl": "https://example.com",
  "bookSourceType": 0,
  "enabled": true,
  "ruleSearch": { /* 搜索规则 */ },
  "ruleBookInfo": { /* 书籍信息规则 */ },
  "ruleToc": { /* 目录规则 */ },
  "ruleContent": { /* 正文规则 */ }
}
```

## 解析方式

Reader-Rust 支持多种解析方式：

| 方式 | 标识 | 用途 |
|------|------|------|
| CSS选择器 | 默认/ `@css:` | HTML网页解析 |
| JSONPath | `@json:` 或自动检测 | JSON API解析 |
| XPath | `/` 开头或 `@xpath:` | XML/HTML解析 |
| 正则 | `@regex:` 或 `:`开头 | 文本提取 |
| JavaScript | `js:` 或 `@js:` | 复杂逻辑处理 |

## 规则流程

```
搜索关键词
    ↓
ruleSearch → 书籍列表 → 选择书籍
    ↓
ruleBookInfo → 书籍详情
    ↓
ruleToc → 章节列表
    ↓
ruleContent → 章节正文
```

## 调试技巧

1. 使用 `/reader3/testBookSource` 接口测试书源
2. 检查 `log_level=debug` 的日志输出
3. 使用浏览器开发者工具分析网页结构

## 下一步

- [规则语法](./rules) - 了解每种解析方式的语法
- [搜索规则](./search-rule) - 编写搜索功能
- [书籍信息](./book-info) - 提取书籍详情
- [目录规则](./toc-rule) - 获取章节列表
- [正文规则](./content-rule) - 获取章节内容
- [JavaScript](./javascript) - 使用 JS 处理复杂逻辑
