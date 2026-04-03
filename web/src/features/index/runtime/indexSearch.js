import Axios from "../../../plugins/axios";
import { HeaderEventSource } from "../../../plugins/sse";

const buildURL = require("axios/lib/helpers/buildURL");

export default {
  data() {
    return {
      search: "",
      searchTypeList: [
        { name: "单源搜索", value: "single" },
        { name: "多源搜索(过滤书名/作者名)", value: "multi" }
      ],
      isSearchResult: false,
      isExploreResult: false,
      searchResult: [],
      searchPage: 1,
      searchLastIndex: -1,
      loadingMore: false,
      searchEventSource: null
    };
  },
  beforeDestroy() {
    this.closeSearchEventSource();
  },
  methods: {
    closeSearchEventSource() {
      try {
        if (
          this.searchEventSource &&
          this.searchEventSource.readyState !== this.searchEventSource.CLOSED
        ) {
          this.searchEventSource.close();
        }
      } catch (error) {
        //
      }
      this.searchEventSource = null;
    },
    searchBook(page) {
      if (!this.$store.state.connected) {
        this.$message.error("后端未连接");
        return;
      }
      if (!this.search) {
        this.$message.error("请输入关键词进行搜索");
        return;
      }
      if (
        this.searchConfig.searchType === "single" &&
        !this.searchConfig.bookSourceUrl
      ) {
        this.$message.error("请选择书源进行搜索");
        return;
      }
      if (page) {
        this.searchPage = page;
      }
      page = this.searchPage;
      if (page === 1) {
        this.searchLastIndex = -1;
      }
      if (this.searchConfig.searchType === "multi" && window.EventSource) {
        this.searchBookByEventStream(page);
        return;
      }
      if (this.loadingMore) {
        return;
      }
      this.isSearchResult = true;
      this.isExploreResult = false;
      this.loadingMore = true;
      if (page === 1) {
        this.searchResult = [];
      }
      const isSingleSearch = this.searchConfig.searchType === "single";
      Axios.post(
        this.api + (isSingleSearch ? "/searchBook" : "/searchBookMulti"),
        {
          key: this.search,
          bookSourceUrl: isSingleSearch
            ? this.searchConfig.bookSourceUrl
            : undefined,
          bookSourceGroup: this.searchConfig.bookSourceGroup,
          concurrentCount: this.searchConfig.concurrentCount,
          lastIndex: this.searchLastIndex,
          page
        },
        {
          timeout: isSingleSearch ? 30000 : 180000
        }
      ).then(
        res => {
          this.loadingMore = false;
          if (!res.data.isSuccess) {
            return;
          }
          let resultList = [];
          if (this.searchConfig.searchType === "single") {
            resultList = res.data.data;
          } else {
            this.searchLastIndex = res.data.data.lastIndex;
            resultList = res.data.data.list;
          }
          const data = [].concat(this.searchResult);
          const length = data.length;
          resultList.forEach(v => {
            if (!this.searchResultMap[v.bookUrl]) {
              data.push(v);
            }
          });
          this.searchResult = data;
          if (data.length === length) {
            this.$message.error("没有更多啦");
          }
        },
        error => {
          this.loadingMore = false;
          this.$message.error("搜索书籍失败 " + (error && error.toString()));
        }
      );
    },
    searchBookByEventStream(page) {
      if (this.loadingMore) {
        this.closeSearchEventSource();
        this.loadingMore = false;
        if (page !== 1) {
          return;
        }
      }
      const params = {
        key: this.search,
        bookSourceGroup: this.searchConfig.bookSourceGroup,
        concurrentCount: this.searchConfig.concurrentCount,
        lastIndex: this.searchLastIndex,
        page
      };
      this.isSearchResult = true;
      this.isExploreResult = false;
      this.loadingMore = true;
      if (page === 1) {
        this.searchResult = [];
      }
      const url = buildURL(this.api + "/searchBookMultiSSE", params);
      this.closeSearchEventSource();
      this.searchEventSource = new HeaderEventSource(url, {
        withCredentials: true
      });
      this.searchEventSource.addEventListener("error", e => {
        this.loadingMore = false;
        this.closeSearchEventSource();
        try {
          if (e.data) {
            const result = JSON.parse(e.data);
            if (result && result.errorMsg) {
              this.$message.error(result.errorMsg);
            }
          }
        } catch (error) {
          //
        }
      });
      const oldSearchResultLength = this.searchResult.length;
      this.searchEventSource.addEventListener("end", e => {
        this.loadingMore = false;
        this.closeSearchEventSource();
        try {
          if (e.data) {
            const result = JSON.parse(e.data);
            if (result && result.lastIndex) {
              this.searchLastIndex = result.lastIndex;
            }
          }
          if (this.searchResult.length === oldSearchResultLength) {
            this.$message.error("没有更多啦");
          }
        } catch (error) {
          //
        }
      });
      this.searchEventSource.addEventListener("message", e => {
        try {
          if (!e.data) {
            return;
          }
          const result = JSON.parse(e.data);
          if (result && result.lastIndex) {
            this.searchLastIndex = result.lastIndex;
          }
          if (result.data) {
            const data = [].concat(this.searchResult);
            result.data.forEach(v => {
              if (!this.searchResultMap[v.bookUrl]) {
                data.push(v);
              }
            });
            this.searchResult = data;
          }
        } catch (error) {
          //
        }
      });
    }
  }
};
