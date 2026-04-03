import { simplized, traditionalized } from "../../../plugins/chinese";
import { LimitResquest, editDistance } from "../../../plugins/helper";
import {
  defaultReplaceRule,
  defaultBookmark
} from "../../../plugins/config.js";
import eventBus from "../../../plugins/eventBus";

// eslint-disable-next-line no-useless-escape
const symboRegex = /[\u2000-\u206F\u2E00-\u2E7F\\'!"#$%&\(\)*+,-\./:;<=>?@\[\]^_`{\|}~，。？《》；：、«]/g;

export default {
  data() {
    return {
      showCacheContentZone: false,
      isCachingContent: false,
      cachingContentTip: "",
      autoReading: false,
      autoReadingParagraphIndex: -1,
      autoReadingProcessing: false,
      autoReadingTimer: null,
      showTextFilterPrompting: false,
      showAddBookmarking: false,
      cachingHandler: null
    };
  },
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
    formatProgressTip(value) {
      return `第 ${value || this.progressValue}/${this.totalPages} 页`;
    },
    checkSelection(show) {
      let text = "";
      if (window.getSelection) {
        text = window.getSelection().toString();
      } else if (document.selection && document.selection.type !== "Control") {
        text = document.selection.createRange().text;
      }
      if (text && show) {
        setTimeout(() => {
          if (
            this.$store.getters.config.selectionAction === "过滤弹窗" ||
            this.$store.getters.config.selectionAction === "操作弹窗"
          ) {
            this.showTextOperate(text);
          }
        }, 200);
      }
      return text;
    },
    async showTextOperate(text) {
      const res = await this.$confirm("请选择操作?", "提示", {
        confirmButtonText: "添加过滤规则",
        cancelButtonText: "添加书签",
        type: "warning",
        closeOnClickModal: false,
        closeOnPressEscape: false,
        distinguishCancelAndClose: true
      }).catch(action => {
        return action === "close" ? "close" : false;
      });
      if (res === "close") {
        return;
      }
      if (res) {
        return this.showTextFilterPrompt(text);
      }
      return this.showAddBookmark(text);
    },
    async showTextFilterPrompt(text) {
      if (this.showTextFilterPrompting) {
        return;
      }
      if (!text.replace(/^\s+/, "").replace(/\s+$/, "")) {
        return;
      }
      const replaceRule = Object.assign({}, defaultReplaceRule, {
        name: "文本替换",
        pattern: text,
        replacement: "",
        isRegex: false,
        isEnabled: true,
        scope:
          this.$store.getters.readingBook.name +
          ";" +
          this.$store.getters.readingBook.bookUrl
      });
      this.showTextFilterPrompting = true;
      eventBus.$emit("showReplaceRuleForm", replaceRule, true, () => {
        this.showTextFilterPrompting = false;
      });
    },
    async showAddBookmark(text) {
      if (this.showAddBookmarking) {
        return;
      }
      const pureText = text.replace(/^\s+/, "").replace(/\s+$/, "");
      const paragraph = this.getContentMatchParagraph(pureText, 1, 0.7);
      if (!paragraph) {
        this.$message.error("选择1-2段整段文字才能定位段落");
        return;
      }
      const paragraphLength = 5;
      const paragraphTextLength = 150;
      const paragraphList = [paragraph];
      let bookText = paragraph.innerText + "\n";
      if (
        paragraphList.length < paragraphLength &&
        bookText.length < paragraphTextLength
      ) {
        let paragraphIndex = -1;
        const list = this.$refs.bookContentRef.$el.querySelectorAll("h3,p");
        for (let i = 0; i < list.length; i++) {
          if (paragraphIndex > 0 && i > paragraphIndex) {
            paragraphList.push(list[i]);
            bookText += list[i].innerText + "\n";
            if (
              paragraphList.length >= paragraphLength ||
              bookText.length >= paragraphTextLength
            ) {
              break;
            }
          } else if (paragraphList[paragraphList.length - 1] === list[i]) {
            paragraphIndex = i;
          }
        }
      }
      bookText = bookText.replace(/\n*$/, "");
      const bookmark = Object.assign({}, defaultBookmark, {
        bookName: this.$store.getters.readingBook.name,
        bookAuthor: this.$store.getters.readingBook.author,
        chapterIndex: this.chapterIndex,
        chapterPos: this.currentPage,
        chapterName: this.title,
        bookText,
        content: ""
      });
      this.showAddBookmarking = true;
      eventBus.$emit("showBookmarkForm", bookmark, true, () => {
        this.showAddBookmarking = false;
      });
    },
    getFilteredParagraphs() {
      const allElements = this.$refs.bookContentRef.$el.querySelectorAll(
        "h3,p"
      );
      const list = [];
      let lastText = "";
      allElements.forEach(el => {
        const text = el.textContent.trim();
        if (text && text !== lastText) {
          list.push(el);
          lastText = text;
        }
      });
      return list;
    },
    getCurrentParagraph() {
      const readingEle = this.$refs.bookContentRef.$el.querySelectorAll(".reading");
      let currentParagraph = null;
      if (!readingEle.length) {
        const list = this.getFilteredParagraphs();
        for (let i = 0; i < list.length; i++) {
          const elePos = list[i].getBoundingClientRect();
          if (this.isSlideRead) {
            if (elePos.right > 0) {
              currentParagraph = list[i];
              break;
            }
          } else if (
            elePos.bottom >
            30 + 20 + (window.webAppDistance | 0) + (this.$store.state.safeArea.top | 0)
          ) {
            currentParagraph = list[i];
            break;
          }
        }
      } else {
        currentParagraph = readingEle[0];
      }
      return currentParagraph;
    },
    getPrevParagraph() {
      const current = this.getCurrentParagraph();
      const list = this.getFilteredParagraphs();
      for (let i = 0; i < list.length; i++) {
        if (i > 0 && current === list[i]) {
          return list[i - 1];
        }
      }
      return null;
    },
    getNextParagraph() {
      const current = this.getCurrentParagraph();
      const list = this.getFilteredParagraphs();
      for (let i = 0; i < list.length; i++) {
        if (current === list[i]) {
          return list[i + 1];
        }
      }
      return null;
    },
    exitRead() {
      this.stopSpeech();
      const current = this.getCurrentParagraph();
      this.showReadBar = false;
      this.showParagraph(current);
    },
    showParagraph(paragraph, scroll) {
      if (!paragraph) {
        return;
      }
      if (this.isSlideRead) {
        this.$nextTick(() => {
          const pos = paragraph.getBoundingClientRect();
          if (pos.left > this.windowSize.width - 16) {
            this.showPage(
              Math.round(pos.left / (this.windowSize.width - 16)) + 1,
              0
            );
          }
        });
      } else if (scroll) {
        this.$nextTick(() => {
          const pos = paragraph.getBoundingClientRect();
          this.scrollContent(
            pos.top -
              (this.$store.state.miniInterface
                ? this.getFirstParagraphPos().bottom
                : 0) -
              (window.webAppDistance | 0) -
              (this.$store.state.safeArea.top | 0),
            0
          );
        });
      }
    },
    getFirstParagraphPos() {
      return this.$refs.top.getBoundingClientRect();
    },
    beforeReadMethodChange() {
      this.currentParagraph = this.getCurrentParagraph();
    },
    showCacheContent() {
      this.showCacheContentZone = !this.showCacheContentZone;
    },
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
      this.cachingContentTip = "正在缓存章节  0/" + cacheChapterList.length;
      this.cachingHandler = LimitResquest(2, handler => {
        this.cachingContentTip =
          "正在缓存章节  " + handler.requestCount + "/" + cacheChapterList.length;
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
    cancelCaching() {
      if (this.cachingHandler && this.cachingHandler.cancel) {
        this.cachingHandler.cancel();
        this.isCachingContent = false;
        this.cachingContentTip = "";
      }
    },
    startAutoReading() {
      this.showToolBar = false;
      this.autoReading = true;
      this.autoReadingParagraphIndex = -1;
      this.autoReadingProcessing = false;
      this.autoRead();
    },
    autoRead() {
      if (!this.autoReading || this.autoReadingProcessing) {
        return;
      }
      this.autoReadingProcessing = true;
      if (this.showToolBar) {
        this.autoReadingProcessing = false;
        this.autoReadingTimer = setTimeout(() => this.autoRead(), 300);
        return;
      }
      if (this.config.autoReadingMethod === "像素滚动") {
        this.autoReadingProcessing = false;
        this.autoReadByPixel();
        return;
      }
      const allElements = this.$refs.bookContentRef.$el.querySelectorAll("h3,p");
      const list = [];
      let lastText = "";
      allElements.forEach(el => {
        const text = el.textContent.trim();
        if (text && text !== lastText) {
          list.push(el);
          lastText = text;
        }
      });
      if (!list.length) {
        this.autoReadingProcessing = false;
        return;
      }
      if (this.autoReadingParagraphIndex < 0) {
        for (let i = 0; i < list.length; i++) {
          const elePos = list[i].getBoundingClientRect();
          if (this.isSlideRead ? elePos.right > 0 : elePos.bottom > 50) {
            this.autoReadingParagraphIndex = i;
            break;
          }
        }
        if (this.autoReadingParagraphIndex < 0) {
          this.autoReadingParagraphIndex = 0;
        }
      }
      if (this.autoReadingParagraphIndex >= list.length) {
        this.autoReadingParagraphIndex = -1;
        this.autoReadingProcessing = false;
        this.$once("showContent", () => {
          setTimeout(() => this.autoRead(), 300);
        });
        this.toNextChapter(() => {
          this.autoReading = false;
        });
        return;
      }
      const current = list[this.autoReadingParagraphIndex];
      list.forEach(el => el.classList.remove("reading"));
      current.classList.add("reading");
      this.showParagraph(current, true);
      let delayTime = this.config.autoReadingLineTime;
      try {
        const currentPos = current.getBoundingClientRect();
        delayTime =
          delayTime *
          Math.ceil(
            currentPos.height / this.config.fontSize / this.config.lineHeight
          );
      } catch (error) {
        //
      }
      this.autoReadingProcessing = false;
      this.autoReadingTimer = setTimeout(() => {
        this.autoReadingParagraphIndex++;
        this.autoRead();
      }, delayTime);
    },
    autoReadByPixel() {
      if (!this.autoReading) {
        return;
      }
      if (this.showToolBar) {
        this.autoReadingTimer = setTimeout(() => this.autoRead(), 300);
        return;
      }
      if (this.config.autoReadingMethod !== "像素滚动") {
        this.autoRead();
        return;
      }
      const scrollTop =
        document.documentElement.scrollTop || document.body.scrollTop;
      if (
        scrollTop + this.windowSize.height <
        document.documentElement.scrollHeight
      ) {
        this.autoReadingTimer = setTimeout(() => {
          this.scrollContent(this.config.autoReadingPixel, 0);
          this.autoReadByPixel();
        }, this.config.autoReadingLineTime);
      } else {
        this.$once("showContent", () => {
          setTimeout(() => this.autoReadByPixel(), 100);
        });
        this.toNextChapter(() => {
          this.autoReading = false;
        });
      }
    },
    stopAutoReading() {
      if (this.autoReadingTimer) {
        clearTimeout(this.autoReadingTimer);
        this.autoReadingTimer = null;
      }
      this.autoReading = false;
      this.autoReadingParagraphIndex = -1;
      this.autoReadingProcessing = false;
      const list = this.$refs.bookContentRef?.$el?.querySelectorAll("h3,p");
      if (list) {
        list.forEach(el => el.classList.remove("reading"));
      }
    },
    toggleAutoReading() {
      if (this.autoReading) {
        this.stopAutoReading();
      } else {
        this.startAutoReading();
      }
    },
    showReadingBookInfo() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookInfoDialog", book);
    },
    toogleNight() {
      this.$store.commit("setNightTheme", !this.isNight);
    },
    formatChinese(text) {
      if (this.isEpub || this.isAudio || this.isCbz || this.isCarToon) {
        return text;
      }
      return this.config.chineseFont === "简体"
        ? simplized(text)
        : traditionalized(text);
    },
    showSearchBookContentDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showSearchBookContentDialog", book);
    },
    showMatchKeyword(data) {
      if (this._inactive) {
        return;
      }
      if (!this.$refs.bookContentRef) {
        setTimeout(() => this.showMatchKeyword(data), 10);
        return;
      }
      try {
        const list = this.$refs.bookContentRef.$el.querySelectorAll(
          ".reading-chapter h3,p"
        );
        let matchCount = 0;
        for (let i = 0; i < list.length; i++) {
          const pContent = list[i].innerText;
          let startIndex = -1;
          let isFound = false;
          while (startIndex < pContent.length) {
            startIndex = pContent.indexOf(data.query, startIndex + 1);
            if (startIndex >= 0) {
              matchCount++;
              if (matchCount === data.resultCountWithinChapter + 1) {
                isFound = true;
                this.showParagraph(list[i], true);
                break;
              }
            } else {
              break;
            }
          }
          if (isFound) {
            break;
          }
        }
      } catch (error) {
        //
      }
    },
    getParagraphListInView() {
      const list = this.$refs.bookContentRef.$el.querySelectorAll("h3,p");
      const paragraphList = [];
      for (let i = 0; i < list.length; i++) {
        const elePos = list[i].getBoundingClientRect();
        if (this.isSlideRead) {
          if (elePos.right > 0 && elePos.left > 0) {
            paragraphList.push(list[i]);
          }
        } else if (
          elePos.bottom >
            30 + 20 + (window.webAppDistance | 0) + (this.$store.state.safeArea.top | 0) &&
          elePos.bottom < this.windowSize.height
        ) {
          paragraphList.push(list[i]);
        }
      }
      return paragraphList;
    },
    showBookmarkDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookmarkDialog", book);
    },
    getContentMatchParagraph(text, distance, minDistance) {
      distance = distance || 0.7;
      const paragraphList = text
        .replace(/\\n+/g, "\n")
        .split(/\n+/)
        .map(v => v.replace(symboRegex, ""))
        .filter(v => v);
      try {
        const list = this.$refs.bookContentRef.$el.querySelectorAll(
          ".reading-chapter h3,p"
        );
        let paragraph = null;
        for (let i = 0; i < list.length; i++) {
          let isMatch = true;
          let pos = 0;
          let startPos = i;
          for (let j = 0; j < paragraphList.length; j++) {
            let content = null;
            while (i + pos < list.length) {
              content = list[i + pos].innerText.replace(symboRegex, "");
              if (!content.length) {
                pos++;
                startPos++;
              } else {
                break;
              }
            }
            if (!content) {
              isMatch = false;
              break;
            }
            const paragraphDistance = editDistance(content, paragraphList[j]);
            if (paragraphDistance < distance) {
              isMatch = false;
              break;
            }
            pos++;
          }
          if (isMatch) {
            paragraph = list[startPos];
            break;
          }
        }
        if (paragraph) {
          return paragraph;
        }
        if (distance - 0.1 >= minDistance) {
          return this.getContentMatchParagraph(text, distance - 0.1, minDistance);
        }
      } catch (error) {
        //
      }
      return null;
    },
    showContentMatchParagraph(content) {
      if (this._inactive) {
        return;
      }
      const paragraph = this.getContentMatchParagraph(content, 1, 0.6);
      if (paragraph) {
        this.showParagraph(paragraph, true);
      } else {
        this.$message.error("无法定位内容所在段落");
      }
    },
    showBookmark(bookmark) {
      if (this._inactive) {
        return;
      }
      if (!this.$refs.bookContentRef) {
        setTimeout(() => this.showBookmark(bookmark), 10);
        return;
      }
      this.showContentMatchParagraph(bookmark.bookText);
    }
  }
};
