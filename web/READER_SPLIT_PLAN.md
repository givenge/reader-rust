# Reader.vue 拆分计划

## 当前状态分析

**Reader.vue 结构：**
- Template: 448 行（保持不变）
- Script: ~2,000 行（含大量已迁移但保留存根的方法）
- Style: ~830 行（保持不变）

**已创建的 Features 模块：**

| 模块 | 行数 | 核心功能 | 状态 |
|------|------|----------|------|
| `readerSession` | 120 | 会话初始化、换书、目录加载 | ✅ 已完成 |
| `readerContent` | 297 | 内容获取、过滤、滚动章节管理 | ✅ 已完成 |
| `readerProgress` | 150 | 阅读位置保存、恢复 | ✅ 已完成 |
| `readerPagination` | 194 | 翻页逻辑、动画 | ✅ 已完成 |
| `readerSpeech` | 207 | TTS 朗读 | ✅ 已完成 |
| `readerEnvironment` | 245 | 触摸、键盘、滚动、屏幕常亮 | ✅ 已完成 |
| `readerInteraction` | 708 | 点击处理、书签、搜索、过滤 | ⚠️ 过大，需拆分 |
| `readerAutoRead` | 189 | 自动翻页 | ✅ 已完成 |
| `readerPrefetch` | 72 | 章节缓存 | ✅ 已完成 |
| `readerBookmark` | 248 | 书签定位匹配 | ✅ 已完成 |

**主要问题：**
1. Reader.vue 中的存根方法（1054-1104 行）与 mixin 重复，需要清理
2. readerInteraction.js 过大（708行），需要进一步拆分
3. Reader.vue 需要引入新的拆分模块

---

## 拆分计划（UI界面保持不变）

### Phase 1: 清理 Reader.vue 存根方法

从 Reader.vue 移除以下已迁移到 mixin 的存根方法：

```javascript
// Session 模块
init(...args)
changeBook(...args)
changeBookSource(...args)
loadCatalog(...args)
getCatalog(...args)
refreshCatalog(...args)

// Content 模块
getBookContent(...args)
refreshContent(...args)
getContent(...args)
filterContent(...args)
loadShowChapter(...args)
addChapterContentToCache(...args)
computeShowChapterList(...args)

// Progress 模块
saveBookProgress(...args)
showPosition(...args)
saveReadingPosition(...args)
autoShowPosition(...args)

// Pagination 模块
computePages(...args)
nextPage(...args)
prevPage(...args)
showPage(...args)
transform(...args)
scrollContent(...args)

// Prefetch 模块
cacheChapterContent(...args)
cancelCaching(...args)
```

### Phase 2: 拆分 readerInteraction.js

将 `readerInteraction.js` (708行) 拆分为以下模块：

| 新模块 | 包含的方法 | 行数预估 |
|--------|-----------|---------|
| `readerClick.js` | handlerClick, eventHandler, epubClickHash, epubLocationChangeHandler, beforeReadMethodChange | ~150行 |
| `readerFilter.js` | checkSelection, showTextOperate, showTextFilterPrompt, formatChinese | ~120行 |
| `readerSelection.js` | getFilteredParagraphs, getCurrentParagraph, getPrevParagraph, getNextParagraph, showParagraph, getFirstParagraphPos, getParagraphListInView | ~150行 |
| `readerUI.js` | showReadingBookInfo, showSearchBookContentDialog, showCacheContent, toogleNight, showBookmarkDialog, exitRead, formatProgressTip, formatTime | ~120行 |

保留在 `readerInteraction.js` 的：
- showAddBookmark, showBookmark, showMatchKeyword, getContentMatchParagraph, showContentMatchParagraph（与 readerBookmark.js 合并）
- startAutoReading, autoRead, autoReadByPixel, stopAutoReading, toggleAutoReading（移至 readerAutoRead.js）
- cacheChapterContent, cancelCaching（移至 readerPrefetch.js）

### Phase 3: 更新模块索引

修改 `features/reader/runtime/index.js`：
1. 添加新模块导入
2. 移除重复的方法
3. 统一导出接口

### Phase 4: 更新 Reader.vue

修改 Reader.vue：
1. 移除所有存根方法
2. 确保 mixin 正确引入所有新模块
3. 保持 template 和 style 不变

---

## 目录结构

```
web/src/features/reader/
├── runtime/
│   ├── readerSession.js      # 会话管理
│   ├── readerContent.js      # 内容管理
│   ├── readerProgress.js     # 进度管理
│   ├── readerPagination.js   # 分页动画
│   ├── readerSpeech.js       # 语音朗读
│   ├── readerEnvironment.js  # 环境事件
│   ├── readerClick.js        # 点击处理 (新建)
│   ├── readerSelection.js    # 段落选择 (新建)
│   ├── readerFilter.js         # 文本过滤 (新建)
│   ├── readerUI.js             # UI操作 (新建)
│   ├── readerAutoRead.js     # 自动阅读
│   ├── readerPrefetch.js     # 预加载
│   ├── readerBookmark.js     # 书签
│   └── readerInteraction.js    # 剩余交互 (保留)
└── index.js
```

---

## 执行步骤

### 步骤 1: 创建新模块文件 (已完成 ✓)
- [x] readerClick.js
- [x] readerSelection.js
- [x] readerFilter.js
- [x] readerUI.js

### 步骤 2: 更新 index.js (已完成 ✓)
- [x] 添加新模块导入
- [x] 导出统一接口

### 步骤 3: 清理 Reader.vue (进行中)
- [x] Phase 1: Pagination 模块方法 (nextPage, prevPage, showPage, transform, scrollContent) ✓
- [x] Phase 2: Progress 模块方法 (saveBookProgress, showPosition, saveReadingPosition, autoShowPosition) ✓
- [x] Phase 3: Prefetch 模块方法 (cacheChapterContent, cancelCaching) ✓
- [x] Phase 4: Content 模块方法 (getBookContent, refreshContent, loadShowChapter, addChapterContentToCache, computeShowChapterList) ✓
- [x] 移除相关未使用的 import (Axios, Animate, setCache, getCache, networkFirstRequest, defaultReplaceRule, LimitResquest) ✓

---

## 注意事项

1. **UI界面保持不变**：template 和 style 部分不做任何修改
2. **保持兼容性**：确保所有方法在组件实例上可用
3. **循序渐进**：每次修改后验证功能正常
