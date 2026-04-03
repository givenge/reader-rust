# Reader Rust 后端接口文档

基于当前后端代码整理，面向前端重写使用。路由来源见 [src/api/router.rs](/Users/mac/project/reder/reader-rust/src/api/router.rs)。

## 1. 基础约定

### 1.1 Base URL

- 开发默认地址：`http://127.0.0.1:8080`
- 业务接口前缀：`/reader3`

### 1.2 统一返回格式

除少数文件流、封面流、SSE、WebDAV 原生响应外，接口统一返回：

```json
{
  "isSuccess": true,
  "errorMsg": "",
  "data": {}
}
```

失败时通常为：

```json
{
  "isSuccess": false,
  "errorMsg": "错误信息",
  "data": null
}
```

对应实现见 [src/error/error.rs](/Users/mac/project/reder/reader-rust/src/error/error.rs)。

### 1.3 鉴权

大多数 `/reader3/*` 接口需要登录态，支持以下几种方式：

- Header `Authorization: Bearer <accessToken>`
- Header `Authorization: <accessToken>`
- Query `accessToken=<accessToken>`
- 管理接口可额外使用 Header/Query `X-Secure-Key` / `secureKey`
- 管理员代操作用户数据时可附带 Header/Query `X-User-NS` / `userNS`

鉴权提取逻辑见 [src/api/auth.rs](/Users/mac/project/reder/reader-rust/src/api/auth.rs)。

### 1.4 兼容旧前端的入参规则

若路由同时声明了 `GET` 和 `POST`，通常兼容以下传参方式：

- Query string
- JSON body
- `application/x-www-form-urlencoded`

尤其是书籍相关接口，后端会优先从 body 覆盖 query。

### 1.5 特殊错误码语义

虽然 HTTP 状态码仍会返回 4xx/5xx，但前端通常更需要关注 `errorMsg` 和部分 `data`：

- `errorMsg = "NEED_LOGIN"`: 需要登录
- `data = "NEED_LOGIN"`: 某些用户接口会这样返回
- `data = "NEED_SECURE_KEY"`: 需要管理员口令

## 2. 核心数据结构

### 2.1 Book

见 [src/model/book.rs](/Users/mac/project/reder/reader-rust/src/model/book.rs)。

```ts
type Book = {
  name: string
  author: string
  bookUrl: string
  origin: string
  originName?: string
  coverUrl?: string
  tocUrl?: string
  charset?: string
  customCoverUrl?: string
  canUpdate?: boolean
  durChapterIndex?: number
  durChapterPos?: number
  durChapterTime?: number
  durChapterTitle?: string
  intro?: string
  latestChapterTitle?: string
  lastCheckTime?: number
  totalChapterNum?: number
  type?: number
  group?: number
  wordCount?: string
  infoHtml?: string
  tocHtml?: string
  kind?: string
  updateTime?: string
}
```

### 2.2 SearchBook

见 [src/model/search.rs](/Users/mac/project/reder/reader-rust/src/model/search.rs)。

```ts
type SearchBook = {
  name: string
  author: string
  bookUrl: string
  origin: string
  coverUrl?: string
  intro?: string
  kind?: string
  lastChapter?: string
  updateTime?: string
  bookSourceUrls?: string[]
}
```

### 2.3 BookChapter

见 [src/model/book_chapter.rs](/Users/mac/project/reder/reader-rust/src/model/book_chapter.rs)。

```ts
type BookChapter = {
  title: string
  url: string
  index: number
}
```

### 2.4 BookSource

见 [src/model/book_source.rs](/Users/mac/project/reder/reader-rust/src/model/book_source.rs) 和 [src/model/rule.rs](/Users/mac/project/reder/reader-rust/src/model/rule.rs)。

常用字段：

```ts
type BookSource = {
  bookSourceName: string
  bookSourceGroup?: string
  bookSourceUrl: string
  bookSourceType?: number
  enabled?: boolean
  enabledExplore?: boolean
  customOrder?: number
  weight?: number
  searchUrl?: string
  exploreUrl?: string
  header?: string
  loginUrl?: string
  ruleSearch?: BookListRule
  ruleExplore?: BookListRule
  ruleBookInfo?: BookInfoRule
  ruleToc?: TocRule
  ruleContent?: ContentRule
}
```

