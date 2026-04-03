# `Reader.vue` / `Index.vue` / `App.vue` 拆分方案

## 目标

为当前前端的大型页面建立一套可执行的拆分方案，解决以下问题：

- 页面文件过大，职责混杂
- 运行时逻辑、网络请求、状态同步、弹窗编排耦合严重
- 阅读器能力与业务页能力没有边界
- 后续 Vuex 模块化、SSE 迁移、PWA 治理缺少稳定落点

本方案覆盖：

- `web/src/views/Reader.vue`
- `web/src/views/Reader.vue`
- `web/src/App.vue`

---

## 当前结论

### `Reader.vue`

需要拆分，而且优先级最高。

原因：

- 文件体量最大，约 4000+ 行
- 同时处理阅读流程、目录、正文、分页、滚动、预缓存、语音朗读、键盘事件、唤醒锁、书源切换
- 是全项目最复杂、最脆弱的页面

### `Index.vue`

需要拆分，优先级仅次于 `Reader.vue`。

原因：

- 文件体量约 3700+ 行
- 同时承载书架、搜索、探索、书源导入、缓存管理、用户切换、WebDAV、本地存储、导航交互
- 既是首页又是管理台，边界混乱

### `App.vue`

需要拆分，但不是优先级最高。

原因：

- 体量约 1000+ 行，明显偏大
- 但它的复杂度主要来自“全局弹窗编排 + 登录流程 + 编辑器 + 主题入口”
- 比起 `Reader.vue` 和 `Index.vue`，它更像“全局壳层过胖”，不是“核心业务逻辑爆炸”

结论：

1. 先拆 `Reader.vue`
2. 再拆 `Index.vue`
3. 最后精简 `App.vue`

---

## 一、`Reader.vue` 详细拆分方案

## 1. 当前职责盘点

从现有结构看，`Reader.vue` 至少承担了以下 9 类职责：

1. 页面初始化与阅读入口
2. 目录加载、目录刷新、章节切换
3. 正文抓取、正文过滤、错误恢复
4. 阅读模式控制
5. 阅读进度保存与恢复
6. 预读 / 预缓存
7. 语音朗读（TTS）
8. 浏览器事件与设备能力接入
9. 阅读器浮层与交互编排

### 1.1 当前关键状态

当前 data/computed/watch 可以大致归类为：

- 阅读内容状态
  - `title`
  - `content`
  - `error`
  - `book`
  - `show`
- 分页/渲染状态
  - `contentStyle`
  - `currentPage`
  - `totalPages`
  - `transformX`
  - `transforming`
  - `currentParagraph`
- 弹层/面板状态
  - `popCataVisible`
  - `readSettingsVisible`
  - `popBookSourceVisible`
  - `popBookShelfVisible`
  - `showToolBar`
  - `showReadBar`
- 进度与滚动状态
  - `startSavePosition`
  - `scrollStartChapterIndex`
  - `showChapterList`
- 缓存与预读状态
  - `showCacheContentZone`
  - `isCachingContent`
  - `cachingContentTip`
  - `preCaching`
- 语音朗读状态
  - `speechAvalable`
  - `voiceList`
  - `speechSpeaking`
  - `showSpeechConfig`
  - `speechMinutes`
  - `speechEndTime`
  - `isSpeechTransitioning`
  - `autoReadingParagraphIndex`
  - `autoReadingProcessing`
- 浏览器/环境联动
  - `windowSize`
  - `keydownHandler`
  - `scrollHandler`
  - `wakeLock`
  - `speechSynthesis`

### 1.2 当前关键方法簇

从方法命名和 watch 行为判断，主要方法簇包括：

- 初始化与切书
  - `init`
  - `changeBook`
  - `changeBookSource`
- 目录与章节
  - `loadCatalog`
  - `getCatalog`
  - `refreshCatalog`
  - `toNextChapter`
  - `toLastChapter`
- 正文与过滤
  - `getBookContent`
  - `getContent`
  - `filterContent`
- 分页与显示
  - `computePages`
  - `showPage`
  - `showParagraph`
  - `computeShowChapterList`
- 进度管理
  - `saveReadingPosition`
  - `saveBookProgress`
  - `autoShowPosition`
- 语音与自动阅读
  - `fetchVoiceList`
  - `speech...`
  - `autoReading...`
- 浏览器交互
  - `keydownHandler`
  - `scrollHandler`
  - `wakeLock`

---

## 2. 拆分目标结构

建议将 `Reader.vue` 拆为“页面容器 + 5 个子容器 + 4 个运行时模块”。

## 2.1 页面层

### 保留文件

- `web/src/views/Reader.vue`

### 保留职责

