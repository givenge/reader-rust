// 自动生成的 Reader.vue 方法映射
// 用于同步修改组件和 mixins

export const methodMap = {
  session: ['init', 'changeBook', 'changeBookSource', 'loadCatalog', 'getCatalog', 'refreshCatalog'],
  content: ['getBookContent', 'refreshContent', 'getContent', 'filterContent', 'loadShowChapter', 'addChapterContentToCache', 'computeShowChapterList'],
  prefetch: ['cacheChapterContent', 'cancelCaching'],
  progress: ['showPosition', 'saveReadingPosition', 'autoShowPosition', 'saveBookProgress'],
  pagination: ['computePages', 'nextPage', 'prevPage', 'showPage', 'transform', 'scrollContent'],
  speech: ['fetchVoiceList', 'changeSpeechRate', 'changeSpeechPitch', 'changeSpeechMinutes', 'startSpeech', 'stopSpeech', 'restartSpeech', 'toggleSpeech', 'speechPrev', 'speechNext'],
  environment: ['handleTouchStart', 'handleTouchMove', 'handleTouchEnd', 'keydownHandler', 'scrollHandler', 'wakeLock', 'lazyloadHandler'],
  interaction: ['handlerClick', 'epubClickHash', 'epubLocationChangeHandler', 'eventHandler', 'formatProgressTip', 'checkSelection', 'showTextOperate', 'showTextFilterPrompt', 'toTop', 'toBottom', 'toNextChapter', 'toLastChapter', 'toShelf'],
  paragraph: ['getFilteredParagraphs', 'getCurrentParagraph', 'getPrevParagraph', 'getNextParagraph', 'exitRead', 'showParagraph', 'getFirstParagraphPos', 'beforeReadMethodChange'],
  autoRead: ['startAutoReading', 'autoRead', 'autoReadByPixel', 'stopAutoReading', 'toggleAutoReading'],
  bookmark: ['showBookmarkDialog', 'showBookmark', 'showMatchKeyword', 'showContentMatchParagraph', 'getContentMatchParagraph', 'getParagraphListInView', 'showAddBookmark'],
  readingBook: ['showReadingBookInfo', 'toogleNight', 'formatChinese', 'showSearchBookContentDialog'],
  cache: ['showCacheContent']
};

// 导出所有方法名
export const allMethods = Object.values(methodMap).flat();
