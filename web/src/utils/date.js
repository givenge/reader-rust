/**
 * 日期格式化工具函数
 */

/**
 * 格式化日期
 * @param {Date} date - 日期对象
 * @param {string} fmt - 格式字符串，如 "yyyy-MM-dd hh:mm:ss"
 * @returns {string} 格式化后的日期字符串
 */
export function formatDate(date, fmt) {
  var o = {
    "M+": date.getMonth() + 1, //月份
    "d+": date.getDate(), //日
    "h+": date.getHours(), //小时
    "m+": date.getMinutes(), //分
    "s+": date.getSeconds(), //秒
    "q+": Math.floor((date.getMonth() + 3) / 3), //季度
    S: date.getMilliseconds() //毫秒
  };
  if (/(y+)/.test(fmt)) {
    fmt = fmt.replace(
      RegExp.$1,
      (date.getFullYear() + "").substr(4 - RegExp.$1.length)
    );
  }
  for (var k in o) {
    if (new RegExp("(" + k + ")").test(fmt)) {
      fmt = fmt.replace(
        RegExp.$1,
        RegExp.$1.length == 1 ? o[k] : ("00" + o[k]).substr(("" + o[k]).length)
      );
    }
  }
  return fmt;
}

/**
 * 扩展Date原型，添加format方法
 * @deprecated 推荐使用formatDate函数
 */
if (!Date.prototype.format) {
  Date.prototype.format = function(fmt) {
    return formatDate(this, fmt);
  };
}