### 2.5 其他模型

- `BookGroup`: `groupId`, `groupName`, `orderNo`
- `Bookmark`: `time`, `bookName`, `bookAuthor`, `chapterIndex`, `chapterPos`, `chapterName`, `bookText`, `content`
- `ReplaceRule`: `id`, `name`, `group`, `pattern`, `replacement`, `scope`, `isEnabled`, `isRegex`, `order`
- `RssSource`: RSS 源配置
- `RssArticle`: RSS 文章项

## 3. 接口清单

## 3.1 健康检查

### GET `/health`

- 鉴权：否
- 返回：`data = "ok"`

---

## 3.2 用户与认证

### POST `/reader3/login`

- 鉴权：否
- Body:

```json
{
  "username": "alice01",
  "password": "password123",
  "isLogin": true,
  "code": "邀请码，可选"
}
```

- 说明：
  - `isLogin = true` 表示登录
  - `isLogin = false` 表示注册
  - 首个注册用户自动成为管理员
- 返回 `data` 结构：

```json
{
  "username": "alice01",
  "lastLoginAt": 1710000000000,
  "accessToken": "alice01:xxxxx",
  "enableWebdav": false,
  "enableLocalStore": false,
  "createdAt": 1710000000000,
  "isAdmin": true
}
```

### POST `/reader3/logout`

- 鉴权：登录态
- 返回：`isSuccess = false`，`errorMsg = "请重新登录"`，`data = "NEED_LOGIN"`

### GET `/reader3/getUserInfo`

- 鉴权：可匿名
- 返回 `data`：

```json
{
  "userInfo": {
    "username": "alice01",
    "lastLoginAt": 1710000000000,
    "accessToken": "alice01:xxxxx",
    "enableWebdav": false,
    "enableLocalStore": false,
    "createdAt": 1710000000000,
    "isAdmin": false
  },
  "secure": true,
  "secureKey": true
}
```

说明：

- `userInfo` 未登录时可能为 `null`
- `secure` 表示系统是否开启账户体系
- `secureKey` 表示是否要求管理员口令

### POST `/reader3/saveUserConfig`

- 鉴权：登录态
- Body：任意 JSON
- 返回：空字符串
- 说明：服务端会自动写入 `@updateTime`

### GET `/reader3/getUserConfig`

- 鉴权：登录态
- 返回：用户配置 JSON 原样返回

### GET `/reader3/getUserList`

- 鉴权：管理员
- 返回：用户列表，元素格式同 `login.data`

### POST `/reader3/addUser`

- 鉴权：管理员
- Body:

```json
{
  "username": "bob01",
  "password": "password123"
}
```

- 返回：最新用户列表

### POST `/reader3/resetPassword`

- 鉴权：管理员
- Body:

```json
{
  "username": "bob01",
  "password": "newpassword123"
}
```

- 返回：空字符串

### POST `/reader3/updateUser`

- 鉴权：管理员
- Body:

```json
{
  "username": "bob01",
  "enableWebdav": true,
  "enableLocalStore": true
}
```

- 返回：最新用户列表

### POST `/reader3/deleteUsers`

- 鉴权：管理员
- Body:

```json
["bob01", "charlie01"]
```

- 返回：最新用户列表

---

## 3.3 书源管理

### GET/POST `/reader3/getBookSource`

- 鉴权：登录态
- 参数：
  - `bookSourceUrl: string`
- 返回：`BookSource`

### GET/POST `/reader3/getBookSources`

- 鉴权：登录态
- 返回：`BookSource[]`

### POST `/reader3/saveBookSource`

- 鉴权：登录态
- Body：`BookSource`
- 返回：

```json
{ "saved": true }
```

### POST `/reader3/saveBookSources`

- 鉴权：登录态
- Body 支持以下任意形式：

```json
[BookSource, BookSource]
```

```json
{ "bookSourceList": [BookSource, BookSource] }
```

也兼容键名：`bookSources`、`data`、`sources`。

