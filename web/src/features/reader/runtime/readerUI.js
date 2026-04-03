/**
 * UI操作模块
 * 负责UI交互、显示弹窗、状态切换等
 */

import eventBus from "../../../plugins/eventBus";

export default {
  data() {
    return {
      showCacheContentZone: false
    };
  },
  methods: {
    formatProgressTip(value) {
      return `第 ${value || this.progressValue}/${this.totalPages} 页`;
    },

    formatTime() {
      const now = new Date();
      const pad = v => (v >= 10 ? "" + v : "0" + v);
      this.timeStr = pad(now.getHours()) + ":" + pad(now.getMinutes());
    },

    toogleNight() {
      this.$store.commit("setNightTheme", !this.isNight);
    },

    showCacheContent() {
      this.showCacheContentZone = !this.showCacheContentZone;
    },

    showReadingBookInfo() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookInfoDialog", book);
    },

    showSearchBookContentDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showSearchBookContentDialog", book);
    },

    showBookmarkDialog() {
      let book = { ...this.$store.getters.readingBook };
      const shelfBook = this.$store.getters.shelfBooks.find(
        v => v.bookUrl === book.bookUrl
      );
      book = Object.assign(book, shelfBook || {});
      eventBus.$emit("showBookmarkDialog", book);
    },

    exitRead() {
      this.stopSpeech();
      const current = this.getCurrentParagraph();
      this.showReadBar = false;
      this.showParagraph(current);
    }
  }
};
