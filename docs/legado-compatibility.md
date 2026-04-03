# Legado 书源兼容清单

面向 `reader-rust` 后端解析引擎的 Legado 兼容跟踪文档。

目标不是“为某一个书源打补丁”，而是按 Legado 规则语义分层补齐，让一类书源一起受益。

---

## 1. 兼容目标

- 目标：常见 Legado 书源达到 `80% - 90%` 可用率
- 范围：
  - 书源导入与识别
  - 搜索 / 发现页
  - 书籍详情
  - 目录
  - 正文
  - 规则模板
  - JS 兼容层
- 非目标：
  - 为单个站点写硬编码逻辑
  - 为某个源单独分支处理数据结构

---

## 2. 当前结论

当前后端已经能覆盖：

- 简单 HTML/CSS 规则源
- 一部分 JSONPath 规则源
- 一部分带 `{{...}}` 模板的 JSON 源
- 一部分依赖基础 `java.*` 的 JS 规则

当前后端仍未完整覆盖：

- Legado 的完整 JS / `java.*` 运行时
- 更复杂的 URL 配置语义
- 更完整的 XPath / JSONPath 语义
- 登录态 / CookieJar / WebView 相关能力
- 高级发现页、分页、二次请求规则

---

## 3. 已兼容

### 3.1 书源 URL 与历史数据修复

- [x] 清理 `bookSourceUrl` 中的控制字符
- [x] 统一规范化书源 URL 用于匹配和自动发现
- [x] 修复书架中历史坏链接：
  - `%3F` -> `?`
  - `%26` -> `&`
  - `%3D` -> `=`
- [x] 读取书架时自动修复旧链接
- [x] 保存书架前自动修复链接

涉及文件：

- [src/util/text.rs](/Users/mac/project/reder/reader-rust/src/util/text.rs)
- [src/service/book_service.rs](/Users/mac/project/reder/reader-rust/src/service/book_service.rs)
- [src/api/handlers/book.rs](/Users/mac/project/reder/reader-rust/src/api/handlers/book.rs)

### 3.2 相对 URL 解析

- [x] `bookUrl` 相对路径转绝对地址
- [x] `coverUrl` 相对路径转绝对地址
- [x] `tocUrl` 相对路径转绝对地址
- [x] `chapterUrl` 相对路径转绝对地址
- [x] 相对路径拼接时忽略 `bookSourceUrl` 里的 fragment（如 `#小说/`）
- [x] 保留 query，不再把 `?` 编码成 `%3F`

涉及文件：

- [src/parser/rule_engine.rs](/Users/mac/project/reder/reader-rust/src/parser/rule_engine.rs)
- [src/service/book_service.rs](/Users/mac/project/reder/reader-rust/src/service/book_service.rs)

### 3.3 JSONPath 列表语义

- [x] `$.data` 命中数组时自动展开数组元素
- [x] 搜索 / 发现页支持 JSON 数组项逐项解析
- [x] 目录 JSON 数组项逐项解析

涉及文件：

- [src/parser/jsonpath.rs](/Users/mac/project/reder/reader-rust/src/parser/jsonpath.rs)

### 3.4 `init` 作用域语义

- [x] `ruleBookInfo.init` 会切换后续规则的 JSON 解析根节点
- [x] `ruleToc.init` 会切换后续规则的 JSON 解析根节点

说明：

- 这已经对齐了 Legado 里大量 `init: "$.data"` 的通用用法
- 不再只是“执行 init”，而是“基于 init 结果继续解析”

涉及文件：

- [src/parser/rule_engine.rs](/Users/mac/project/reder/reader-rust/src/parser/rule_engine.rs)

### 3.5 模板插值

- [x] 支持 `{{$.field}}`
- [x] 支持 `{{@get:{key}}}`
- [x] 支持字符串中混合模板拼接
- [x] 已应用于：
  - 搜索结果
  - 发现页结果
  - 详情规则
  - 目录规则

示例：

- `"/detail?book_id={{$.book_id}}&source={{$.source}}"`
- `"🎯最新章节：{{$.last_chapter_title}}"`

涉及文件：

- [src/parser/rule_engine.rs](/Users/mac/project/reder/reader-rust/src/parser/rule_engine.rs)

### 3.6 基础 JS 兼容层

- [x] `java.ajax`
- [x] `java.md5Encode`
- [x] `java.timeFormat`
- [x] `java.androidId`
- [x] `java.deviceID`
- [x] `source.key`
- [x] `cookie.removeCookie`
- [x] `kv_get`
- [x] `kv_put`

涉及文件：

- [src/parser/js.rs](/Users/mac/project/reder/reader-rust/src/parser/js.rs)

### 3.7 发现页链路

- [x] `exploreBook` 请求链路打通
- [x] `ruleFindUrl` 相对路径解析
- [x] 发现页支持书源全局 header
- [x] 发现页 JSON 规则支持模板拼接

