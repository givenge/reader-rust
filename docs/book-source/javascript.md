# JavaScript 规则

JavaScript 规则允许你用代码处理复杂的解析逻辑。

## 基本语法

规则以 `js:` 或 `@js:` 开头：

```json
{
  "content": "js:result.replace(/<script[^>]*>[\\s\\S]*?<\\/script>/gi, '')"
}
```

## 可用变量

| 变量 | 类型 | 说明 |
|------|------|------|
| `result` | string/Element | 上一阶段结果 |
| `baseUrl` | string | 书源基础 URL |
| `html` | string | 原始 HTML |

## 常用操作

### DOM 操作

选择元素：

```js
document.querySelector('.content');
document.querySelectorAll('.chapter-list li');
```

获取属性：

```js
element.getAttribute('href');
element.textContent;
element.innerHTML;
```

### 字符串处理

```js
// 替换
result.replace(/pattern/g, '');

// 正则匹配
result.match(/chapter_(\d+)/)[1];

// URL 拼接
new URL(result, baseUrl).href;

// 分割
result.split('\\n').filter(Boolean);
```

### 数组操作

过滤列表：

```json
{
  "chapterList": "js:Array.from(result).filter(li => li.querySelector('a'))"
}
```

反转顺序：

```json
{
  "chapterList": "js:Array.from(result).reverse()"
}
```

映射字段：

```json
{
  "chapterList": "js:Array.from(document.querySelectorAll('a')).map(a => ({title: a.textContent, url: a.href}))"
}
```

## 条件判断

分页判断：

```json
{
  "nextContentUrl": "js:document.querySelector('.next')?.textContent?.includes('下一页') ? document.querySelector('.next').href : ''"
}
```

## 完整示例

处理复杂章节：

```json
{
  "ruleContent": {
    "content": "js:(() => { \\n      const div = document.querySelector('.content'); \\n      const scripts = div.querySelectorAll('script, .ad, .promo'); \\n      scripts.forEach(s => s.remove()); \\n      return div.innerHTML.replace(/<br\\s*\\/?>/gi, '\\n').replace(/<\\/p>/gi, '\\n').replace(/<[^>]+>/g, ''); \\n    })()",
    "title": "js:document.querySelector('.title')?.textContent?.replace(/.*第/, '第')"
  }
}
```

## 调试技巧

1. **测试片段**: 在浏览器控制台先测试代码
2. **打印日志**: 使用 `console.log`（在服务端日志中可见）
3. **错误处理**: 用 `try-catch` 包装复杂逻辑

## 性能提示

- JavaScript 解析比普通选择器慢
- 优先使用 CSS/XPath 选择器
- JS 仅用于处理选择器无法完成的复杂逻辑
