/**
 * 书签管理模块
 * 负责书签定位、搜索匹配等功能
 */

import { defaultBookmark } from "../../../plugins/config.js";
import eventBus from "../../../plugins/eventBus";
import { editDistance } from "../../../plugins/helper";

// eslint-disable-next-line no-useless-escape
const symboRegex = /[\u2000-\u206F\u2E00-\u2E7F\\'!"#$%&\(\)*+,-\./:;<=>?@\[\]^_`{\|}~，。？《》；：、«]/g;

export default {
  data() {
    return {
      showAddBookmarking: false
    };
  },
  mounted() {
    // 监听显示书签事件
    eventBus.$on("showBookmark", bookmark => {
      if (this.$route.name === "reader") {
        this.showBookmark(bookmark);
      }
    });
  },
  methods: {
    async showAddBookmark(text) {
      if (this.showAddBookmarking) {
        return;
      }
      let pureText = text.replace(/^\s+/, "").replace(/\s+$/, "");
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
        // 补全内容
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
      bookText = bookText.replace(/\\n*$/, "");

      const bookmark = Object.assign({}, defaultBookmark, {
        bookName: this.$store.getters.readingBook.name,
        bookAuthor: this.$store.getters.readingBook.author,
        chapterIndex: this.chapterIndex,
        chapterPos: this.currentPage,
        chapterName: this.title,
        bookText: bookText,
        content: ""
      });
      this.showAddBookmarking = true;
      eventBus.$emit("showBookmarkForm", bookmark, true, () => {
        this.showAddBookmarking = false;
      });
    },

    showBookmarkDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookmarkDialog", book);
    },

    showBookmark(bookmark) {
      if (this._inactive) {
        return;
      }
      if (!this.$refs.bookContentRef) {
        setTimeout(() => {
          this.showBookmark(bookmark);
        }, 10);
        return;
      }
      this.showContentMatchParagraph(bookmark.bookText);
    },

    showMatchKeyword(data) {
      if (this._inactive) {
        return;
      }
      if (!this.$refs.bookContentRef) {
        setTimeout(() => {
          this.showMatchKeyword(data);
        }, 10);
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
          // eslint-disable-next-line no-constant-condition
          while (true) {
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
        // console.error(error);
      }
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

    getContentMatchParagraph(text, distance, minDistance) {
      distance = distance || 0.7;
      // 正则过滤标点符号后，近似匹配每一段内容
      let paragraphList = text
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
            // 过滤所有字符
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
              // 说明没找到有内容的段落，终止匹配
              isMatch = false;
              break;
            }
            const paragraphDistance = editDistance(content, paragraphList[j]);
            if (paragraphDistance < distance) {
              isMatch = false;
              break;
            } else {
              pos++;
            }
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
          return this.getContentMatchParagraph(
            text,
            distance - 0.1,
            minDistance
          );
        }
      } catch (error) {
        // eslint-disable-next-line no-console
        console.error(error);
      }
      return null;
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