涉及文件：

- [src/service/book_service.rs](/Users/mac/project/reder/reader-rust/src/service/book_service.rs)
- [src/api/handlers/book.rs](/Users/mac/project/reder/reader-rust/src/api/handlers/book.rs)

---

## 4. 部分兼容

### 4.1 HTML 规则

- [x] 基础 CSS 选择器
- [x] Legado 一部分 `text.xxx` 语义
- [ ] 更复杂的属性提取组合仍需补强

涉及文件：

- [src/parser/html.rs](/Users/mac/project/reder/reader-rust/src/parser/html.rs)

### 4.2 XPath

- [ ] 当前仍偏弱
- [ ] 大多数场景仍回退到 HTML 逻辑
- [ ] 复杂 XPath 源不能认为已经兼容

涉及文件：

- [src/parser/rule_engine.rs](/Users/mac/project/reder/reader-rust/src/parser/rule_engine.rs)
- [src/parser/html.rs](/Users/mac/project/reder/reader-rust/src/parser/html.rs)

### 4.3 JS 运行时

- [x] 基础 `java.*`
- [ ] 更完整的 Legado JS API 仍缺失
- [ ] `java.put/get` 风格扩展未统一
- [ ] 复杂同步脚本、环境对象仍不完整

### 4.4 缓存与刷新

- [x] 目录刷新支持 `refresh=1`
- [x] 目录缓存可跳过
- [ ] 与所有 Legado 风格分页目录源的联动还没系统验证

### 4.5 书架旧数据兼容

- [x] 已支持自动修复一部分历史坏链接
- [ ] 若旧数据缺字段或字段来源已变，仍可能需要重新刷新书籍信息

---

## 5. 未兼容 / 待开发

### 5.1 更完整的 Legado JS / `java.*`

- [ ] `java.get`
- [ ] `java.put`
- [ ] `java.post`
- [ ] 更多时间 / 编码 / 解码工具
- [ ] 更完整的 cookie / header / storage 语义
- [ ] 更贴近 Legado 的 `baseUrl` / `input` / `result` 环境

### 5.2 登录态与 CookieJar

- [ ] `enabledCookieJar` 的完整语义
- [ ] 登录接口后的 Cookie 持久化
- [ ] 书源级登录态自动附带
- [ ] 需要登录才能看的详情 / 正文 / 发现页

### 5.3 WebView / 浏览器选项串

- [ ] `##$##` 更多配置格式
- [ ] `webView` 相关选项
- [ ] `loadWithBaseUrl`
- [ ] `singleUrl`
- [ ] 更复杂的 URL 配置串解析

### 5.4 XPath 语义

- [ ] 完整 XPath 列表提取
- [ ] XPath + 属性提取
- [ ] XPath 下的分页和节点级提取

### 5.5 规则上下文语义

- [ ] 更完整的 `@put/@get` 行为对齐
- [ ] 多层嵌套上下文
- [ ] 列表项上下文继承
- [ ] init 与局部上下文联合生效的复杂规则

### 5.6 正文高级语义

- [ ] 多页正文拼接的复杂规则
- [ ] `nextContentUrl` 的更多变种
- [ ] 正文前后处理链增强

### 5.7 发现页高级语义

- [ ] `exploreUrl` 中更复杂的 JS / 表达式
- [ ] 多分区、多布局规则统一支持
- [ ] 分页发现页的更多变体

---

## 6. 高频兼容断点

后面如果再出现“某个 Legado 源不工作”，优先按这几类排查，而不是先看具体站点：

1. URL 是否被污染
   - 控制字符
   - fragment
   - `%3F/%26/%3D`
2. JSONPath 是否选到了数组容器而不是元素
3. `init` 是否应该切换解析根
4. 模板 `{{...}}` 是否真的展开
5. `<js>` 是否依赖未实现的 `java.*`
6. 是否缺登录态 / Cookie / Header
7. 结果 URL 是否仍是相对路径

---

## 7. 下一步优先级

### P0

- [ ] 完整 CookieJar / 登录态兼容
- [ ] 扩充 `java.*` 高频 API
- [ ] 把 XPath 从“弱兼容”提升到“可用”

### P1

- [ ] 更完整的 URL 配置串解析
- [ ] 更完整的 `@put/@get` / 上下文语义
- [ ] 正文分页 / 多页拼接增强

### P2

- [ ] 建一套 Legado 兼容回归测试样例
- [ ] 用样例源覆盖：
  - HTML 源
  - JSON 源
  - JS 源
  - 登录源
  - 发现页源

---

## 8. 开发原则

以后做兼容时遵守这几条：

- 不按“书源名”写逻辑
- 不按“站点域名”写硬编码
- 一次修一种规则语义
- 让搜索 / 发现 / 详情 / 目录 / 正文共用同一层兼容能力
- 改完后优先补到引擎层，而不是只在接口层兜底

