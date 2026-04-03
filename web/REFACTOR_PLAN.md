# 前端代码重构计划

## 概述

当前前端不是一个普通的 Vue 2 CRUD 项目，而是一个带有以下特征的阅读器前端：

- 与 Rust 后端强耦合，且要兼容 Java 版接口约定
- 大量使用 SSE、`window.open`、Service Worker、离线缓存
- 核心页面超大：`Reader.vue`、`Index.vue`
- 运行时依赖较多浏览器能力：TTS、Wake Lock、滚动恢复、键盘事件

因此，重构不能只做通用治理，还必须覆盖：

- 接口契约与兼容性
- SSE/认证迁移
- Service Worker/PWA 行为
- 阅读器特有能力拆分
- 缓存分层与失效策略

本计划按四条主线推进：

1. 认证与接口契约
2. 页面与状态结构重构
3. 缓存、PWA 与性能优化
4. 工程化与测试体系

---

## 基线现状

### 代码结构现状

- `src/views/Reader.vue`：超大阅读页，约 4000+ 行
- `src/views/Index.vue`：书架/搜索主页，约 3700+ 行
- `src/App.vue`：入口壳层，约 1000+ 行
- `src/plugins/vuex.js`：单文件大 store
- `src/plugins/axios.js`：请求、认证、错误处理耦合在一起

### 当前高风险点

- `accessToken` / `secureKey` 仍通过 URL 参数传递
- SSE 直接使用 `EventSource(url)`，与请求头认证方案冲突
- Service Worker 可能干扰 API/SSE 请求
- 大量 `window.*` 全局变量和全局组件引用
- 缓存机制分散在 `localStorage`、`localforage`、Service Worker 之间
- 前后端接口兼容性没有自动校验

### 重构原则

1. 小步迁移，不一次性推翻
2. 接口契约优先，先保使用，再谈美化
3. 页面拆分优先于样式优化
4. 先治理运行时风险，再做 Vue 3 迁移
5. 所有关键改动都要有回归验证

---

## 阶段一：认证与接口契约治理（1-2 周）

### 目标

先解决当前最容易出生产问题的部分：认证方式、SSE、接口兼容、Service Worker 对请求链路的干扰。

### 任务列表

#### 1.1 依赖升级与安全治理（优先级：高）

- [ ] 升级 `axios` 到 1.x
- [ ] 升级 Vue CLI 相关依赖到当前 Vue 2 可接受的稳定版本
- [ ] 修复 `npm audit` 中的高危漏洞
- [ ] 记录升级后兼容性风险点

**验收标准**：`npm audit` 无高危漏洞，构建与核心功能可用

#### 1.2 认证链路改造（优先级：高）

- [x] 将普通 API 的 `accessToken` 从 URL 参数迁移到请求头
- [x] 将 `secureKey` 从 URL 参数迁移到请求头
- [x] 统一封装认证注入逻辑到请求层
- [x] 设计并实现“非 axios 请求”的认证方案

**必须补充的设计项**

- [x] 明确 SSE 的认证方案
- [ ] 明确 `window.open` 下载/预览链接的认证方案
- [ ] 明确 WebDAV 相关访问链路的认证方案

**建议实现路径**

方案 A：短期兼容
- 普通请求用 header
- SSE / `window.open` 暂保留 URL token，但改为短时票据或最小暴露

方案 B：中期收敛
- SSE 改为 `fetch + ReadableStream`
- 下载改为先获取短时授权链接

**验收标准**：敏感信息不再长期暴露在 URL；SSE/下载/管理功能不受影响

#### 1.3 接口契约清单与兼容校验（优先级：高）

- [ ] 建立前端依赖接口清单
- [ ] 列出每个接口的请求方式、参数、返回结构、错误码语义
- [ ] 单独定义 SSE 事件格式约定
- [ ] 标记 Java 版兼容字段与 Rust 扩展字段

**首批必须覆盖接口**

- [ ] `login`
- [ ] `getBookshelf`
- [ ] `saveBook`
- [ ] `setBookSource`
- [ ] `getBookInfo`
- [ ] `getChapterList`
- [ ] `getBookContent`
- [x] `searchBookMultiSSE`
- [x] `searchBookSourceSSE`
- [x] `cacheBookSSE`

**验收标准**：接口清单完整，关键接口有自动回归校验

#### 1.4 Service Worker 风险收敛（优先级：高）

- [x] 审查当前 Service Worker 对 API/SSE 的影响
- [ ] 开发环境默认禁用 Service Worker
- [ ] 生产环境明确绕过 `/reader3/*` API 请求缓存
- [ ] SSE 请求明确不走离线缓存
- [ ] 设计版本升级和缓存清理策略

**验收标准**：不再出现 Service Worker 干扰 API/SSE 的问题

---

## 阶段二：页面与状态结构重构（2-4 周）

### 目标