- 返回：

```json
{ "saved": true, "count": 2 }
```

### POST `/reader3/deleteBookSource`

- 鉴权：登录态
- Body:

```json
{ "bookSourceUrl": "https://example.com" }
```

- 返回：`{ "deleted": true }`

### POST `/reader3/deleteBookSources`

- 鉴权：登录态
- Body:

```json
[
  { "bookSourceUrl": "https://a.com" },
  { "bookSourceUrl": "https://b.com" }
]
```

- 返回：`{ "deleted": true }`

### POST `/reader3/deleteAllBookSources`

- 鉴权：登录态
- 返回：`{ "deleted": true }`

### POST `/reader3/setAsDefaultBookSources`

- 鉴权：管理员
- Body:

```json
{ "username": "alice01" }
```

- 返回：

```json
{ "success": true, "count": 123 }
```

### POST `/reader3/readRemoteSourceFile`

- 鉴权：否
- Body:

```json
{ "url": "https://example.com/source.json" }
```

- 返回：`string[]`
- 说明：返回的是“书源数组序列化后的 JSON 字符串列表”，当前实现固定为单元素数组，例如：

```json
[
  "[{\"bookSourceName\":\"xxx\",\"bookSourceUrl\":\"https://...\"}]"
]
```

### POST `/reader3/readSourceFile`

- 鉴权：否
- Content-Type：`multipart/form-data`
- 表单：上传 `.json` 或 `.txt`
- 返回：直接返回 `BookSource[]`，不包 `ApiResponse`

---

## 3.4 搜索、书架、阅读

### GET/POST `/reader3/searchBook`

- 鉴权：登录态
- 参数：
  - `key: string` 必填
  - `page?: number` 默认 `1`
  - `bookSourceUrl?: string`
  - `bookSource?: BookSource`
- 返回：`SearchBook[]`
- 说明：
  - 必须提供 `bookSourceUrl` 或完整 `bookSource`
  - 若未提供，后端会尝试从书架或 URL 自动推断

### GET/POST `/reader3/searchBookMulti`

- 鉴权：登录态
- 参数：
  - `key: string` 必填
  - `page?: number`
  - `bookSourceUrls?: string[]`
  - `bookSourceGroup?: string`
- 返回：`SearchBook[]`
- 说明：
  - 返回会按 `name + author` 去重合并
  - 合并后可能多一个 `bookSourceUrls: string[]`

### GET/POST `/reader3/exploreBook`

- 鉴权：登录态
- 参数：
  - `ruleFindUrl: string` 必填
  - `page?: number`
  - `bookSourceUrl?: string`
  - `bookSource?: BookSource`
- 返回：`SearchBook[]`

### GET `/reader3/getBookshelf`

- 鉴权：登录态
- 返回：`Book[]`

### GET/POST `/reader3/getShelfBook`

- 鉴权：登录态
- 参数：
  - `url: string`
- 返回：单个 `Book`

### GET `/reader3/getShelfBookWithCacheInfo`

- 鉴权：登录态
- 返回：`Book[]`，每项额外附带：
  - `cachedChapterCount: number`

### POST `/reader3/saveBook`

- 鉴权：登录态
- Body：`Book`
- 最低要求：
  - `bookUrl` 必填
  - `origin` 必填
- 返回：保存后的 `Book`
- 说明：
  - 如果 `tocUrl` 或 `name` 缺失，后端会尝试实时拉取书籍详情补齐

### POST `/reader3/setBookSource`

- 鉴权：登录态
- 支持 query/json/form
- 参数：
  - `bookUrl` 或 `url`: 原书架书籍 URL
  - `newUrl`: 新书源下的书籍 URL
  - `bookSourceUrl`: 新书源 URL
- 返回：切换后的 `Book`
- 说明：
  - 若旧 `bookUrl` 与新 `bookUrl` 不同，旧书架记录会被删除

### POST `/reader3/deleteBook`

- 鉴权：登录态
- Body：`Book`
- 返回：`"删除书籍成功"`

### POST `/reader3/deleteBooks`

- 鉴权：登录态
- Body：`Book[]`
- 返回：

