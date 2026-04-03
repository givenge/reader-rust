import Animate from "../../../plugins/animate";

export default {
  data() {
    return {
      currentPage: 1,
      totalPages: 1,
      transformX: 0,
      transforming: false,
      showLastPage: false,
      contentStyle: {}
    };
  },
  methods: {
    computePages(cb) {
      if (!this.$refs.bookContentRef || !this.$refs.bookContentRef.$el) {
        setTimeout(() => {
          this.computePages(cb);
        }, 30);
        return;
      }
      if (this.isSlideRead) {
        this.totalPages = Math.ceil(
          this.$refs.bookContentRef.$el.scrollWidth /
            (this.windowSize.width - 16)
        );
      } else {
        this.totalPages = Math.ceil(
          this.$refs.bookContentRef.$el.scrollHeight /
            (this.windowSize.height - this.scrollOffset)
        );
      }
      if (this.showLastPage) {
        this.showPage(this.totalPages, 0);
        this.showLastPage = false;
      }
      cb && cb();
    },
    nextPage(moveX) {
      if (!this.show || this.transforming) {
        return;
      }
      if (this.isSlideRead) {
        if (this.currentPage < this.totalPages) {
          if (typeof moveX === "undefined") {
            this.transformX =
              -(this.windowSize.width - 16) * (this.currentPage - 1);
          }
          this.currentPage += 1;
          this.transforming = true;
          this.transform(
            typeof moveX === "undefined"
              ? -(this.windowSize.width - 16)
              : moveX,
            this.animateMSTime
          );
        } else {
          this.toNextChapter(() => {
            if (typeof moveX !== "undefined") {
              this.showPage(this.currentPage, 0);
            }
          });
        }
      } else if (
        (document.documentElement.scrollTop || document.body.scrollTop) +
          this.windowSize.height <
        document.documentElement.scrollHeight
      ) {
        this.currentPage += 1;
        const moveY = this.windowSize.height - this.scrollOffset;
        this.transforming = true;
        this.scrollContent(moveY, this.animateMSTime);
      } else {
        this.currentPage = 1;
        this.toNextChapter();
      }
    },
    prevPage(moveX) {
      if (!this.show || this.transforming) {
        return;
      }
      if (this.isSlideRead) {
        if (this.currentPage > 1) {
          if (typeof moveX === "undefined") {
            this.transformX =
              -(this.windowSize.width - 16) * (this.currentPage - 1);
          }
          this.currentPage -= 1;
          this.transforming = true;
          this.transform(
            typeof moveX === "undefined" ? this.windowSize.width - 16 : moveX,
            this.animateMSTime
          );
        } else {
          this.showLastPage = true;
          this.toLastChapter(() => {
            if (typeof moveX !== "undefined") {
              this.showPage(this.currentPage, 0);
            }
          });
        }
      } else if (
        (document.documentElement.scrollTop || document.body.scrollTop) > 0
      ) {
        this.currentPage -= 1;
        const moveY = -(this.windowSize.height - this.scrollOffset);
        this.transforming = true;
        this.scrollContent(moveY, this.animateMSTime);
      } else {
        this.toLastChapter();
      }
    },
    showPage(page, duration) {
      if (!this.show) {
        return;
      }
      this.currentPage = Math.min(page, this.totalPages);
      if (this.isSlideRead) {
        const moveX =
          -(this.windowSize.width - 16) * (this.currentPage - 1) -
          this.transformX;
        this.transform(
          moveX,
          typeof duration === "undefined" ? this.animateMSTime : duration
        );
      } else {
        const moveY =
          (this.windowSize.height - 10) * (this.currentPage - 1) -
          (document.documentElement.scrollTop || document.body.scrollTop);
        this.scrollContent(
          moveY,
          typeof duration === "undefined" ? this.animateMSTime : duration
        );
      }
    },
    transform(moveX, duration) {
      const onEnd = () => {
        this.contentStyle = {
          transform: `translateX(${this.transformX + moveX}px)`
        };
        this.transformX += moveX;
        this.transforming = false;
        setTimeout(this.saveReadingPosition, duration);
      };
      if (!duration) {
        onEnd();
        return;
      }
      const timing = Animate.Utils.makeEaseInOut(
        Animate.Timings.power.bind(null, 3)
      );

      new Animate({
        duration: duration || 500,
        timing,
        draw: progress => {
          this.contentStyle = {
            transform: `translateX(${this.transformX + moveX * progress}px)`
          };
        },
        onEnd
      });
    },
    scrollContent(moveY, duration, isAccurate) {
      const lastScrollTop = isAccurate
        ? 0
        : document.documentElement.scrollTop || document.body.scrollTop;
      const onEnd = () => {
        document.documentElement.scrollTop = lastScrollTop + moveY;
        document.body.scrollTop = lastScrollTop + moveY;
        this.transforming = false;
        setTimeout(this.saveReadingPosition, duration);
      };
      if (!duration) {
        onEnd();
        return;
      }
      const timing = Animate.Utils.makeEaseInOut(
        Animate.Timings.power.bind(null, 3)
      );

      new Animate({
        duration: duration || 500,
        timing,
        draw: progress => {
          document.documentElement.scrollTop = lastScrollTop + moveY * progress;
          document.body.scrollTop = lastScrollTop + moveY * progress;
        },
        onEnd
      });
    }
  }
};
