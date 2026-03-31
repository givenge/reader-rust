mod book;
mod book_source;
mod user;
mod rss;
mod bookmark;
mod replace_rule;
mod webdav;
mod book_group;

pub use book::*;
pub use book_source::*;
pub use user::{
    add_user,
    delete_file,
    delete_users,
    get_user_config,
    get_user_info,
    get_user_list,
    login,
    logout,
    reset_password,
    save_user_config,
    update_user,
    upload_file,
};
pub use rss::{
    delete_rss_source,
    get_rss_articles,
    get_rss_content,
    get_rss_sources,
    save_rss_source,
    save_rss_sources,
};
pub use bookmark::{
    delete_bookmark,
    delete_bookmarks,
    get_bookmarks,
    save_bookmark,
    save_bookmarks,
};
pub use replace_rule::{
    delete_replace_rule,
    delete_replace_rules,
    get_replace_rules,
    save_replace_rule,
    save_replace_rules,
};
pub use webdav::{
    delete_webdav_file,
    delete_webdav_file_list,
    get_webdav_file,
    get_webdav_file_list,
    upload_file_to_webdav,
    webdav_handler,
};
pub use book_group::*;

use axum::response::IntoResponse;
use axum::Json;
use crate::error::error::ApiResponse;

pub async fn health() -> impl IntoResponse {
    Json(ApiResponse::ok("ok"))
}