```json
{ "deleted": 2 }
```

### POST `/reader3/saveBookProgress`

- 鉴权：登录态
- 支持 query/json
- 参数：
  - `url?: string`
  - `bookUrl?: string`
  - `index: number` 必填
  - `position?: number`
  - `searchBook?: { bookUrl?: string }`
- 返回：空字符串
- 说明：
  - `url / bookUrl / searchBook.bookUrl` 三者取其一
  - 会同步刷新 `durChapterIndex`、`durChapterPos`、`durChapterTitle`、`durChapterTime`

### GET/POST `/reader3/getBookInfo`

- 鉴权：登录态
- 支持 query/json/form
- 参数：
  - `url: string` 必填
  - `bookSourceUrl?: string`，别名 `origin`
  - `bookSource?: BookSource`
- 返回：`Book`

### GET/POST `/reader3/getChapterList`

- 鉴权：登录态
- 支持 query/json/form
- 参数：
  - `tocUrl?: string`
  - `bookUrl?: string`，别名 `url`
  - `bookSourceUrl?: string`，别名 `origin`
  - `bookSource?: BookSource`
- 返回：`BookChapter[]`
- 说明：
  - `tocUrl` 与 `bookUrl` 至少一个必填
  - 有章节缓存
  - 如果目录存在分页，接口先返回第一页，再在后台继续抓取剩余章节并追加缓存

### GET/POST `/reader3/getBookContent`

- 鉴权：登录态
- 支持 query/json/form
- 参数：
  - `chapterUrl?: string`，别名 `url`、`href`
  - `bookSourceUrl?: string`，别名 `origin`
  - `bookSource?: BookSource`
  - `index?: number`
  - `refresh?: number`
- 返回：章节正文字符串
- 说明：
  - 若 `chapterUrl` 实际上传入的是书籍 URL，且同时带 `index`，后端会先取目录，再按索引定位章节
  - `refresh > 0` 会清理缓存后重新抓取

### POST `/reader3/deleteBookCache`

- 鉴权：登录态
- 支持 query/json/form
- 参数：
  - `bookUrl?: string`
  - `url?: string`
  - `chapterUrl?: string`
- 实际删除依据：
  - 以 `bookUrl` 为主
  - 没有 `bookUrl` 时退化为 `url`
- 返回：

```json
{
  "deleted": true,
  "contentCache": true,
  "chapterListCache": true
}
```

### POST `/reader3/getInvalidBookSources`

- 鉴权：登录态
- 返回：数组，元素结构不固定

### GET/POST `/reader3/getAvailableBookSource`

- 鉴权：登录态
- 支持 query/json
- 参数：
  - `url?: string`
  - `name?: string`
  - `author?: string`
  - `refresh?: number`
- 返回：`SearchBook[]`
- 说明：
  - 优先按 `url` 查书架书
  - 查不到再按 `name + author` 查
  - 结果会写入书源候选缓存

### GET `/reader3/cover`

- 鉴权：否
- 参数：
  - `path: string`
- 返回：封面图片二进制流
- 失败：`404`

### GET `/reader3/getTxtTocRules`

- 鉴权：否
- 返回：空数组 `[]`

---

## 3.5 SSE 接口

SSE 返回 `Content-Type: text/event-stream`。

### GET/POST `/reader3/cacheBookSSE`

- 鉴权：登录态
- 参数：
  - `url?: string`
  - `bookUrl?: string`
  - `refresh?: number`
  - `concurrentCount?: number` 默认 `24`

普通事件 `data`：

```json
{
  "cachedCount": 10,
  "successCount": 2,
  "failedCount": 0
}
```

结束事件：

- `event: end`
- `data` 同上

### GET `/reader3/searchBookMultiSSE`

- 鉴权：登录态
- 参数：
  - `key: string`
  - `bookSourceUrl?: string`
  - `bookSourceGroup?: string`
  - `lastIndex?: number` 默认 `-1`
  - `searchSize?: number` 默认 `50`
  - `concurrentCount?: number` 默认 `24`

普通事件 `data`：

