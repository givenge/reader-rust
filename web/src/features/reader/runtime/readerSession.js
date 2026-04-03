import Axios from "../../../plugins/axios";
import { networkFirstRequest } from "../../../plugins/helper";

export default {
  data() {
    return {
      lastReadingBook: null,
      loading: null
    };
  },
  methods: {
    init(refresh) {
      if (!this.$store.getters.readingBook) {
        this.$message.error("请在书架选择书籍");
        return;
      }
      if (
        refresh ||
        !this.lastReadingBook ||
        this.lastReadingBook.bookUrl !== this.$store.getters.readingBook.bookUrl
      ) {
        this.title = "";
        this.show = false;
        this.loading = this.$loading({
          target: this.$refs.content,
          lock: true,
          text: "正在获取内容",
          spinner: "el-icon-loading",
          background: "rgba(0,0,0,0)"
        });
        this.lastReadingBook = this.$store.getters.readingBook;
        this.autoShowPosition();
        this.loadCatalog(false, true);
        return;
      }
      if (this.isScrollRead) {
        this.scrollStartChapterIndex = this.chapterIndex;
        this.showPrevChapterSize = 0;
        this.computeShowChapterList().then(() => {
          this.autoShowPosition(true);
        });
      } else if (this.isEpub) {
        this.autoShowPosition(true);
      } else {
        this.startSavePosition = true;
      }
      setTimeout(() => {
        this.$store.commit("setReadingBook", this.lastReadingBook);
      }, 100);
    },
    changeBook(book) {
      this.$message.info("换书成功");
      this.popBookShelfVisible = false;
      this.show = false;
      this.$store.commit("setReadingBook", book);
      this.loadCatalog(true, true);
    },
    changeBookSource() {
      this.popBookSourceVisible = false;
      this.show = false;
      this.tryRefresh = false;
      this.loadCatalog(true, true);
    },
    loadCatalog(refresh, init) {
      if (!this.api) {
        setTimeout(() => {
          if (this.loadCatalog) {
            this.loadCatalog(refresh, init);
          }
        }, 1000);
        return;
      }
      this.getCatalog(refresh).then(
        res => {
          if (res.data.isSuccess) {
            const book = Object.assign({}, this.$store.getters.readingBook);
            book.catalog = res.data.data;
            this.$store.commit("setReadingBook", book);
            this.$emit("loadCatalog");
            const index = book.index || 0;
            this.getContent(index);
          } else {
            if (init) {
              this.title = "";
              this.content = "获取章节目录失败！\n" + res.data.errorMsg;
              this.error = true;
              this.show = true;
              this.$emit("showContent");
            }
            this.loading && this.loading.close();
          }
        },
        error => {
          this.loading && this.loading.close();
          this.$message.error("获取书籍目录列表 " + (error && error.toString()));
        }
      );
    },
    getCatalog(refresh) {
      const params = {
        url: this.$store.getters.readingBook.bookUrl,
        refresh: refresh ? 1 : 0
      };
      if (this.$route.query.search) {
        params.bookSourceUrl = this.$store.getters.readingBook.origin;
      }
      return networkFirstRequest(
        () => Axios.post(this.api + "/getChapterList", params),
        this.$store.getters.readingBook.name +
          "_" +
          this.$store.getters.readingBook.author +
          "@" +
          this.$store.getters.readingBook.bookUrl +
          "@chapterList"
      );
    },
    refreshCatalog() {
      return this.loadCatalog(true, true);
    }
  }
};
