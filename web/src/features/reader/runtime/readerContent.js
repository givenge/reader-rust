export default {
  data() {
    return {
      chapterContentCache: null,
      tryRefresh: false,
      preCaching: false
    };
  },
  methods: {
    getBookContent(chapterIndex, options, refresh, cache) {
      return this.$root.$children[0].getBookContent(
        chapterIndex,
        options,
        refresh,
        cache
      );
    },
    refreshContent() {
      this.getContent(this.$store.getters.readingBook.index, true);
    },
    getContent(index, refresh) {
      this.show = false;
      if (!this.loading || !this.loading.visible) {
        this.loading = this.$loading({
          target: this.$refs.content,
          lock: true,
          text: refresh ? "正在刷新内容" : "正在获取内容",
          spinner: "el-icon-loading",
          background: "rgba(0,0,0,0)"
        });
      }
      const bookUrl = this.$store.getters.readingBook.bookUrl;
      try {
        const book = { ...this.$store.getters.readingBook };
        book.index = index;
        this.$store.commit("setReadingBook", book);
      } catch (error) {
        //
      }
      this.toTop(0);
      if (!this.$store.getters.readingBook.catalog[index]) {
        if (this.tryRefresh) {
          this.tryRefresh = false;
          this.content = "获取章节内容失败，请更新目录！";
          this.error = true;
          this.show = true;
          this.$emit("showContent");
          this.loading.close();
        } else {
          this.tryRefresh = true;
          this.refreshCatalog();
        }
        return;
      }
      const chapterName = this.$store.getters.readingBook.catalog[index].title;
      const chapterIndex = this.$store.getters.readingBook.catalog[index].index;
      this.title = chapterName;
      const now = new Date().getTime();
      this.getBookContent(chapterIndex, {}, refresh).then(
        res => {
          if (
            bookUrl !== this.$store.getters.readingBook.bookUrl ||
            index !== this.$store.getters.readingBook.index
          ) {
            return;
          }
          if (now + 100 > new Date().getTime()) {
            this.saveBookProgress();
          }
          if (res.data.isSuccess) {
            this.content = this.filterContent(res.data.data);
            this.addChapterContentToCache({
              bookUrl,
              index,
              title: chapterName,
              content: res.data.data,
              error: false
            });
            this.loading.close();
            this.error = false;
            this.show = true;
            this.$emit("showContent");
          } else {
            this.content = "获取章节内容失败！\n" + res.data.errorMsg;
            this.addChapterContentToCache({
              bookUrl,
              index,
              title: chapterName,
              content: "获取章节内容失败！\n" + res.data.errorMsg,
              error: true
            });
            this.error = true;
            this.show = true;
            this.$emit("showContent");
            this.loading.close();
          }
          if (this.isScrollRead) {
            this.computeShowChapterList();
          }
        },
        error => {
          if (
            bookUrl !== this.$store.getters.readingBook.bookUrl ||
            index !== this.$store.getters.readingBook.index
          ) {
            return;
          }
          this.content = "获取章节内容失败！\n" + (error && error.toString());
          this.addChapterContentToCache({
            bookUrl,
            index,
            title: chapterName,
            content: "获取章节内容失败！\n" + (error && error.toString()),
            error: true
          });
          this.error = true;
          this.show = true;
          this.$emit("showContent");
          this.loading.close();
          this.$message.error("获取章节内容失败 " + (error && error.toString()));
          if (this.isScrollRead) {
            this.computeShowChapterList();
          }
          throw error;
        }
      );
    },
    filterContent(content) {
      if (this.isEpub || this.isAudio || !content) {
        return content;
      }
      try {
        this.filterRules.forEach(rule => {
          if (
            typeof rule.isEnabled !== "undefined" &&
            rule.isEnabled === false
          ) {
            return;
          }
          const scope = rule.scope.split(";");
          if (
            scope[0] === "*" ||
            scope[0] === this.$store.getters.readingBook.name
          ) {
            if (
              scope.length === 1 ||
              (scope.length > 1 &&
                scope[1] === this.$store.getters.readingBook.bookUrl)
            ) {
              if (rule.isRegex) {
                content = content.replace(
                  new RegExp(rule.pattern, "ig"),
                  rule.replacement
                );
              } else {
                content = content.replace(rule.pattern, rule.replacement);
              }
            }
          }
        });
      } catch (error) {
        //
      }
      content.replace(/\\n+/g, "\n");
      return this.formatChinese(content);
    },
    loadShowChapter(index, refresh) {
      if (
        !refresh &&
        this.chapterContentCache &&
        this.chapterContentCache.chapters[index] &&
        !this.chapterContentCache.chapters[index].error
      ) {
        if (
          index >= this.chapterIndex - this.showPrevChapterSize &&
          index <= this.chapterIndex + this.showNextChapterSize
        ) {
          this.computeShowChapterList();
        }
        return Promise.resolve();
      }
      const bookUrl = this.$store.getters.readingBook.bookUrl;
      if (!this.$store.getters.readingBook.catalog) {
        return new Promise(resolve => {
          this.$once("loadCatalog", () => {
            this.loadShowChapter(index, refresh).then(resolve);
          });
        });
      }
      if (!this.$store.getters.readingBook.catalog[index]) {
        return Promise.reject("章节不存在");
      }
      const chapterName = this.$store.getters.readingBook.catalog[index].title;
      const chapterIndex = this.$store.getters.readingBook.catalog[index].index;
      return this.getBookContent(chapterIndex, {}, refresh, true).then(
        res => {
          if (res.data.isSuccess) {
            this.addChapterContentToCache({
              bookUrl,
              index,
              title: chapterName,
              content: res.data.data,
              error: false
            });
          } else {
            this.addChapterContentToCache({
              bookUrl,
              index,
              title: chapterName,
              content: "获取章节内容失败！\n" + res.data.errorMsg,
              error: true
            });
          }
        },
        error => {
          this.addChapterContentToCache({
            bookUrl,
            index,
            title: chapterName,
            content: "获取章节内容失败！\n" + (error && error.toString()),
            error: true
          });
          throw error;
        }
      );
    },
    addChapterContentToCache(chapter) {
      if (
        !this.chapterContentCache ||
        this.chapterContentCache.bookUrl !== this.readingBook.bookUrl
      ) {
        this.chapterContentCache = {
          bookUrl: this.readingBook.bookUrl,
          chapters: {}
        };
      }
      if (
        typeof this.chapterContentCache.chapters[chapter.index] ===
          "undefined" ||
        !chapter.error ||
        this.chapterContentCache.chapters[chapter.index].error
      ) {
        chapter.isVolume = !!(this.readingBook.catalog[chapter.index] || {})
          .isVolume;
        this.chapterContentCache.chapters[chapter.index] = chapter;
      }
    },
    computeShowChapterList(reset) {
      if (!this.chapterContentCache) {
        return new Promise(resolve => {
          setTimeout(() => {
            this.computeShowChapterList(reset).then(resolve);
          }, 10);
        });
      }
      if (!this.isScrollRead) {
        return Promise.resolve();
      }
      const list = [];
      let startIndex = this.scrollStartChapterIndex || this.chapterIndex;
      if (this.config.readMethod === "上下滚动2") {
        startIndex = this.chapterIndex - this.showPrevChapterSize;
      }
      const waitPromise = [];
      for (let i = startIndex; i <= this.chapterIndex + this.showNextChapterSize; i++) {
        if (!this.chapterContentCache.chapters[i]) {
          waitPromise.push(this.loadShowChapter(i));
          continue;
        }
        list.push({
          ...this.chapterContentCache.chapters[i],
          content: this.filterContent(this.chapterContentCache.chapters[i].content)
        });
      }
      if (waitPromise.length) {
        return Promise.all(waitPromise).then(() => {
          this.computeShowChapterList(reset);
        });
      }
      this.saveReadingPosition();
      this.startSavePosition = false;
      this.showChapterList = list;
      this.$nextTick(() => {
        this.computePages(() => {
          if (reset) {
            this.toTop(0);
            this.startSavePosition = true;
          } else if (this.config.readMethod === "上下滚动2") {
            this.autoShowPosition(true);
          } else {
            this.startSavePosition = true;
          }
        });
      });
    }
  }
};