```json
{
  "lastIndex": 3,
  "data": [SearchBook]
}
```

错误事件：

- `event: error`
- `data: { "errorMsg": "..." }`

结束事件：

- `event: end`
- `data: { "lastIndex": 3 }`

### GET `/reader3/searchBookSourceSSE`

- 鉴权：登录态
- 参数：
  - `url: string`
  - `bookSourceGroup?: string`
  - `lastIndex?: number` 默认 `-1`
  - `searchSize?: number` 默认 `30`
  - `refresh?: number`

事件格式同上。

### GET `/reader3/bookSourceDebugSSE`

- 鉴权：登录态
- 参数：
  - `bookSourceUrl: string`
  - `keyword: string`

事件格式：

- 普通消息：`{ "msg": "start search" }`
- 结果批次：`{ "data": [SearchBook] }`
- 错误：`event: error` + `{ "errorMsg": "..." }`
- 结束：`event: end` + `{ "lastIndex": 0 }`

---

## 3.6 分组管理

### GET/POST `/reader3/getBookGroups`

- 鉴权：登录态
- 返回：`BookGroup[]`

### POST `/reader3/saveBookGroup`

- 鉴权：登录态
- Body：`BookGroup`
- 返回：`"success"`

### POST `/reader3/saveBookGroupOrder`

- 鉴权：登录态
- Body：`BookGroup[]`
- 返回：`"success"`

### POST `/reader3/deleteBookGroup`

- 鉴权：登录态
- Body:

```json
{ "groupId": 1 }
```

- 返回：`"success"`

### POST `/reader3/saveBookGroupId`

- 鉴权：登录态
- Body:

```json
{
  "bookUrl": "https://example.com/book/1",
  "groupId": 1
}
```

- 返回：`"success"`

### POST `/reader3/addBookGroupMulti`

- 鉴权：登录态
- Body:

```json
{
  "bookUrls": ["url1", "url2"],
  "groupId": 4
}
```

- 返回：`"success"`
- 说明：`group` 按位或，兼容旧版 Reader 的位标记分组

### POST `/reader3/removeBookGroupMulti`

- 鉴权：登录态
- Body 同上
- 返回：`"success"`
- 说明：`group` 按位清除

---

## 3.7 RSS

### GET `/reader3/getRssSources`

- 鉴权：登录态
- 返回：`RssSource[]`

### POST `/reader3/saveRssSource`

- 鉴权：登录态
- Body：`RssSource`
- 要求：
  - `sourceUrl` 非空
  - `sourceName` 非空
- 返回：空字符串

### POST `/reader3/saveRssSources`

- 鉴权：登录态
- Body：`RssSource[]`
- 返回：空字符串

### POST `/reader3/deleteRssSource`

- 鉴权：登录态
- Body：`RssSource`
- 实际按 `sourceUrl` 删除
- 返回：空字符串

### GET/POST `/reader3/getRssArticles`

- 鉴权：登录态
- 当前实现只读 JSON body，不读 query
- Body:

```json
{
  "sourceUrl": "https://example.com/rss.xml",
  "sortName": "默认",
  "sortUrl": "https://example.com/rss.xml",
  "page": 1
}
```

- 返回：

```json
{
  "first": [RssArticle],
  "second": null
}
```

- 说明：
  - 分页大小固定 `50`
  - `sortUrl` 为空时回退到 `sourceUrl`

### GET/POST `/reader3/getRssContent`

- 鉴权：登录态
- 当前实现只读 JSON body，不读 query
- Body:

```json
{
  "sourceUrl": "https://example.com/rss.xml",
  "link": "https://example.com/article/1",
  "origin": "https://example.com/rss.xml"
}
```

- 返回：文章 HTML 字符串

---

## 3.8 书签

### GET `/reader3/getBookmarks`

- 鉴权：登录态
- 返回：`Bookmark[]`

### POST `/reader3/saveBookmark`

- 鉴权：登录态
- Body：`Bookmark`
- 返回：空字符串
- 说明：以 `bookName + bookAuthor` 为唯一键覆盖

### POST `/reader3/saveBookmarks`