- 路由级容器
- 子模块装配
- 顶层事件汇总
- 少量页面级弹层开关

### 移除职责

- 不再直接处理抓取/目录/预读/TTS/进度算法
- 不再直接管理浏览器事件副作用

## 2.2 推荐目录

```text
web/src/features/reader/
  api/
    readerService.js
  runtime/
    readerSession.js
    readerProgress.js
    readerPagination.js
    readerPrefetch.js
    readerSpeech.js
    readerEnvironment.js
  containers/
    ReaderShell.vue
    ReaderToolbar.vue
    ReaderBody.vue
    ReaderPanels.vue
    ReaderStatusBar.vue
  panels/
    CatalogPanel.vue
    BookSourcePanel.vue
    BookShelfPanel.vue
    ReaderSettingsPanel.vue
  content/
    ContentRenderer.vue
    ScrollReader.vue
    SlideReader.vue
  utils/
    contentFilter.js
    bookmarkMatcher.js
```

---

## 3. 模块职责划分

## 3.1 `ReaderShell.vue`

职责：

- 作为 `Reader.vue` 的主容器
- 连接 store 的 `readingBook`、`config`、`windowSize`
- 组织工具栏、正文区域、状态栏、各类面板

输入：

- 当前阅读书籍
- 阅读配置
- UI 可见性状态

输出：

- 页面级事件
- 对 runtime 的方法调用

## 3.2 `readerSession.js`

职责：

- 管理当前阅读会话
- 统一处理 `init` / `changeBook` / `changeBookSource`
- 协调目录加载与正文加载顺序

边界：

- 不直接触达 DOM
- 不直接处理分页和滚动显示

## 3.3 `readerProgress.js`

职责：

- 保存阅读位置
- 恢复阅读位置
- 控制 `saveBookProgress` / `saveReadingPosition`
- 处理章节索引、段落索引、页面位置映射

原因：

- 当前进度逻辑散落在 watch、事件、章节切换中，最容易回归

## 3.4 `readerPagination.js`

职责：

- `computePages`
- `showPage`
- `showParagraph`
- 页码计算和横向翻页展示

原因：

- 这是阅读器核心算法，不应继续混在页面文件里

## 3.5 `readerPrefetch.js`

职责：

- 预读下一章
- 缓存邻近章节
- scroll 模式下的章节窗口计算

原因：

- 预读逻辑与显示逻辑混在一起，容易引入竞态

## 3.6 `readerSpeech.js`

职责：

- 管理 TTS 列表获取
- 管理朗读开始/停止/切换
- 管理自动翻页与朗读之间的竞态保护

原因：

- 当前 `speechSynthesis` 相关状态高度集中，适合独立模块

## 3.7 `readerEnvironment.js`

职责：

- `keydown` 绑定/解绑
- `scroll` 绑定/解绑
- `unload` 时机处理
- `WakeLock` 管理
- `Lazyload` 事件绑定

原因：

- 这些都是浏览器运行时副作用，不应混入业务逻辑

## 3.8 面板组件

面板类组件只负责 UI 与交互，不负责业务编排：

- `CatalogPanel.vue`
- `BookSourcePanel.vue`
- `BookShelfPanel.vue`
- `ReaderSettingsPanel.vue`

当前已有组件可以保留，但要把状态和事件统一从 `Reader.vue` 抽走。

---

## 4. `Reader.vue` 迁移顺序

### 第一步

先抽运行时模块，不动 UI 结构：

- `readerSession.js`
- `readerProgress.js`
- `readerPagination.js`

### 第二步

再拆环境能力：

- `readerSpeech.js`
- `readerEnvironment.js`
- `readerPrefetch.js`

### 第三步

最后拆 UI 容器：

- `ReaderShell.vue`
- `ReaderToolbar.vue`
- `ReaderBody.vue`
- `ReaderPanels.vue`

---

## 5. `Reader.vue` 拆分验收标准

- `Reader.vue` 控制在 500-700 行以内
- 不直接出现 `speechSynthesis`、`WakeLock`、`window.addEventListener`
- 分页、进度、预读逻辑不再直接写在页面 methods 中
- `Content.vue` 与阅读 runtime 之间通过明确接口交互

---

## 二、`Index.vue` 详细拆分方案

## 1. 当前职责盘点

`Index.vue` 当前同时承担：

1. 书架首页
2. 搜索页
3. 探索页
4. 书源导入与管理
5. 失效书源检查
6. 本地存储入口
7. WebDAV 管理入口
8. 用户切换与管理模式
9. 导航栏折叠与手势交互

这已经不是一个“页面”，而是“前台首页 + 管理页 + 工具页”的混合体。

