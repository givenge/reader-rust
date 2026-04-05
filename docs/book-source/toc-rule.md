# 目录规则 (ruleToc)

定义如何从目录页提取章节列表。

## 规则字段

```json
{
  "ruleToc": {
    "chapterList": "",
    "chapterName": "",
    "chapterUrl": "",
    "nextTocUrl": "",
    "updateTime": ""
  }
}
```

## 字段说明

| 字段 | 必需 | 说明 |
|------|------|------|
| `chapterList` | 是 | 章节列表的选择器 |
| `chapterName` | 是 | 章节标题选择器 |
| `chapterUrl` | 是 | 章节链接选择器 |
| `nextTocUrl` | 否 | 下一页目录链接 |
| `updateTime` | 否 | 更新时间选择器 |

## 示例

### 标准目录页

```json
{
  "ruleToc": {
    "chapterList": ".chapter-list li",
    "chapterName": "a@text",
    "chapterUrl": "a@attr[href]"
  }
}
```

### 分页目录

如果目录有多页：

```json
{
  "ruleToc": {
    "chapterList": ".chapter-list li",
    "chapterName": "a@text",
    "chapterUrl": "a@attr[href]",
    "nextTocUrl": ".next-page@attr[href]"
  }
}
```

### 正序目录

有些网站章节是倒序的，可以用 JS 处理：

```json
{
  "ruleToc": {
    "chapterList": "js:Array.from(document.querySelectorAll('.chapter-list li')).reverse()",
    "chapterName": "a@text",
    "chapterUrl": "a@attr[href]"
  }
}
```

## 常见问题

### 章节链接是相对路径

自动处理，如需要手动拼接：

```
chapterUrl@attr[href]##${baseUrl}${result}
```

### 混合内容(章节+目录)

有些目录页杂糅了推荐内容，需要过滤：

```json
{
  "chapterList": ".chapter-list li:not(.ad)"
}
```

或者用 JavaScript 过滤：

```json
{
  "chapterList": "js:result.filter(item => item.querySelector('a'))"
}
```
