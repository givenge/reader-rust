use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_json::Value;

use crate::model::rule::{SearchRule, ExploreRule, BookInfoRule, TocRule, ContentRule};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct BookSource {
    pub book_source_name: String,
    pub book_source_group: Option<String>,
    pub book_source_url: String,
    pub book_source_type: Option<i32>,
    pub book_url_pattern: Option<String>,
    pub custom_order: Option<i32>,
    pub enabled: Option<bool>,
    pub enabled_explore: Option<bool>,
    pub enabled_cookie_jar: Option<bool>,
    pub js_lib: Option<String>,
    pub concurrent_rate: Option<String>,
    pub header: Option<String>,
    pub login_url: Option<String>,
    pub login_ui: Option<String>,
    pub login_check_js: Option<String>,
    pub cover_decode_js: Option<String>,
    pub last_update_time: Option<i64>,
    pub weight: Option<i32>,
    pub explore_url: Option<String>,
    pub explore_screen: Option<String>,
    pub rule_explore: Option<ExploreRule>,
    pub search_url: Option<String>,
    pub rule_search: Option<SearchRule>,
    pub rule_book_info: Option<BookInfoRule>,
    pub rule_toc: Option<TocRule>,
    pub rule_content: Option<ContentRule>,
    pub rule_review: Option<Value>,
    pub book_source_comment: Option<String>,
    pub variable_comment: Option<String>,
    pub respond_time: Option<i64>,
    pub load_with_base_url: Option<bool>,
    pub single_url: Option<bool>,
}
