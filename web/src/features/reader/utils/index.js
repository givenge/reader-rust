/**
 * 获取当前格式化时间
 * @returns {string} HH:mm
 */
export function getFormattedTime() {
  const now = new Date();
  const pad = v => (v >= 10 ? "" + v : "0" + v);
  return pad(now.getHours()) + ":" + pad(now.getMinutes());
}

/**
 * 格式化进度提示文字
 * @param {number} value
 * @param {number} progressValue
 * @param {number} totalPages
 * @returns {string}
 */
export function formatProgressTip(value, progressValue, totalPages) {
  return `第 ${value || progressValue}/${totalPages} 页`;
}

/**
 * 获取进度的缓存 Key
 * @param {object} book
 * @returns {string}
 */
export function getProgressCacheKey(book) {
  if (!book) return "";
  return "bookChapterProgress@" + book.name + "_" + book.author;
}

/**
 * 请求屏幕常亮唤醒锁 (WakeLock)
 * @returns {function|null} 返回取消的函数，或 null
 */
export function requestWakeLock() {
  if ("WakeLock" in window && "request" in window.WakeLock) {
    let wakeLock = null;
    const requestWakeLockHandler = () => {
      const controller = new AbortController();
      const signal = controller.signal;
      window.WakeLock.request("screen", { signal }).catch(() => {
        // failed to lock
      });
      return controller;
    };

    wakeLock = requestWakeLockHandler();

    const handleVisibilityChange = () => {
      if (wakeLock !== null && document.visibilityState === "visible") {
        wakeLock = requestWakeLockHandler();
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
  } else if ("wakeLock" in navigator && "request" in navigator.wakeLock) {
    let wakeLock = null;
    const requestWakeLockHandler = async () => {
      try {
        wakeLock = await navigator.wakeLock.request("screen");
        wakeLock.addEventListener("release", () => {});
      } catch (e) {
        // error
      }
    };
    requestWakeLockHandler();
    const handleVisibilityChange = () => {
      if (wakeLock !== null && document.visibilityState === "visible") {
        requestWakeLockHandler();
      }
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);
    document.addEventListener("fullscreenchange", handleVisibilityChange);
    return () => {
      if (wakeLock != null) {
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
}
