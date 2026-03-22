<template>
  <div class="popup-wrapper" :style="popupTheme">
    <div class="title-zone">
      <div class="title">
        目录
        <span v-if="catalog.length">({{ catalog.length }})</span>
      </div>
      <div :class="{ 'title-btn': true }">
        <span class="span-btn" v-if="catalog.length" @click="asc = !asc">{{
          asc ? "倒序" : "顺序"
        }}</span>
        <span class="span-btn" v-if="catalog.length" @click="toTop">顶部</span>
        <span class="span-btn" v-if="catalog.length" @click="toBottom"
          >底部</span
        >
        <span
          class="span-btn"
          v-if="book.origin === 'loc_book'"
          @click="changeRule"
        >
          修改规则
        </span>
        <span
          :class="{ loading: refreshLoading, 'refresh-btn': true }"
          @click="refreshChapter"
        >
          <i class="el-icon-loading" v-if="refreshLoading"></i>
          {{ refreshLoading ? "刷新中..." : "刷新" }}
        </span>
      </div>
    </div>
    <div
      class="data-wrapper"
      ref="cataData"
      :class="{ night: $store.getters.isNight, day: !$store.getters.isNight }"
    >
      <div class="cata">
        <div
          class="log"
          v-for="(note, index) in cataList"
          :class="{ selected: isSelected(index) }"
          :key="note.index"
          @click="gotoChapter(note)"
          ref="cata"
        >
          <div
            :class="{
              'log-text': true,
              cached: cachedCataMap[note.index]
            }"
          >
            {{ note.title }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import jump from "../plugins/jump";
import Axios from "../plugins/axios";
import Vue from "vue";

export default {
  name: "PopCata",
  data() {
    return {
      refreshLoading: false,
      asc: true,
      cachedCataMap: {},
      tocUrl: ""
    };
  },
  props: ["visible"],
  computed: {
    book() {
      return this.$store.getters.readingBook;
    },
    index() {
      return this.$store.getters.readingBook.index;
    },
    catalog() {
      return this.$store.getters.readingBook.catalog || [];
    },
    cataList() {
      if (this.asc) {
        return this.catalog;
      } else {
        return [].concat(this.catalog).reverse();
      }
    },
    theme() {
      return this.$store.getters.config.theme;
    },
    popupTheme() {
      return {
        background: this.$store.getters.currentThemeConfig.popup
      };
    },
    tocRuleList() {
      if (!this.book || !this.book.originName) {
        return [];
      }
      if (this.book.originName.toLowerCase().endsWith(".txt")) {
        // txt
        return this.$store.state.txtTocRules;
      } else {
        // epub
        return [
          { name: "根据 Spin 获取章节，使用 Toc 补充章节名", rule: "spin+toc" },
          { name: "根据 Spin 获取章节，强制使用 Toc 章节名", rule: "spin<toc" },
          { name: "根据 Spin 获取章节", rule: "spin" },
          { name: "根据 Toc 获取章节，使用 Spin 补充章节名", rule: "toc+spin" },
          { name: "根据 Toc 获取章节，强制使用 Spin 章节名", rule: "toc<spin" },
          { name: "根据 Toc 获取章节", rule: "toc" }
        ];
      }
    }
  },
  mounted() {
    window.popcatalogComp = this;
  },
  watch: {
    visible(isVisible) {
      if (isVisible) {
        this.computeCachedCata();
        this.$nextTick(() => {
          this.jumpToCurrent();
        });
      }
    },
    catalog() {
      this.refreshLoading = false;
      this.computeCachedCata();
    },
    index() {
      this.computeCachedCata();
    },
    asc() {
      this.jumpToCurrent();
    }
  },
  methods: {
    isSelected(index) {
      if (this.asc) {
        return index == this.$store.getters.readingBook.index;
      } else {
        return (
          this.catalog.length - 1 - index ==
          this.$store.getters.readingBook.index
        );
      }
    },
    gotoChapter(note) {
      const index = this.catalog.indexOf(note);
      this.$emit("close");
      this.$emit("getContent", index);
    },
    refreshChapter() {
      this.refreshLoading = true;
      this.$emit("refresh");
    },
    async changeRule() {
      const res = await this.$msgbox({
        title: "修改目录规则",
        message: this.renderComp(),
        showCancelButton: true,
        confirmButtonText: "确定",
        cancelButtonText: "取消"
      }).catch(action => {
        return action === "close" ? "close" : false;
      });
      if (res === "confirm") {
        //
        if (this.tocUrl === this.book.tocUrl) {
          this.$message.error("未修改规则");
          return;
        }
        const shelfBook = this.$store.getters.shelfBooks.find(
          v => v.bookUrl === this.book.bookUrl
        );
        shelfBook.tocUrl = this.tocUrl;
        return Axios.post(this.api + "/saveBook", shelfBook).then(
          res => {
            if (res.data.isSuccess) {
              this.$message.success("操作成功");
              this.$store.commit("updateShelfBook", res.data.data);
              this.refreshLoading = true;
              this.$emit("refresh");
            }
          },
          error => {
            this.$message.error("操作失败" + (error && error.toString()));
          }
        );
      } else {
        return false;
      }
    },
    renderComp() {
      var tocRuleList = this.tocRuleList;
      this.tocUrl = this.book.tocUrl;
      var catalog = this;
      Vue.component("custComp2", {
        render() {
          window.custComp2 = this;
          return (
            <div style={{ textAlign: "center" }}>
              <span>请选择规则：</span>
              <el-select
                size="mini"
                vModel={this.selectedRule}
                filterable={true}
                placeholder="未分组"
                vOn:change={this.change}
              >
                {tocRuleList.map((rule, index) => {
                  return (
                    <el-option
                      key={"rule-" + index}
                      label={rule.name}
                      value={rule.rule}
                    ></el-option>
                  );
                })}
              </el-select>
              <el-input
                type="textarea"
                rows={3}
                style={{ marginTop: "10px" }}
                vModel={this.selectedRule}
                size="small"
                vOn:change={this.change}
              />
            </div>
          );
        },
        data() {
          return {
            selectedRule: catalog.tocUrl
          };
        },
        methods: {
          change() {
            catalog.tocUrl = this.selectedRule;
          }
        }
      });
      var custComp2 = Vue.component("custComp2");
      return this.$createElement(custComp2);
    },
    jumpToCurrent(index) {
      if (typeof index === "undefined") {
        index = this.asc
          ? this.$store.getters.readingBook.index
          : this.catalog.length - 1 - this.$store.getters.readingBook.index;
      }
      if (!this.$refs.cata || !this.$refs.cata[index]) {
        setTimeout(() => {
          this.jumpToCurrent(index);
        }, 10);
        return;
      }
      let wrapper = this.$refs.cataData;
      jump(this.$refs.cata[index], { container: wrapper, duration: 0 });
    },
    toTop() {
      this.jumpToCurrent(0);
    },
    toBottom() {
      this.jumpToCurrent(this.catalog.length - 1);
    },
    computeCachedCata() {
      const cacheMap = {};
      const cachePrefix =
        "localCache@" +
        this.$store.getters.readingBook.name +
        "_" +
        this.$store.getters.readingBook.author +
        "@" +
        this.$store.getters.readingBook.bookUrl +
        "@chapterContent-";
      window.$cacheStorage
        .iterate(function(value, key) {
          if (key.startsWith(cachePrefix)) {
            try {
              let index = parseInt(key.replace(cachePrefix, ""));
              cacheMap[index] = true;
            } catch (error) {
              //
              // console.error(error);
            }
          }
        })
        .then(() => {
          this.cachedCataMap = cacheMap;
        })
        .catch(function() {});
    }
  }
};
</script>

<style lang="stylus" scoped>
.popup-wrapper {
  margin: -16px;
  margin-bottom: -13px;
  padding: 24px;
  padding-top: calc(24px + constant(safe-area-inset-top));
  padding-top: calc(24px + env(safe-area-inset-top));

  .title-zone {
    margin: 0 0 20px 0;
    width: 100%;
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: space-between;
  }

  .title {
    font-size: 20px;
    font-weight: 600;
    font-family: var(--font-display, 'Noto Serif SC', Georgia, serif);
    color: var(--color-text-primary, #2C2825);
    border-bottom: none;
    width: fit-content;
    position: relative;
    letter-spacing: -0.01em;
  }

  .title::after {
    content: '';
    position: absolute;
    bottom: -8px;
    left: 0;
    width: 32px;
    height: 2px;
    background: var(--color-accent, #8B4513);
    border-radius: 1px;
  }

  .title span {
    font-family: var(--font-body, 'Noto Sans SC', sans-serif);
    font-size: 14px;
    font-weight: 400;
    color: var(--color-text-tertiary, #8A857F);
    margin-left: 8px;
  }

  .title-btn {
    font-size: 14px;
    line-height: 26px;
    color: var(--color-text-secondary, #5A5550);
    .progress-percent {
      display: inline-block;
      margin-right: 25px;
    }
    .span-btn {
      display: inline-block;
      color: var(--color-text-secondary, #5A5550);
      padding: 4px 12px;
      border-radius: 9999px;
      background: var(--color-bg-secondary, #F5F2ED);
      margin-left: 8px;
      cursor: pointer;
      font-size: 13px;
      transition: all 150ms ease;
    }
    .span-btn:hover {
      background: var(--color-accent, #8B4513);
      color: #fff;
    }
    .refresh-btn {
      display: inline-block;
      margin-left: 8px;
      color: var(--color-accent, #8B4513);
      padding: 4px 12px;
      border-radius: 9999px;
      border: 1px solid var(--color-accent, #8B4513);
      cursor: pointer;
      font-size: 13px;
      transition: all 150ms ease;
      &:hover {
        background: var(--color-accent, #8B4513);
        color: #fff;
      }
      &.loading {
        color: var(--color-text-tertiary, #8A857F);
        border-color: var(--color-border, #E5E0D8);
      }
    }
  }

  .data-wrapper {
    height: 320px;
    overflow: auto;
    border-radius: 10px;
    background: var(--color-bg-secondary, #F5F2ED);
    padding: 8px;

    .cata {
      display: flex;
      flex-direction: row;
      flex-wrap: wrap;
      justify-content: space-between;

      .cached {
        color: var(--color-text-tertiary, #8A857F) !important;
      }

      .selected {
        background: var(--color-accent-soft, rgba(139, 69, 19, 0.1));
        border-radius: 6px;
      }

      .selected .log-text {
        color: var(--color-accent, #8B4513) !important;
        font-weight: 500;
      }

      .log {
        width: 50%;
        height: 44px;
        cursor: pointer;
        float: left;
        font-family: var(--font-body, 'Noto Sans SC', sans-serif);
        font-size: 14px;
        line-height: 44px;
        padding: 0 12px;
        border-radius: 6px;
        transition: all 150ms ease;
        position: relative;

        &:hover {
          background: var(--color-accent-soft, rgba(139, 69, 19, 0.1));
        }

        .log-text {
          margin-right: 0;
          overflow: hidden;
          white-space: nowrap;
          text-overflow: ellipsis;
          color: var(--color-text-primary, #2C2825);
        }
      }
    }
  }

  .data-wrapper::-webkit-scrollbar {
    width: 6px;
  }

  .data-wrapper::-webkit-scrollbar-track {
    background: var(--color-bg-tertiary, #EDE9E3);
    border-radius: 3px;
  }

  .data-wrapper::-webkit-scrollbar-thumb {
    background: var(--color-border, #E5E0D8);
    border-radius: 3px;
  }

  .night {
    background: var(--color-bg-tertiary, #2A2928) !important;

    .log {
      border-bottom: none;
    }
  }

  .day {
    .log {
      border-bottom: none;
    }
  }
}
@media screen and (max-width: 500px) {
  .popup-wrapper .data-wrapper .cata .log {
    width: 100%;
    padding: 0 16px;

    .log-text {
      margin-right: 0;
    }
  }
}
</style>
