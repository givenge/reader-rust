import Axios from "../../../plugins/axios";
import { setCache } from "../../../plugins/cache";

export default {
  data() {
    return {
      connecting: false,
      refreshLoading: false,
      loading: null
    };
  },
  methods: {
    init(refresh) {
      this.$root.$children[0].init(refresh);
    },
    setIP() {
      this.$prompt("请输入接口地址 ( 如：localhost:8080/reader3 )", "提示", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        inputValue: this.api,
        beforeClose: (action, instance, done) => {
          if (action === "confirm") {
            this.connecting = true;
            instance.confirmButtonLoading = true;
            instance.confirmButtonText = "校验中……";
            const inputUrl = instance.inputValue.replace(/\/*$/g, "");
            this.loadBookshelf(inputUrl)
              .then(() => {
                this.connecting = false;
                instance.confirmButtonLoading = false;
                done();
                setCache("api_prefix", inputUrl);
                this.$store.commit("setApi", inputUrl);
                this.init();
              })
              .catch(() => {
                instance.confirmButtonLoading = false;
                instance.confirmButtonText = "确定";
              });
          } else {
            done();
          }
        }
      })
        .then(({ value }) => {
          this.$message({
            type: "success",
            message: "与" + value + "连接成功"
          });
        })
        .catch(() => {});
    },
    loadBookshelf(api, refresh) {
      api = api || this.api;
      if (!api) {
        this.$message.error("请先设置后端接口地址");
        this.$store.commit("setConnected", false);
        return Promise.reject(false);
      }
      if (!this.loading || !this.loading.visible) {
        this.loading = this.$loading({
          target: this.$refs.bookList,
          lock: true,
          text: refresh ? "正在刷新书籍信息" : "正在获取书籍信息",
          spinner: "el-icon-loading",
          background: this.isNight ? "#222" : "#fff"
        });
      }
      if (
        !api.startsWith("http://") &&
        !api.startsWith("https://") &&
        !api.startsWith("//")
      ) {
        api = "//" + api;
      }
      return this.$root.$children[0].loadBookShelf(refresh, api).then(() => {
        this.loading && this.loading.close();
      });
    },
    refreshShelf() {
      return this.loadBookshelf(null, true);
    },
    loadBookGroup(refresh) {
      return this.$root.$children[0].loadBookGroup(refresh);
    },
    loadBookSource(refresh) {
      return this.$root.$children[0].loadBookSource(refresh);
    },
    getChapterList(params) {
      return Axios.post(this.api + "/getChapterList", params);
    }
  }
};