- 鉴权：登录态
- Body：`Bookmark[]`
- 返回：空字符串

### POST `/reader3/deleteBookmark`

- 鉴权：登录态
- Body：`Bookmark`
- 返回：空字符串

### POST `/reader3/deleteBookmarks`

- 鉴权：登录态
- Body：`Bookmark[]`
- 返回：空字符串

---

## 3.9 替换规则

### GET `/reader3/getReplaceRules`

- 鉴权：登录态
- 返回：`ReplaceRule[]`

### POST `/reader3/saveReplaceRule`

- 鉴权：登录态
- Body：`ReplaceRule`
- 要求：
  - `name` 非空
  - `pattern` 非空
- 返回：空字符串

### POST `/reader3/saveReplaceRules`

- 鉴权：登录态
- Body：`ReplaceRule[]`
- 返回：空字符串

### POST `/reader3/deleteReplaceRule`

- 鉴权：登录态
- Body：`ReplaceRule`
- 实际按 `name` 删除
- 返回：空字符串

### POST `/reader3/deleteReplaceRules`

- 鉴权：登录态
- Body：`ReplaceRule[]`
- 返回：空字符串

---

## 3.10 文件上传

### POST `/reader3/uploadFile`

- 鉴权：登录态
- Query:
  - `type?: string`，默认 `images`
- Content-Type：`multipart/form-data`
- 返回：字符串数组，每项是可访问的静态 URL

```json
[
  "/assets/alice01/images/cover.png"
]
```

### POST `/reader3/deleteFile`

- 鉴权：登录态
- Body:

```json
{ "url": "/assets/alice01/images/cover.png" }
```

- 返回：空字符串

---

## 3.11 WebDAV 辅助接口

### GET `/reader3/getWebdavFileList`

- 鉴权：已登录且开启 WebDAV
- Query:
  - `path?: string`，默认 `/`
- 返回：

```json
[
  {
    "name": "books",
    "size": 0,
    "path": "/books",
    "lastModified": 1710000000000,
    "isDirectory": true
  }
]
```

### GET `/reader3/getWebdavFile`

- 鉴权：已登录且开启 WebDAV
- Query:
  - `path: string`
- 返回：文件二进制流

### POST `/reader3/uploadFileToWebdav`

- 鉴权：已登录且开启 WebDAV
- Content-Type：`multipart/form-data`
- 表单：
  - `path`: 目标目录，可选，默认 `/`
  - 其余 file field: 上传文件
- 返回：上传后的文件信息数组

### POST `/reader3/deleteWebdavFile`

- 鉴权：已登录且开启 WebDAV
- Body:

```json
{ "path": "/books/a.txt" }
```

- 返回：空字符串

### POST `/reader3/deleteWebdavFileList`

- 鉴权：已登录且开启 WebDAV
- Body:

```json
{ "path": ["/books/a.txt", "/books/b.txt"] }
```

- 返回：空字符串

---

## 3.12 WebDAV 原生协议入口

### ANY `/reader3/webdav/*path`

- 鉴权：`Basic Auth`
- 支持方法：
  - `OPTIONS`
  - `PROPFIND`
  - `MKCOL`
  - `PUT`
  - `GET`
  - `DELETE`
  - `MOVE`
  - `COPY`
  - `LOCK`
  - `UNLOCK`

说明：

- 这里是给 WebDAV 客户端直接访问的原生协议接口，不建议普通前端页面直接调用
- 用户必须开启 `enableWebdav`
- `MOVE` / `COPY` 依赖 `Destination` 请求头

## 4. 前端重写建议

- 建议统一封装 `ApiResponse<T>`，并把 `isSuccess = false` 视为业务失败。
- 对 `NEED_LOGIN`、`NEED_SECURE_KEY` 做全局拦截。
- 对同时支持 `GET/POST` 的老接口，前端新实现建议统一发 `POST JSON`，兼容性最好。
- SSE 接口需要同时处理默认消息、`error` 事件、`end` 事件。
- `/reader3/readSourceFile` 与 `/reader3/getWebdavFile`、`/reader3/cover`、`/reader3/webdav/*` 不走统一返回包裹，单独处理。

