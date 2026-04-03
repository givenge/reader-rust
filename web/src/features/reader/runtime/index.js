/**
 * 阅读器运行时混合器
 * 整合所有运行时模块为一个统一的混合器
 */

import readerSession from "./readerSession";
import readerProgress from "./readerProgress";
import readerPagination from "./readerPagination";
import readerSpeech from "./readerSpeech";
import readerEnvironment from "./readerEnvironment";
import readerContent from "./readerContent";
import readerPrefetch from "./readerPrefetch";
import readerClick from "./readerClick";
import readerSelection from "./readerSelection";
import readerFilter from "./readerFilter";
import readerUI from "./readerUI";
import readerAutoRead from "./readerAutoRead";
import readerBookmark from "./readerBookmark";

export default {
  ...readerSession,
  ...readerProgress,
  ...readerPagination,
  ...readerSpeech,
  ...readerEnvironment,
  ...readerContent,
  ...readerPrefetch,
  ...readerClick,
  ...readerSelection,
  ...readerFilter,
  ...readerUI,
  ...readerAutoRead,
  ...readerBookmark
};
