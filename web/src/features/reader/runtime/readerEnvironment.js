export default {
  data() {
    return {
      lastTouch: false,
      lastMoveX: false,
      lastMoveY: false,
      lastSelection: false,
      lastScrollTop: 0,
      scrollTimer: null,
      releaseWakeLockFn: null,
      timer: null,
      unwatchFn: null
    };
  },
  methods: {
    handleTouchStart(e) {
      this.lastSelection = this.checkSelection();
      if (this.lastSelection || this.isAudio || this.isEpub) {
        return;
      }
      this.lastTouch = false;
      this.lastMoveX = false;
      if (e.touches && e.touches[0]) {
        this.lastTouch = e.touches[0];
      }
    },
    handleTouchMove(e) {
      if (this.checkSelection()) {
        return;
      }
      if (e.touches && e.touches[0] && this.lastTouch) {
        this.lastMoveY = e.touches[0].clientY - this.lastTouch.clientY;
        if (this.isSlideRead) {
          e.preventDefault();
          e.stopPropagation();
          const moveX = e.touches[0].clientX - this.lastTouch.clientX;
          this.contentStyle = {
            transform: `translateX(${this.transformX + moveX}px)`
          };
          this.lastMoveX = moveX;
        }
      }
    },
    handleTouchEnd() {
      if (this.checkSelection(true)) {
        return;
      }
      if (this.lastSelection) {
        setTimeout(() => {
          this.showTextFilterPrompt(this.lastSelection);
          this.lastSelection = false;
        }, 200);
        return;
      }
      if (this.lastMoveX) {
        this.transformX += this.lastMoveX;
        if (this.lastMoveX > 0) {
          this.prevPage(this.windowSize.width - 16 - this.lastMoveX);
        } else {
          this.nextPage(-(this.windowSize.width - 16) - this.lastMoveX);
        }
      } else if (Math.abs(this.lastMoveY) <= 3 && this.lastTouch) {
        this.eventHandler(this.lastTouch);
      }
      setTimeout(() => {
        this.lastTouch = false;
        this.lastMoveX = false;
        this.lastMoveY = false;
      }, 300);
    },
    keydownHandler(event, force) {
      if (
        this.popBookSourceVisible ||
        this.popBookShelfVisible ||
        this.popCataVisible ||
        this.readSettingsVisible ||
        this.showTextFilterPrompting
      ) {
        return;
      }
      if (!force && document.activeElement !== document.body) {
        return;
      }
      if (this.isAudio) {
        return;
      }
      const keyCodeMap = {
        37: "ArrowLeft",
        38: "ArrowUp",
        39: "ArrowRight",
        40: "ArrowDown",
        27: "Escape"
      };
      const eventKey = event.key || keyCodeMap[event.keyCode];
      switch (eventKey) {
        case "ArrowLeft":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          if (this.isSlideRead) {
            this.prevPage();
          } else {
            this.toLastChapter();
          }
          break;
        case "ArrowRight":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          if (this.isSlideRead) {
            this.nextPage();
          } else {
            this.toNextChapter();
          }
          break;
        case "ArrowUp":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          this.prevPage();
          break;
        case "ArrowDown":
          event.preventDefault && event.preventDefault();
          event.stopPropagation && event.stopPropagation();
          this.showToolBar = false;
          this.nextPage();
          break;
        case "Escape":
          this.toShelf();
          break;
      }
    },
    scrollHandler() {
      const scrollTop =
        document.documentElement.scrollTop || document.body.scrollTop;
      if (!this.isSlideRead) {
        this.currentPage = Math.round(
          (scrollTop + this.windowSize.height) /
            (this.windowSize.height - this.scrollOffset)
        );
      }
      if (this.isScrollRead) {
        const lastScrollTop = this.lastScrollTop || 0;
        if (
          !(lastScrollTop > 0 && scrollTop === 0) &&
          scrollTop >
            document.documentElement.scrollHeight - 2 * this.windowSize.height &&
          !this.preCaching &&
          this.startSavePosition
        ) {
          this.preCaching = true;
          let nextIndex = this.chapterIndex + 1;
          if (this.showChapterList.length) {
            nextIndex =
              this.showChapterList[this.showChapterList.length - 1].index + 1;
          }
          this.showNextChapterSize = nextIndex - this.chapterIndex;
          this.loadShowChapter(nextIndex)
            .then(() => {
              this.computeShowChapterList();
              this.preCaching = false;
            })
            .catch(() => {
              this.preCaching = false;
            });
        }
      }
      this.lastScrollTop = scrollTop;
      this.scrollTimer && clearTimeout(this.scrollTimer);
      this.scrollTimer = setTimeout(this.saveReadingPosition, 100);
    },
    wakeLock() {
      if ("WakeLock" in window && "request" in window.WakeLock) {
        let wakeLock = null;
        const requestWakeLock = () => {
          const controller = new AbortController();
          const signal = controller.signal;
          window.WakeLock.request("screen", { signal }).catch(() => {});
          return controller;
        };
        wakeLock = requestWakeLock();
        const handleVisibilityChange = () => {
          if (wakeLock !== null && document.visibilityState === "visible") {
            wakeLock = requestWakeLock();
          }
        };
        document.addEventListener("visibilitychange", handleVisibilityChange);
        document.addEventListener("fullscreenchange", handleVisibilityChange);
        return () => {
          if (wakeLock != null) {
            wakeLock.abort();
            wakeLock = null;
          }
          document.removeEventListener(
            "visibilitychange",
            handleVisibilityChange
          );
          document.removeEventListener(
            "fullscreenchange",
            handleVisibilityChange
          );
        };
      }
      if ("wakeLock" in navigator && "request" in navigator.wakeLock) {
        let wakeLock = null;
        const requestWakeLock = async () => {
          try {
            wakeLock = await navigator.wakeLock.request("screen");
            wakeLock.addEventListener("release", () => {});
          } catch (e) {
            return e;
          }
        };
        requestWakeLock();
        const handleVisibilityChange = () => {
          if (wakeLock !== null && document.visibilityState === "visible") {
            requestWakeLock();
          }
        };
        document.addEventListener("visibilitychange", handleVisibilityChange);
        document.addEventListener("fullscreenchange", handleVisibilityChange);
        return () => {
          if (wakeLock !== null) {
            wakeLock.release();
            wakeLock = null;
          }
          document.removeEventListener(
            "visibilitychange",
            handleVisibilityChange
          );
          document.removeEventListener(
            "fullscreenchange",
            handleVisibilityChange
          );
        };
      }
      return null;
    },
    lazyloadHandler() {
      if (!this.isAudio) {
        this.computePages();
      }
    }
  }
};
