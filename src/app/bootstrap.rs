use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use tracing_subscriber::EnvFilter;

use crate::app::config;
use crate::api::{self, AppState};
use crate::storage::{db, fs::storage_fs::StorageFs, cache::file_cache::FileCache};
use crate::service::{book_service::BookService, book_source_service::BookSourceService, user_service::UserService, book_group_service::BookGroupService};
use crate::crawler::http_client::HttpClient;
use crate::parser::rule_engine::RuleEngine;

pub async fn run() -> anyhow::Result<()> {
    println!("DEBUG: starting bootstrap::run");
    let cfg = config::load()?;
    println!("DEBUG: config loaded: addr={}:{}", cfg.server_host, cfg.server_port);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(cfg.log_level.clone()))
        .init();

    let storage_fs = StorageFs::new(&cfg.storage_dir, &cfg.assets_dir);
    storage_fs.ensure().await?;

    let pool = db::init_pool(&cfg.database_url).await?;
    println!("DEBUG: db pool initialized");
    let repo = db::repo::BookSourceRepo::new(pool.clone());

    let http = HttpClient::new(cfg.request_timeout_secs, None)?;
    println!("DEBUG: http client created");
    let parser = RuleEngine::new()?;
    println!("DEBUG: rule engine created");
    let cache = FileCache::new(format!("{}/cache", cfg.storage_dir));

    let book_service = Arc::new(BookService::new(http, parser, cache, &cfg.storage_dir));
    let book_source_service = Arc::new(BookSourceService::new(repo, &cfg.storage_dir));
    let user_service = Arc::new(UserService::new(cfg.clone()));
    let book_group_service = Arc::new(BookGroupService::new(&cfg.storage_dir));

    let state = AppState {
        config: cfg.clone(),
        book_service,
        book_source_service,
        user_service,
        book_group_service,
    };

    let app: Router = api::router::build_router(state);

    let addr = SocketAddr::new(cfg.server_host.parse()?, cfg.server_port);
    tracing::info!("listening on {}", addr);
    println!("DEBUG: starting axum::serve on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}
