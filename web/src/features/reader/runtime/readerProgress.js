import Axios from "../../../plugins/axios";
import { setCache, getCache } from "../../../plugins/cache";

function getProgressCacheKey(book) {
  return "bookChapterProgress@" + book.name + "_" + book.author;
}

export default {
  data() {
    return {
      currentParagraph: null,
      startSavePosition: false
    };
  },
  methods: {
    showPosition(pos, callback) {
      if (this.isAudio) {
        if (!this.$refs.bookContentRef) {
          setTimeout(() => {
            this.showPosition(pos, callback);
          }, 10);
          return;
        }
        this.$refs.bookContentRef.ensureSeekTime(pos);
      } else if (this.isEpub || this.isCarToon) {
        this.scrollContent(pos, 0, true);
        if (this.isEpub) {
          this.$once("iframeLoad", () => {
            this.scrollContent(pos, 0, true);
            callback && callback();
          });
        }
      } else {
        if (!this.$refs.bookContentRef) {
          setTimeout(() => {
            this.showPosition(pos, callback);
          }, 10);
          return;
        }
        const list = this.$refs.bookContentRef.$el.querySelectorAll(
          ".reading-chapter h3,p"
        );
        for (let i = 0; i < list.length; i++) {
          if (
            list[i].dataset &&
            typeof list[i].dataset.pos !== "undefined" &&
            +list[i].dataset.pos >= pos
          ) {
            this.showParagraph(list[i], true);
            break;
          }
        }
        callback && callback();
      }
    },
    saveReadingPosition() {
      try {
        if (this.error || !this.startSavePosition) {
          return;
        }
        let position = 0;
        if (this.isAudio) {
          position = this.$refs.bookContentRef
            ? this.$refs.bookContentRef.currentTime
            : 0;
        } else if (this.isEpub || this.isCarToon) {
          position =
            document.documentElement.scrollTop || document.body.scrollTop;
        } else {
          if (this.preCaching) {
            return;
          }
          this.currentParagraph = this.getCurrentParagraph();
          if (this.currentParagraph) {
            let currentChapter = this.currentParagraph;
            while (currentChapter.className.indexOf("chapter-content") < 0) {
              currentChapter = currentChapter.parentNode;
              if (currentChapter === this.$refs.bookContentRef.$el) {
                break;
              }
            }
            if (currentChapter) {
              if (
                currentChapter.dataset &&
                typeof currentChapter.dataset.index !== "undefined"
              ) {
                const chapterIndex = +currentChapter.dataset.index;
                if (chapterIndex !== this.$store.getters.readingBook.index) {
                  const book = { ...this.$store.getters.readingBook };
                  book.index = chapterIndex;
                  this.$store.commit("setReadingBook", book);
                  this.saveBookProgress();
                  this.title = this.$store.getters.readingBook.catalog[
                    chapterIndex
                  ].title;
                }
              }
              position = currentChapter.innerText.indexOf(
                this.currentParagraph.innerText
              );
            }
          }
        }
        setCache(
          getProgressCacheKey(this.$store.getters.readingBook),
          position
        );
      } catch (error) {
        //
      }
    },
    autoShowPosition(immediate) {
      const handler = () => {
        setTimeout(() => {
          this.startSavePosition = true;
        }, 2000);
        if (this.error) {
          return;
        }
        const lastPosition = getCache(
          getProgressCacheKey(this.$store.getters.readingBook)
        );
        if (lastPosition && +lastPosition) {
          this.$nextTick(() => {
            this.showPosition(+lastPosition, () => {
              this.startSavePosition = true;
            });
          });
        }
      };
      if (immediate) {
        handler();
      } else {
        this.$once("showContent", handler);
      }
    },
    saveBookProgress() {
      return Axios.post(
        this.api + "/saveBookProgress",
        {
          url: this.$store.getters.readingBook.bookUrl,
          index: this.chapterIndex
        },
        {
          silent: true
        }
      ).catch(() => {});
    }
  }
};
