# Reader 功能模块拆分

## 拆分状态

### 已完成 ✅

#### 运行时模块 (Runtime Modules)

所有运行时模块已创建完成，位于 `web/src/features/reader/runtime/` 目录下：

1. **readerSession.js** - 会话管理
   - `init()` - 初始化阅读器
   - `changeBook()` - 切换书籍
   - `changeBookSource()` - 切换书源
   - `loadCatalog()` - 加载目录
   - `getCatalog()` - 获取目录
   - `refreshCatalog()` - 刷新目录

2. **readerProgress.js** - 进度管理
   - `showPosition()` - 显示指定位置
   - `saveReadingPosition()` - 保存阅读位置
   - `autoShowPosition()` - 自动显示上次阅读位置
   - `saveBookProgress()` - 保存书籍进度

3. **readerPagination.js** - 分页管理
   - `computePages()` - 计算总页数
   - `nextPage()` - 下一页
   - `prevPage()` - 上一页
   - `showPage()` - 显示指定页面
   - `transform()` - 执行横向翻页动画
   - `scrollContent()` - 执行纵向滚动
   - `showParagraph()` - 显示指定段落
   - `getFirstParagraphPos()` - 获取第一个段落位置

4. **readerSpeech.js** - 语音朗读
   - `fetchVoiceList()` - 获取可用语音列表
   - `changeSpeechRate()` - 改变朗读语速
   - `changeSpeechPitch()` - 改变朗读音调
   - `changeSpeechMinutes()` - 改变定时朗读时长
   - `startSpeech()` - 开始朗读
   - `stopSpeech()` - 停止朗读
   - `restartSpeech()` - 重启朗读
   - `toggleSpeech()` - 切换朗读状态
   - `speechPrev()` - 朗读上一段
   - `speechNext()` - 朗读下一段

5. **readerEnvironment.js** - 环境管理
   - `handleTouchStart()` - 触摸开始处理
   - `handleTouchMove()` - 触摸移动处理
   - `handleTouchEnd()` - 触摸结束处理
   - `keydownHandler()` - 键盘事件处理
   - `scrollHandler()` - 滚动事件处理
   - `wakeLock()` - 屏幕唤醒锁
   - `lazyloadHandler()` - 延迟加载处理器

6. **readerPrefetch.js** - 预读取管理
   - `loadShowChapter()` - 加载并显示指定章节
   - `addChapterContentToCache()` - 添加章节内容到缓存
   - `computeShowChapterList()` - 计算要显示的章节列表
   - `cacheChapterContent()` - 缓存指定数量的章节内容
   - `cancelCaching()` - 取消正在进行的缓存

#### 工具函数 (Utils)

已创建的工具函数文件：

1. **web/src/utils/date.js** - 日期格式化
   - `formatDate()` - 格式化日期
   - `Date.prototype.format` - 向后兼容的原型扩展

2. **web/src/utils/string.js** - 字符串处理
   - `getBytesLength()` - 计算字符串字节长度
   - `String.prototype.getBytesLength` - 向后兼容的原型扩展

#### 原型链污染移除

已从 `App.vue` 移除以下原型链污染代码：
- `Date.prototype.format`
- `String.prototype.getBytesLength`

这些功能已迁移到独立的工具函数模块，并保留了向后兼容的原型扩展。

### 待进行 🚧

#### UI 容器拆分

根据 `VIEW_SPLIT_PLAN.md` 的规划，还需要拆分以下UI容器：

1. **ReaderShell.vue** - 阅读器主容器
2. **ReaderToolbar.vue** - 工具栏
3. **ReaderBody.vue** - 正文区域
4. **ReaderPanels.vue** - 面板区域（目录、书源、书架、设置）
5. **ReaderStatusBar.vue** - 状态栏

#### 面板组件拆分

现有面板组件需要从 `Reader.vue` 中抽离：

1. **CatalogPanel.vue** - 目录面板
2. **BookSourcePanel.vue** - 书源面板
3. **BookShelfPanel.vue** - 书架面板
4. **ReaderSettingsPanel.vue** - 阅读设置面板

## 使用方式

### 在 Reader.vue 中使用运行时模块

可以通过混合器（mixin）的方式在 Reader.vue 中使用这些模块：

```javascript
import readerRuntime from "../features/reader/runtime";

export default {
  mixins: [readerRuntime],
  // 组件的其他配置
};
```

### 独立使用模块

也可以只导入需要的模块：

```javascript
import readerSession from "../features/reader/runtime/readerSession";
import readerProgress from "../features/reader/runtime/readerProgress";

export default {
  mixins: [readerSession, readerProgress],
  // 组件的其他配置
};
```

## 模块间依赖

各运行时模块之间存在以下依赖关系：

- `readerEnvironment` 依赖 `readerPagination` 的 `scrollContent` 方法
- `readerPrefetch` 依赖其他模块的多个方法（需要进一步解耦）
- `readerProgress` 依赖 `readerPagination` 的 `showParagraph` 方法

## 下一步计划

1. **解耦模块间依赖** - 减少模块之间的直接调用，改为通过事件总线或状态管理进行通信
2. **拆分UI容器** - 将 `Reader.vue` 中的UI结构拆分为独立的容器组件
3. **完善API层** - 创建 `readerService.js` 统一管理所有API请求
4. **添加单元测试** - 为各个运行时模块编写单元测试
5. **性能优化** - 优化预加载和缓存策略
