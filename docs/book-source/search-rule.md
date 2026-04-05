# 搜索规则 (ruleSearch)

定义如何从搜索结果页提取书籍列表。

## 规则字段

```json
{
  "ruleSearch": {
    "bookList": "",
    "name": "",
    "author": "",
    "kind": "",
    "wordCount": "",
    "intro": "",
    "coverUrl": "",
    "bookUrl": ""
  }
}
```

## 字段说明

| 字段 | 必需 | 说明 |
|------|------|------|
| `bookList` | 是 | 书籍列表的选择器 |
| `name` | 是 | 书名选择器 |
| `author` | 否 | 作者选择器 |
| `bookUrl` | 是 | 书籍详情页链接 |
| `coverUrl` | 否 | 封面图片URL |
| `intro` | 否 | 简介 |
| `kind` | 否 | 分类/标签 |
| `wordCount` | 否 | 总字数 |
| `lastChapter` | 否 | 最新章节 |

## 示例

### CSS 选择器示例

```json
{
  "ruleSearch": {
    "bookList": ".search-list li",
    "name": "h3 a@text",
    "author": ".author@text##作者：",
    "bookUrl": "h3 a@attr[href]",
    "coverUrl": "img@attr[src]",
    "intro": ".intro@text",
    "wordCount": ".words@text##万字",
    "lastChapter": ".latest@text"
  }
}
```

### JSON API 示例

```json
{
  "searchUrl": "https://api.example.com/search?q=${key}&page=${page}",
  "ruleSearch": {
    "bookList": "@json:$.data.books",
    "name": "@json:$.title",
    "author": "@json:$.author",
    "bookUrl": "@json:$.book_id##https://example.com/book/${id}",
    "coverUrl": "@json:$.cover"
  }
}
```

## 高级技巧

### 数据清洗

用 `##` 移除不需要的文本：

```
.author@text##作者：  # 移除"作者："前缀
```

### URL 补全

相对路径自动补全，也可手动拼接：

```
bookUrl@attr[href]##${baseUrl}  # 基础URL + 相对路径
```

### 列表为空处理

如果一页只有一本书，`bookList` 可以省略：

```json
{
  "name": "h1.title@text",
  "author": ".info .author@text"
}
```
