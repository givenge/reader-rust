# 规则语法

## CSS 选择器

默认解析方式，用于从 HTML 中提取内容。

### 基本语法

| 选择器 | 说明 |
|--------|------|
| `#id` | ID选择器 |
| `.class` | 类选择器 |
| `tag` | 标签选择器 |
| `parent > child` | 子选择器 |
| `[attr]` | 属性选择器 |

### 内容提取

规则可以用 `@` 后缀指定提取内容：

| 后缀 | 说明 | 示例 |
|------|------|------|
| `@text` | 提取文本 | `h1@text` |
| `@html` | 提取 HTML | `div.content@html` |
| `@textNodes` | 提取所有文本节点 | `div@textNodes` |
| `@attr[attr]` | 提取属性 | `a@attr[href]` |

### 链式选择

用 `##` 分隔多个选择器：

```
.book-list##h3##a@attr[href]
```

### 联合查询

用 `&&` 在列表项中提取多个字段：

```json
{
  "name": "h3@text",
  "author": ".author@text",
  "coverUrl": "img@attr[src]"
}
```

## JSONPath

用于解析 JSON API 响应。

### 基本语法

| 表达式 | 说明 |
|--------|------|
| `$` | 根对象 |
| `$.data` | 访问 data 属性 |
| `$.data.list` | 访问 data.list |
| `$..name` | 递归查找 name |
| `$.data[0]` | 数组第一个元素 |

### 示例

```json
{
  "bookList": "$.data.list",
  "name": "$.title",
  "author": "$.author_name"
}
```

## XPath

用于 XML 或复杂的 HTML 结构。

### 基本语法

| 表达式 | 说明 |
|--------|------|
| `//tag` | 选择所有 tag |
| `/html/body/div` | 绝对路径 |
| `//div[@class='book']` | 属性选择 |
| `//div/text()` | 提取文本 |

### 显式指定

```
@xpath://div[@class='book']
```

## 正则表达式

用于从文本中提取内容。

### 语法

以 `:` 开头或使用 `@regex:` 前缀：

```
:@"(.*?)":  # 提取双引号中的内容
@regex:(\d+)章  # 提取章节数字
```

### 组提取

使用捕获组，`$1` 表示第一个组：

```
name:(.+?)(?:作者|【|第)  # $1 获取书名
```

## JavaScript

用于复杂的处理逻辑。

### 基本语法

以 `js:` 或 `@js:` 开头：

```
js:result.replace(/<[^>]+>/g, '')
```

### 可用变量

| 变量 | 说明 |
|------|------|
| `result` | 上一阶段的处理结果 |
| `baseUrl` | 基础URL |
| `html` | 页面 HTML 内容 |

### 完整示例

```json
{
  "content": "js:result.replace(/<script[^>]*>[\\s\\S]*?<\\/script>/gi, '')"
}
```

## URL 参数

搜索 URL 支持占位符：

| 占位符 | 说明 |
|--------|------|
| `${key}` | 搜索关键词 |
| `${page}` | 页码 |
| `${baseUrl}` | 书源基础 URL |

示例:
```
https://example.com/search?q=${key}&page=${page}
```
