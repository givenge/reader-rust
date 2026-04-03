/**
 * 段落选择模块
 * 负责获取过滤的段落列表、当前段落、上一段/下一段、显示段落等
 */

export default {
  methods: {
    // 获取过滤后的段落列表（去除重复内容）
    getFilteredParagraphs() {
      const allElements = this.$refs.bookContentRef.$el.querySelectorAll("h3,p");
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
      return list;
    },

    getCurrentParagraph() {
      const readingEle = this.$refs.bookContentRef.$el.querySelectorAll(".reading");
      let currentParagraph = null;
      if (!readingEle.length) {
        // 没有正在读的段落，遍历找到当前页面的第一段
        const list = this.getFilteredParagraphs();
        for (let i = 0; i < list.length; i++) {
          const elePos = list[i].getBoundingClientRect();
          if (this.isSlideRead) {
            // 段尾出现在视野里
            if (elePos.right > 0) {
              currentParagraph = list[i];
              break;
            }
          } else {
            // 段尾出现在视野里
            if (
              elePos.bottom >
              30 +
              20 +
              (window.webAppDistance | 0) +
              (this.$store.state.safeArea.top | 0)
            ) {
              currentParagraph = list[i];
              break;
            }
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

    showParagraph(paragraph, scroll) {
      if (!paragraph) {
        return;
      }
      if (this.isSlideRead) {
        // 跳转位置
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
        // 跳转位置
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

    getParagraphListInView() {
      // 获取视口内的所有段落
      const list = this.$refs.bookContentRef.$el.querySelectorAll("h3,p");
      const paragraphList = [];
      for (let i = 0; i < list.length; i++) {
        const elePos = list[i].getBoundingClientRect();
        if (this.isSlideRead) {
          // 段尾出现在视野里
          if (elePos.right > 0 && elePos.left > 0) {
            paragraphList.push(list[i]);
          }
        } else {
          // 段尾出现在视野里
          if (
            elePos.bottom >
            30 +
            20 +
            (window.webAppDistance | 0) +
            (this.$store.state.safeArea.top | 0) &&
            elePos.bottom < this.windowSize.height
          ) {
            paragraphList.push(list[i]);
          }
        }
      }
      return paragraphList;
    }
  }
};
