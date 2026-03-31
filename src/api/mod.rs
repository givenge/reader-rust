pub mod router;
pub mod handlers;
pub mod auth;

use std::sync::Arc;
use crate::app::config::AppConfig;
use crate::service::{book_service::BookService, book_source_service::BookSourceService, user_service::UserService, book_group_service::BookGroupService};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub book_service: Arc<BookService>,
    pub book_source_service: Arc<BookSourceService>,
    pub user_service: Arc<UserService>,
    pub book_group_service: Arc<BookGroupService>,
}