拆掉超大页面和单文件状态管理，降低耦合，建立可维护的结构。

### 任务列表

#### 2.1 页面拆分优先级重排（优先级：高）

原计划只强调 `App.vue`，这不准确。优先级应改为：

1. `Reader.vue`
2. `Index.vue`
3. `App.vue`
4. `plugins/vuex.js`

#### 2.2 重构 `Reader.vue`（优先级：高）

- [ ] 拆分阅读内容渲染
- [ ] 拆分目录与章节跳转
- [ ] 拆分阅读进度保存
- [ ] 拆分主题与样式配置
- [ ] 拆分 TTS 逻辑
- [ ] 拆分 Wake Lock 与屏幕常亮逻辑
- [ ] 拆分键盘/滚动/触控交互逻辑

**建议目录**

- `src/features/reader/runtime/`
- `src/features/reader/content/`
- `src/features/reader/catalog/`
- `src/features/reader/progress/`
- `src/features/reader/theme/`
- `src/features/reader/tts/`
- `src/features/reader/interaction/`

**验收标准**：`Reader.vue` 只保留页面编排与状态接线

#### 2.3 重构 `Index.vue`（优先级：高）

- [ ] 拆分书架模块
- [ ] 拆分搜索模块
- [ ] 拆分搜索结果模块
- [ ] 拆分探索页模块
- [ ] 拆分用户设置与管理入口
- [ ] 拆分缓存管理逻辑

**建议目录**

- `src/features/bookshelf/`
- `src/features/search/`
- `src/features/explore/`
- `src/features/user/`
- `src/features/cache/`

**验收标准**：`Index.vue` 不再承载完整业务逻辑

#### 2.4 精简 `App.vue`（优先级：中）

- [ ] 仅保留应用壳层、全局布局、主题入口、登录弹窗挂载
- [ ] 移除业务逻辑和具体页面状态

#### 2.5 Vuex 重构（优先级：高）

- [ ] 将 `plugins/vuex.js` 拆为模块
- [ ] 规范 state / mutation / action 边界
- [ ] 弱化组件直接改 store 的模式
- [ ] 为关键状态补 JSDoc

**建议模块**

- `modules/user.js`
- `modules/bookshelf.js`
- `modules/search.js`
- `modules/reader.js`
- `modules/config.js`
- `modules/ui.js`
- `modules/cache.js`

**验收标准**：状态归属清晰，页面不再依赖隐式共享状态

#### 2.6 全局变量清理（优先级：高）

不仅要处理 `window.shelfBooks`、`window.reader`，还要建立完整迁移表。

**需要审查并迁移的对象类别**

- [ ] 页面级组件实例引用
- [ ] 全局错误列表
- [ ] 缓存存储实例
- [ ] 自定义字体注册状态
- [ ] SSE 连接句柄
- [ ] 阅读器运行时句柄

**验收标准**：除浏览器原生能力适配外，不再依赖任意业务全局变量

#### 2.7 网络层拆分（优先级：中）

- [ ] 将 `plugins/axios.js` 中的认证、错误处理、弹窗、副作用拆开
- [ ] 建立 API Service 层
- [ ] 建立 SSE Client 层
- [ ] 建立下载/打开外链的统一封装

**建议目录**

- `src/api/http.js`
- `src/api/interceptors.js`
- `src/api/bookService.js`
- `src/api/userService.js`
- `src/api/bookSourceService.js`
- `src/api/rssService.js`
- `src/api/sseClient.js`

**验收标准**：组件中不再直接拼接复杂请求逻辑

---

## 阶段三：缓存、PWA 与性能优化（2-3 周）

### 目标

收敛缓存策略，优化阅读性能和首屏体验，同时保留离线能力。

### 任务列表

#### 3.1 建立缓存分层策略（优先级：高）

当前缓存来源较多，必须统一治理。

**需明确的缓存层**

- [ ] 配置缓存
- [ ] 书架数据缓存
- [ ] 搜索结果缓存
- [ ] 章节正文缓存
- [ ] 封面/图片缓存
- [ ] 静态资源离线缓存

**需明确的策略**

- [ ] 每类缓存的存储介质
- [ ] key 规则
- [ ] TTL / 失效策略
- [ ] 主动清理策略
- [ ] 用户切换时的隔离策略

**验收标准**：缓存行为可预测，清理逻辑不再分散

#### 3.2 PWA 策略重构（优先级：中）

- [ ] 明确哪些资源允许离线
- [ ] API 请求默认不缓存
- [ ] 阅读正文是否允许离线单独定义
- [ ] 明确更新提示与强制刷新机制

**验收标准**：PWA 仅增强体验，不干扰核心阅读功能

#### 3.3 阅读性能优化（优先级：高）

- [ ] 优化长章节渲染
- [ ] 优化章节切换体验
- [ ] 优化滚动监听与阅读位置保存频率
- [ ] 优化字体与主题切换性能
- [ ] 优化图片懒加载与失败兜底