## 1.1 当前关键状态

可以分成以下几个状态域：

- 搜索域
  - `search`
  - `searchTypeList`
  - `isSearchResult`
  - `searchResult`
  - `searchPage`
  - `searchLastIndex`
  - `loadingMore`
- 探索域
  - `isExploreResult`
  - `popExploreVisible`
- 书源导入/管理域
  - `importSourceList`
  - `showImportSourceDialog`
  - `checkedSourceIndex`
  - `showBookSourceManageDialog`
  - `manageSourceSelection`
  - `isShowFailureBookSource`
  - `checkBookSourceTip`
  - `isCheckingBookSource`
- 书架与显示域
  - `showSourceGroup`
  - `bookSourcePagination`
  - `popIntroVisible`
- 导航与交互域
  - `showNavigation`
  - `navigationClass`
  - `navigationStyle`
  - `lastScrollTop`
- 连接与系统状态
  - `connecting`
  - `localStorageAvaliable`
- 导入书籍域
  - `importBookInfo`
  - `importBookGroup`
  - `importBookChapters`
  - `showImportBookDialog`
  - `importUsedTxtRule`
- 工具入口域
  - `showLocalStoreManageDialog`
  - `showWebDAVManageDialog`
  - `showAddUser`

## 1.2 当前关键计算属性

`Index.vue` 已经隐含了多个子领域：

- 书架展示
  - `bookList`
  - `shelfBooks`
  - `showShelfBooks`
  - `bookCoverList`
- 搜索结果去重
  - `searchResultMap`
- 连接状态
  - `connectStatus`
  - `connectType`
- 用户与命名空间
  - `readingRecent`
  - `loginAuth`
  - `userNS`
  - `userList`
- 书源视图
  - `bookSourceList`
  - `bookSourceShowList`

这说明它已经具备拆成多个容器的条件。

---

## 2. 拆分目标结构

建议把 `Index.vue` 拆成“页面壳层 + 6 个业务容器”。

```text
web/src/features/index/
  api/
    bookshelfService.js
    searchService.js
    sourceService.js
  containers/
    IndexShell.vue
    ShelfContainer.vue
    SearchContainer.vue
    ExploreContainer.vue
    SourceManageContainer.vue
    ToolContainer.vue
    UserNamespaceContainer.vue
  navigation/
    MainNavigation.vue
    SearchBar.vue
    StatusBanner.vue
  dialogs/
    ImportSourceDialog.vue
    ImportBookDialog.vue
    FailureSourceDialog.vue
  runtime/
    indexNavigation.js
    indexImport.js
    indexSourceCheck.js
```

## 2.1 `IndexShell.vue`

职责：

- 作为首页路由容器
- 只负责拼装导航、书架、搜索、探索和工具模块

## 2.2 `ShelfContainer.vue`

职责：

- 加载书架
- 管理分组筛选
- 管理书架卡片展示
- 对接 `BookManage`、`BookInfo` 等弹窗入口

## 2.3 `SearchContainer.vue`

职责：

- 处理单源/多源搜索
- 管理搜索结果列表
- 管理搜索配置与搜索分页
- 承接 `searchBookMultiSSE` / `searchBook`

说明：

- 这部分已经和书架逻辑足够独立，应该优先拆

## 2.4 `ExploreContainer.vue`

职责：

- 处理 `Explore` 相关数据
- 管理探索入口和探索结果显示

## 2.5 `SourceManageContainer.vue`

职责：

- 书源导入
- 书源列表管理
- 批量操作
- 失效书源检查

## 2.6 `ToolContainer.vue`

职责：

- 本地存储管理
- WebDAV 管理
- RSS 入口
- 其他工具入口

## 2.7 `UserNamespaceContainer.vue`

职责：

- 管理员模式下的用户切换
- `userNS` 状态同步
- 管理员视角的用户列表和命名空间控制

---

## 3. `Index.vue` 迁移顺序

### 第一步

先把搜索域独立：

- `SearchContainer.vue`
- `searchService.js`

原因：

- 当前已完成搜索 SSE 认证迁移，正好可以借这个机会把搜索链路从页面里抽离

### 第二步

再拆书架域：

- `ShelfContainer.vue`
- `bookshelfService.js`

### 第三步

再拆书源管理和导入域：

- `SourceManageContainer.vue`
- `indexImport.js`
- `indexSourceCheck.js`

### 第四步

最后拆导航与工具域：

- `MainNavigation.vue`
- `ToolContainer.vue`
- `UserNamespaceContainer.vue`

---

## 4. `Index.vue` 拆分验收标准

