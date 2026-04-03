/**
 * 预读取和缓存管理模块
 * 负责章节预加载、缓存管理等功能
 * 注意：loadShowChapter, addChapterContentToCache, computeShowChapterList
 * 已移至 readerContent.js，避免重复
 */

import { LimitResquest } from "../../../plugins/helper";

export default {
  /**
   * 缓存指定数量的章节内容
   * @param {number|boolean} cacheCount - 缓存章节数量，true表示缓存所有剩余章节
   */
  cacheChapterContent(cacheCount) {
    let cacheChapterList = [];
    if (cacheCount === true) {
      cacheChapterList = cacheChapterList.concat(
        this.catalog.slice(this.chapterIndex + 1, this.catalog.length)
      );
    } else {
      cacheChapterList = cacheChapterList.concat(
        this.catalog.slice(
          this.chapterIndex + 1,
          Math.min(this.catalog.length, this.chapterIndex + 1 + cacheCount)
        )
      );
    }
    if (!cacheChapterList.length) {
      this.$message.error("不需要缓存");
      return;
    }
    this.isCachingContent = true;
    this.cachingContentTip = "正在缓存章节 0/" + cacheChapterList.length;
    this.cachingHandler = LimitResquest(2, handler => {
      this.cachingContentTip =
        "正在缓存章节 " +
        handler.requestCount +
        "/" +
        cacheChapterList.length;
      if (handler.isEnd()) {
        this.$message.success("缓存完成");
        this.isCachingContent = false;
        this.cachingContentTip = "";
      }
    });
    cacheChapterList.forEach(v => {
      this.cachingHandler(() => {
        return this.getBookContent(
          v.index,
          {
            timeout: 30000,
            silent: true
          },
          false,
          true
        );
      });
    });
  },

  /**
   * 取消正在进行的缓存
   */
  cancelCaching() {
    if (this.cachingHandler && this.cachingHandler.cancel) {
      this.cachingHandler.cancel();
      this.isCachingContent = false;
      this.cachingContentTip = "";
    }
  }
};