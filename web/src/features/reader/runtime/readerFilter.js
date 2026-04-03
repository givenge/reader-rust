/**
 * 文本过滤模块
 * 负责文本选择检测、过滤规则弹窗、简繁转换等
 */

import { simplized, traditionalized } from "../../../plugins/chinese";
import { defaultReplaceRule } from "../../../plugins/config.js";
import eventBus from "../../../plugins/eventBus";

export default {
  data() {
    return {
      showTextFilterPrompting: false
    };
  },
  methods: {
    checkSelection(show) {
      let text = "";
      if (window.getSelection) {
        text = window.getSelection().toString();
      } else if (document.selection && document.selection.type != "Control") {
        text = document.selection.createRange().text;
      }
      if (text && show) {
        setTimeout(() => {
          if (
            this.$store.getters.config.selectionAction === "过滤弹窗" ||
            this.$store.getters.config.selectionAction === "操作弹窗"
          ) {
            this.showTextOperate(text);
          }
        }, 200);
      }
      return text;
    },

    async showTextOperate(text) {
      const res = await this.$confirm("请选择操作?", "提示", {
        confirmButtonText: "添加过滤规则",
        cancelButtonText: "添加书签",
        type: "warning",
        closeOnClickModal: false,
        closeOnPressEscape: false,
        distinguishCancelAndClose: true
      }).catch(action => {
        return action === "close" ? "close" : false;
      });
      if (res === "close") {
        return;
      }
      if (res) {
        return this.showTextFilterPrompt(text);
      }
      return this.showAddBookmark(text);
    },

    async showTextFilterPrompt(text) {
      if (this.showTextFilterPrompting) {
        return;
      }
      if (!text.replace(/^\s+/, "").replace(/\s+$/, "")) {
        return;
      }

      const replaceRule = Object.assign({}, defaultReplaceRule, {
        name: "文本替换",
        pattern: text,
        replacement: "",
        isRegex: false,
        isEnabled: true,
        scope:
          this.$store.getters.readingBook.name +
          ";" +
          this.$store.getters.readingBook.bookUrl
      });
      this.showTextFilterPrompting = true;
      eventBus.$emit("showReplaceRuleForm", replaceRule, true, () => {
        this.showTextFilterPrompting = false;
      });
    },

    formatChinese(text) {
      if (this.isEpub || this.isAudio || this.isCbz || this.isCarToon) {
        return text;
      }
      if (this.config.chineseFont === "简体") {
        return simplized(text);
      } else {
        return traditionalized(text);
      }
    }
  }
};
