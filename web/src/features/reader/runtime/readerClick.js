/**
 * 点击交互模块
 * 负责点击事件处理、EPUB点击、阅读方式切换等
 */

export default {
  methods: {
    handlerClick(e) {
      if (this.isEpub) {
        return;
      }
      if (!this.lastTouch && !this.ignoreNextClick) {
        this.eventHandler(e);
      }
      this.ignoreNextClick = false;
    },

    epubClickHash(rect) {
      if (typeof rect.top !== "undefined") {
        this.scrollContent(
          rect.top -
          (this.$store.state.miniInterface
            ? this.getFirstParagraphPos().bottom
            : 0) -
          (window.webAppDistance | 0) -
          (this.$store.state.safeArea.top | 0),
          0,
          true
        );
      }
    },

    epubLocationChangeHandler(url) {
      function getPathname(path) {
        const a = document.createElement("a");
        a.href = path;
        return decodeURIComponent(a.pathname);
      }
      url = getPathname(url);
      const currentChapter = this.catalog[this.chapterIndex];
      if (currentChapter) {
        const chapterPrefix = this.content.replace(currentChapter.url, "");
        const iframeUrlPath = url.replace(chapterPrefix, "");
        let newChapterIndex = -1;
        for (let i = 0; i < this.catalog.length; i++) {
          if (this.catalog[i].url === iframeUrlPath) {
            newChapterIndex = i;
            break;
          }
        }
        if (newChapterIndex >= 0) {
          let book = { ...this.$store.getters.readingBook };
          book.index = newChapterIndex;
          this.$store.commit("setReadingBook", book);
          this.title = this.$store.getters.readingBook.catalog[newChapterIndex].title;
        }
      }
    },

    eventHandler(point) {
      if (this.checkSelection(true)) {
        this.ignoreNextClick = true;
        return;
      }
      if (
        this.popBookSourceVisible ||
        this.popBookShelfVisible ||
        this.popCataVisible ||
        this.readSettingsVisible
      ) {
        if (this.isEpub) {
          this.popBookSourceVisible = false;
          this.popBookShelfVisible = false;
          this.popCataVisible = false;
          this.readSettingsVisible = false;
        }
        return;
      }
      if (this.isAudio) {
        if (!this.showReadBar) {
          this.showToolBar = !this.showToolBar;
        }
        return;
      }
      if (this.autoReading) {
        this.showToolBar = !this.showToolBar;
        return;
      }
      const midX = this.windowSize.width / 2;
      const midY = this.windowSize.height / 2;
      if (this.isEpub) {
        point.clientY =
          point.clientY +
          45 -
          (document.documentElement.scrollTop || document.body.scrollTop);
      }
      if (
        Math.abs(point.clientY - midY) <= this.windowSize.height * 0.2 &&
        Math.abs(point.clientX - midX) <= this.windowSize.width * 0.2
      ) {
        if (!this.showReadBar) {
          this.showToolBar = !this.showToolBar;
        }
      } else if (this.$store.getters.config.clickMethod === "下一页") {
        this.showToolBar = false;
        this.nextPage();
      } else if (this.$store.getters.config.clickMethod === "不翻页") {
        this.showToolBar = !this.showToolBar;
      } else if (this.isSlideRead) {
        if (point.clientX > midX) {
          this.showToolBar = false;
          this.nextPage();
        } else if (point.clientX < midX) {
          this.showToolBar = false;
          this.prevPage();
        }
      } else if (point.clientY > midY) {
        this.showToolBar = false;
        this.nextPage();
      } else if (point.clientY < midY) {
        this.showToolBar = false;
        this.prevPage();
      }
    },

    beforeReadMethodChange() {
      this.currentParagraph = this.getCurrentParagraph();
    }
  }
};
