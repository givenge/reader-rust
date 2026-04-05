# 正文规则 (ruleContent)

定义如何从章节页提取正文内容。

## 规则字段

```json
{
  "ruleContent": {
    "content": "",
    "nextContentUrl": "",
    "title": ""
  }
}
```

## 字段说明

| 字段 | 必需 | 说明 |
|------|------|------|
| `content` | 是 | 正文内容选择器 |
| `nextContentUrl` | 否 | 下一页正文链接(分页章节) |
| `title` | 否 | 章节标题选择器 |

## 内容清洗

正文通常需要清洗广告、脚本等多余内容：

### 简单替换

```json
{
  "content": ".content@html##<script[^>]*>[\\s\\S]*?<\\/script>##"
}
```

### 多步替换

```json
{
  "content": ".content@html##<p>##\\n##</p>####<script[^>]*>[\\s\\S]*?<\\/script>"
}
```

### JavaScript 处理

更复杂的清理逻辑：

```json
{
  "content": "js:result.replace(/<script[^>]*>[\\s\\S]*?<\\/script>/gi, '').replace(/<br\\s*\\/?>/gi, '\\n').replace(/<\\/p>/gi, '\\n').replace(/<[^>]+>/g, '').trim()"
}
```

## 示例

### 标准正文

```json
{
  "ruleContent": {
    "content": ".read-content p@text##\\n##\\n##\\n##\\n",
    "title": ".chapter-title@text"
  }
}
```

### HTML 转纯文本

```json
{
  "ruleContent": {
    "content": ".content@textNodes",
    "title": "h1@text"
  }
}
```

### 分页正文

如果一章分多页：

```json
{
  "ruleContent": {
    "content": ".article-content@textNodes",
    "nextContentUrl": ".next-page@attr[href]##@js:result && result.includes('下一页') ? result : ''"
  }
}
```

## 最佳实践

1. **保留换行**: 用 `\n` 替换 `<p>` 或 `<br>` 保持段落格式
2. **去除广告**: 提前过滤脚本、推广内容
3. **处理分页**: 如网站有分页，设置 `nextContentUrl`
4. **标题补充**: 如果页面没有标题，可以用 `title` 规则提取