**验收标准**：长章节加载、滚动、切换明显更稳定

#### 3.4 列表性能优化（优先级：中）

- [ ] 书架列表虚拟滚动
- [ ] 搜索结果长列表虚拟化
- [ ] 目录列表性能优化

#### 3.5 请求并发与取消（优先级：中）

- [ ] 为搜索请求添加取消机制
- [ ] 为换源搜索添加取消机制
- [ ] 为章节预缓存添加并发控制
- [ ] 为重复请求添加去重策略

**验收标准**：页面切换和重复操作不再遗留无效请求

---

## 阶段四：工程化、测试与文档（1-2 周）

### 目标

建立持续可维护的前端开发基线。

### 任务列表

#### 4.1 代码规范（优先级：高）

- [ ] 配置 ESLint
- [ ] 配置 Prettier
- [ ] 统一 import 顺序和文件命名规则
- [ ] 禁止直接修改内建原型

#### 4.2 测试体系（优先级：高）

不能只做普通组件测试，必须覆盖项目特有场景。

**测试分层**

- [ ] 工具函数测试
- [ ] Vuex module 测试
- [ ] API Service 测试
- [ ] SSE Client 测试
- [ ] 关键页面组件测试
- [ ] 接口契约测试

**必须覆盖的回归场景**

- [ ] 登录与登出
- [ ] 加载书架
- [ ] 搜索并加入书架
- [ ] 换源
- [ ] 获取目录与正文
- [ ] 阅读进度保存
- [ ] Service Worker 更新与缓存绕过

#### 4.3 文档与清单（优先级：中）

- [ ] 编写目录结构说明
- [ ] 编写网络层使用说明
- [ ] 编写缓存策略说明
- [ ] 编写 SSE 使用说明
- [ ] 编写前后端接口契约文档
- [ ] 建立重构检查清单

#### 4.4 CI / 构建治理（优先级：中）

- [ ] 固定 Node 版本
- [ ] 增加 `lint`、`build`、`test` 流水线
- [ ] 增加构建体积分析
- [ ] 增加 bundle 大小门禁

---

## 长期规划：Vue 3 + TypeScript + Vite（可选）

### 前提

只有在以下条件满足后，再启动迁移：

- [ ] Vue 2 项目结构已拆分完成
- [ ] 网络层与状态层边界清晰
- [ ] Service Worker 与缓存策略稳定
- [ ] 关键接口契约已固化

### 迁移建议

不要边治理边整体迁移。建议顺序：

1. 先完成 Vue 2 结构治理
2. 再把工具层、API 层、store 模块化
3. 然后逐模块迁移到 Vue 3
4. 最后切换到 Vite 与 TypeScript

### 迁移任务

- [ ] 评估 Element UI 替代方案
- [ ] 建立 Vue 3 兼容层
- [ ] 迁移组合式逻辑
- [ ] 为 API 与 store 增加类型定义
- [ ] 切换构建工具到 Vite

---

## 推荐实施顺序

### 第一优先级

- [ ] 认证迁移方案
- [ ] SSE 认证方案
- [ ] API 契约清单
- [ ] Service Worker 绕过 API/SSE
- [ ] `Reader.vue` / `Index.vue` 拆分设计

### 第二优先级

- [ ] Vuex 模块化
- [ ] 网络层拆分
- [ ] 全局变量迁移
- [ ] 缓存分层治理

### 第三优先级

- [ ] 性能优化
- [ ] PWA 重构
- [ ] 工程化与测试

### 第四优先级

- [ ] Vue 3 迁移准备

---

## 成功指标

### 技术指标

- [ ] `npm audit` 无高危漏洞
- [ ] `eslint` 错误数为 0
- [ ] 关键页面文件行数显著下降
- [ ] 全局业务变量趋近于 0
- [ ] 关键接口有契约测试覆盖

### 运行质量指标

- [ ] 不再出现 Service Worker 干扰 API/SSE
- [ ] 不再通过 URL 长期暴露敏感认证信息
- [ ] SSE 搜索、换源、缓存流程稳定
- [ ] 阅读进度、目录、正文加载无回归

### 用户体验指标

- [ ] 首屏加载时间稳定
- [ ] 阅读页滚动与翻章流畅
- [ ] 长章节渲染稳定
- [ ] 错误提示可理解且可恢复

---

## 附录：首批需建立的专题文档

- [ ] `docs/frontend-api-contract.md`
- [ ] `docs/frontend-auth-flow.md`
- [ ] `docs/frontend-sse-strategy.md`
- [ ] `docs/frontend-cache-strategy.md`
- [ ] `docs/frontend-pwa-strategy.md`
- [ ] `docs/frontend-reader-architecture.md`

---

*最后更新：2026-03-31*
*维护者：前端团队*
