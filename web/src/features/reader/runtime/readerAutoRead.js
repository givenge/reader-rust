/**
 * 自动阅读管理模块
 * 负责自动翻页和滚动阅读功能
 */

export default {
  data() {
    return {
      autoReading: false,
      autoReadingParagraphIndex: -1,
      autoReadingProcessing: false,
      autoReadingTimer: null
    };
  },
  methods: {
    startAutoReading() {
      this.showToolBar = false;
      this.autoReading = true;
      this.autoReadingParagraphIndex = -1;
      this.autoReadingProcessing = false;
      this.autoRead();
    },
    autoRead() {
      if (!this.autoReading) {
        return;
      }
      // 防止重入
      if (this.autoReadingProcessing) {
        return;
      }
      this.autoReadingProcessing = true;

      if (this.showToolBar) {
        this.autoReadingProcessing = false;
        this.autoReadingTimer = setTimeout(() => {
          this.autoRead();
        }, 300);
        return;
      }
      if (this.config.autoReadingMethod === "像素滚动") {
        this.autoReadingProcessing = false;
        this.autoReadByPixel();
        return;
      }

      // 获取所有段落
      const allElements = this.$refs.bookContentRef.$el.querySelectorAll("h3,p");
      // 过滤掉重复内容的段落（如章节标题同时存在于h3和p中）
      const list = [];
      let lastText = '';
      allElements.forEach(el => {
        const text = el.textContent.trim();
        // 跳过空内容和与上一段重复的内容
        if (text && text !== lastText) {
          list.push(el);
          lastText = text;
        }
      });
      if (!list.length) {
        this.autoReadingProcessing = false;
        return;
      }

      // 初始化：找到视口中的第一个段落
      if (this.autoReadingParagraphIndex < 0) {
        for (let i = 0; i < list.length; i++) {
          const elePos = list[i].getBoundingClientRect();
          if (this.isSlideRead) {
            if (elePos.right > 0) {
              this.autoReadingParagraphIndex = i;
              break;
            }
          } else {
            if (elePos.bottom > 50) {
              this.autoReadingParagraphIndex = i;
              break;
            }
          }
        }
        if (this.autoReadingParagraphIndex < 0) {
          this.autoReadingParagraphIndex = 0;
        }
      }

      // 确保索引有效
      if (this.autoReadingParagraphIndex >= list.length) {
        // 到达末尾，进入下一章
        this.autoReadingParagraphIndex = -1;
        this.autoReadingProcessing = false;
        this.$once("showContent", () => {
          setTimeout(() => {
            this.autoRead();
          }, 300);
        });
        this.toNextChapter(() => {
          this.autoReading = false;
        });
        return;
      }

      const current = list[this.autoReadingParagraphIndex];

      // 清除所有段落的 reading 标记，标记当前段落
      list.forEach(el => el.classList.remove("reading"));
      current.classList.add("reading");
      this.showParagraph(current, true);

      // 计算当前段落阅读时间
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

      // 标记处理完成，准备下一段
      this.autoReadingProcessing = false;

      // 等待阅读完成后移动到下一段
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
        this.autoReadingTimer = setTimeout(() => {
          this.autoRead();
        }, 300);
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
        // console.log(delayTime, next);
        this.autoReadingTimer = setTimeout(() => {
          // 滚动
          this.scrollContent(this.config.autoReadingPixel, 0);
          this.autoReadByPixel();
        }, this.config.autoReadingLineTime);
      } else {
        // 下一章
        this.$once("showContent", () => {
          setTimeout(() => {
            this.autoReadByPixel();
          }, 100);
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
    }
  }
};