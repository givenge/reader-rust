# 书籍信息规则 (ruleBookInfo)

定义如何从书籍详情页提取详细信息。

## 规则字段

```json
{
  "ruleBookInfo": {
    "name": "",
    "author": "",
    "coverUrl": "",
    "intro": "",
    "kind": "",
    "wordCount": "",
    "lastChapter": "",
    "tocUrl": ""
  }
}
```

## 字段说明

| 字段 | 必需 | 说明 |
|------|------|------|
| `name` | 是 | 书名 |
| `author` | 是 | 作者 |
| `coverUrl` | 否 | 封面图片URL |
| `intro` | 否 | 书籍简介 |
| `kind` | 否 | 分类标签 |
| `wordCount` | 否 | 总字数 |
| `lastChapter` | 否 | 最新章节标题 |
| `tocUrl` | 否 | 目录页URL(默认当前页) |

## 示例

### 标准详情页

```json
{
  "ruleBookInfo": {
    "name": ".book-info h1@text",
    "author": ".book-info .author a@text",
    "coverUrl": ".book-cover img@attr[src]",
    "intro": ".book-intro@html",
    "kind": ".tags a@text",
    "wordCount": ".book-status .words@text",
    "lastChapter": ".book-status .latest a@text",
    "tocUrl": ".read-link@attr[href]"
  }
}
```

### 简介处理

简介可能需要清理 HTML 标签或换行：

```json
{
  "intro": "div.intro@html##<p>##\n##<br>##\n"
}
```

或使用 JavaScript：

```json
{
  "intro": "js:result.replace(/<[^>]+>/g, '\\n').trim()"
}
```

## 注意事项

1. `tocUrl` 如未指定，默认使用当前页面作为目录页
2. 某些网站目录在单独页面，需要指定 `tocUrl`
3. `bookUrl` 会自动作为入参传入 `ruleBookInfo` 上下文