- `Index.vue` 控制在 500-700 行以内
- 搜索、书架、书源管理不再共享一套 methods
- 页面不再直接保存大批导入/管理类临时状态
- `userNS`、`searchConfig`、导航状态具备明确归属

---

## 三、`App.vue` 评估与拆分方案

## 1. 是否需要拆分

需要，但不应优先于 `Reader.vue` 和 `Index.vue`。

## 2. 当前职责盘点

`App.vue` 当前承担：

1. 路由壳层
2. 登录/注册弹窗
3. 全局 JSON 编辑器
4. 全局弹窗挂载点
5. 主题与页面类型 class 控制
6. 启动初始化
7. 全局数据加载入口
8. 事件总线监听
9. 原型链污染代码

### 2.1 当前明显问题

- 全局弹窗挂载过多
- 登录流程与全局初始化耦合
- `Date.prototype.format` 和 `String.prototype.getBytesLength` 仍在这里
- `loadBookshelf` / `loadBookSource` / `loadReplaceRules` / `loadRssSources` / `loadBookGroup` 实际上是多个领域初始化逻辑

## 3. 是否应彻底拆成多个页面

不需要。

`App.vue` 更适合保留为应用壳层，只做“全局装配”，而不是拆成多个路由页面。

因此它应该被**精简**，而不是“解散”。

## 4. 推荐目标结构

```text
web/src/app/
  AppShell.vue
  AppBootstrap.js
  AppThemeRuntime.js
  AppDialogHost.vue
  dialogs/
    LoginDialog.vue
    JsonEditorDialog.vue
    GlobalDialogHost.vue
```

## 4.1 `AppShell.vue`

职责：

- 仅保留 `<router-view>`
- 接入全局对话框宿主
- 接入主题 class 和页面类型 class

## 4.2 `AppBootstrap.js`

职责：

- 应用初始化
- 启动时加载用户信息、书架、书源、RSS、分组、替换规则
- 处理首次连接逻辑

## 4.3 `AppThemeRuntime.js`

职责：

- `autoSetTheme`
- `setTheme`
- `setMiniInterfaceClass`
- `setPageTypeClass`
- 安全区与视口变量设置

## 4.4 `AppDialogHost.vue`

职责：

- 统一挂载各类全局弹窗
- 管理弹窗开关和回调

### 建议拆分出的第一批弹窗

- `LoginDialog.vue`
- `JsonEditorDialog.vue`
- `GlobalDialogHost.vue`

## 4.5 原型链污染迁移

必须从 `App.vue` 移除：

- `Date.prototype.format`
- `String.prototype.getBytesLength`

迁移到：

- `web/src/utils/date.js`
- `web/src/utils/string.js`

---

## 5. `App.vue` 迁移顺序

### 第一步

先抽出工具函数：

- 日期格式化
- 字节长度计算

### 第二步

再抽初始化：

- `AppBootstrap.js`

### 第三步

最后抽全局弹窗宿主：

- `LoginDialog.vue`
- `JsonEditorDialog.vue`
- `AppDialogHost.vue`

---

## 四、推荐整体落地顺序

## 第一阶段

- 拆 `Reader.vue` 运行时模块
- 拆 `Index.vue` 搜索容器
- 移除 `App.vue` 原型链污染

## 第二阶段

- 拆 `Reader.vue` 面板与正文容器
- 拆 `Index.vue` 书架容器
- 抽 `AppBootstrap`

## 第三阶段

- 拆 `Index.vue` 书源管理与工具容器
- 抽 `AppDialogHost`
- 收口全局状态边界

---

## 五、实施前置要求

- 先冻结接口命名，避免拆分时同时改接口契约
- 先保证 SSE 客户端方案稳定
- 拆分期间不并行推进 Vue 3 迁移
- 每拆一个容器都要补最小回归验证

---

## 六、验收指标

### 页面体积

- `Reader.vue` < 700 行
- `Index.vue` < 700 行
- `App.vue` < 400 行

### 运行时边界

- 页面文件不直接持有大量浏览器副作用
- 页面文件不直接管理复杂缓存/朗读/预读算法
- 全局弹窗不再散落在 `App.vue` methods 中

### 可维护性

- 页面具备明确的状态域边界
- 网络层、运行时、UI 容器各自独立
- 后续 Vuex 模块化和 Vue 3 迁移有清晰落点

---

## 七、直接结论

### `Reader.vue`

必须拆，优先级最高。先拆运行时，再拆 UI。

### `Index.vue`

必须拆，优先级第二。先拆搜索和书架，再拆书源管理与工具。

### `App.vue`

需要精简，不需要重构成多个页面。重点是：

- 抽启动流程
- 抽主题运行时
- 抽全局弹窗宿主
- 移除原型链污染
