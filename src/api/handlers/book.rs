use axum::{extract::{State, Query}, Json};
use axum::body::Bytes;
use axum::response::{Response, IntoResponse, Sse};
use axum::response::sse::Event;
use axum::body::Body;
use axum::http::{StatusCode, header};
use serde::Deserialize;
use serde_json::Value;
use crate::api::AppState;
use crate::api::handlers::webdav::AccessTokenQuery;
use crate::error::error::{ApiResponse, AppError};
use crate::model::{book::Book, book_source::BookSource};
use crate::util::hash::md5_hex;
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;

#[derive(Debug, Deserialize)]
pub struct SearchBookRequest {
    key: Option<String>,
    page: Option<i32>,
    #[serde(rename = "bookSourceUrl")]
    book_source_url: Option<String>,
    #[serde(rename = "bookSource")]
    book_source: Option<BookSource>,
}

#[derive(Debug, Deserialize)]
pub struct SearchBookMultiRequest {
    key: Option<String>,
    page: Option<i32>,
    #[serde(rename = "bookSourceUrls")]
    book_source_urls: Option<Vec<String>>,
    #[serde(rename = "bookSourceGroup")]
    book_source_group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExploreBookRequest {
    #[serde(rename = "ruleFindUrl")]
    rule_find_url: Option<String>,
    page: Option<i32>,
    #[serde(rename = "bookSourceUrl")]
    book_source_url: Option<String>,
    #[serde(rename = "bookSource")]
    book_source: Option<BookSource>,
}
#[derive(Debug, Deserialize)]
pub struct BookInfoRequest {
    pub url: Option<String>,
    #[serde(rename = "bookSourceUrl", alias = "origin")]
    pub book_source_url: Option<String>,
    #[serde(rename = "bookSource")]
    pub book_source: Option<BookSource>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterListRequest {
    #[serde(rename = "tocUrl")]
    pub toc_url: Option<String>,
    #[serde(rename = "bookUrl", alias = "url")]
    pub book_url: Option<String>,
    #[serde(rename = "bookSourceUrl", alias = "origin")]
    pub book_source_url: Option<String>,
    #[serde(rename = "bookSource")]
    pub book_source: Option<BookSource>,
}

#[derive(Debug, Deserialize)]
pub struct BookContentRequest {
    #[serde(rename = "chapterUrl", alias = "url", alias = "href")]
    pub chapter_url: Option<String>,
    #[serde(rename = "bookSourceUrl", alias = "origin")]
    pub book_source_url: Option<String>,
    #[serde(rename = "bookSource")]
    pub book_source: Option<BookSource>,
    pub index: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteCacheRequest {
    #[serde(rename = "chapterUrl")]
    chapter_url: Option<String>,
    url: Option<String>,
    #[serde(rename = "bookUrl")]
    book_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveBookProgressRequest {
    url: Option<String>,
    #[serde(rename = "bookUrl")]
    book_url: Option<String>,
    index: Option<i32>,
    position: Option<i32>,
    #[serde(rename = "searchBook")]
    search_book: Option<SearchBookRef>,
}

#[derive(Debug, Deserialize)]
pub struct SearchBookRef {
    #[serde(rename = "bookUrl")]
    book_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetShelfBookRequest {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CoverQuery {
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CacheBookRequest {
    url: Option<String>,
    #[serde(rename = "bookUrl")]
    book_url: Option<String>,
    refresh: Option<i32>,
    #[serde(rename = "concurrentCount")]
    concurrent_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchBookMultiSseRequest {
    key: Option<String>,
    #[serde(rename = "bookSourceUrl")]
    book_source_url: Option<String>,
    #[serde(rename = "bookSourceGroup")]
    book_source_group: Option<String>,
    #[serde(rename = "lastIndex")]
    last_index: Option<i32>,
    #[serde(rename = "searchSize")]
    search_size: Option<i32>,
    #[serde(rename = "concurrentCount")]
    concurrent_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchBookSourceSseRequest {
    url: Option<String>,
    #[serde(rename = "bookSourceGroup")]
    book_source_group: Option<String>,
    #[serde(rename = "lastIndex")]
    last_index: Option<i32>,
    #[serde(rename = "searchSize")]
    search_size: Option<i32>,
    refresh: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BookSourceDebugRequest {
    #[serde(rename = "bookSourceUrl")]
    book_source_url: Option<String>,
    keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetAvailableBookSourceRequest {
    url: Option<String>,
    name: Option<String>,
    author: Option<String>,
    refresh: Option<i32>,
}

pub async fn search_book(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<SearchBookRequest>, body: axum::body::Bytes) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    println!("DEBUG: search_book handler reached: q={:?}, body_len={}", q, body.len());
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    
    let mut req = q;
    if !body.is_empty() {
        if let Ok(v) = serde_json::from_slice::<SearchBookRequest>(&body) {
            req = v;
        } else if let Ok(s) = std::str::from_utf8(&body) {
            for (k, v) in url::form_urlencoded::parse(s.as_bytes()) {
                match k.as_ref() {
                    "key" => req.key = Some(v.into_owned()),
                    "page" => req.page = v.parse::<i32>().ok(),
                    "bookSourceUrl" | "origin" => req.book_source_url = Some(v.into_owned()),
                    _ => {}
                }
            }
        }
    }
    
    let key = req.key.ok_or_else(|| AppError::BadRequest("key required".to_string()))?;
    let page = req.page.unwrap_or(1);
    let source = resolve_book_source(&state, &user_ns, req.book_source_url, req.book_source, None).await?;
    let books = state.book_service.search_book(&source, &key, page).await.map_err(|e| {
        tracing::error!("search_book failed: {:?}", e);
        e
    })?;
    println!("DEBUG: search_book found {} books. First book [0]: {{ name: {:?}, origin: {:?}, url: {:?} }}", 
        books.len(), books.first().map(|b| &b.name), books.first().map(|b| &b.origin), books.first().map(|b| &b.book_url));
    Ok(Json(ApiResponse::ok(serde_json::to_value(books).unwrap_or_default())))
}

pub async fn search_book_multi(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<SearchBookMultiRequest>, body: Option<Json<SearchBookMultiRequest>>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let req = if let Some(b) = body { b.0 } else { q };
    let key = req.key.ok_or_else(|| AppError::BadRequest("key required".to_string()))?;
    let page = req.page.unwrap_or(1);

    let sources = if let Some(urls) = req.book_source_urls {
        let mut out = Vec::new();
        for url in urls {
            if let Some(s) = state.book_source_service.get(&user_ns, &url).await? {
                out.push(s);
            }
        }
        out
    } else {
        let mut list = state.book_source_service.list(&user_ns).await?;
        if let Some(ref group) = req.book_source_group {
            list.retain(|s| s.book_source_group.as_deref().unwrap_or("").contains(group));
        }
        list
    };

    let mut tasks = Vec::new();
    for source in sources {
        let svc = state.book_service.clone();
        let k = key.clone();
        tasks.push(tokio::spawn(async move { svc.search_book(&source, &k, page).await }));
    }
    let mut results: Vec<crate::model::search::SearchBook> = Vec::new();
    for t in tasks {
        if let Ok(Ok(list)) = t.await {
            results.extend(list);
        }
    }

    // Merge books with same name and author
    let merged = merge_search_results(results);

    Ok(Json(ApiResponse::ok(serde_json::to_value(merged).unwrap_or_default())))
}

/// Merge search results from different book sources for the same book
fn merge_search_results(results: Vec<crate::model::search::SearchBook>) -> Vec<crate::model::search::SearchBook> {
    use std::collections::HashMap;
    use crate::model::search::SearchBook;

    let mut merged: HashMap<String, SearchBook> = HashMap::new();

    for book in results {
        let key = book.merge_key();

        if let Some(existing) = merged.get_mut(&key) {
            // Add this source to the existing book
            if let Some(ref mut urls) = existing.book_source_urls {
                if !urls.contains(&book.origin) {
                    urls.push(book.origin.clone());
                }
            } else {
                existing.book_source_urls = Some(vec![existing.origin.clone(), book.origin.clone()]);
            }

            // Fill in missing fields from this source
            if existing.cover_url.is_none() && book.cover_url.is_some() {
                existing.cover_url = book.cover_url;
            }
            if existing.intro.is_none() && book.intro.is_some() {
                existing.intro = book.intro;
            }
            if existing.kind.is_none() && book.kind.is_some() {
                existing.kind = book.kind;
            }
            if existing.last_chapter.is_none() && book.last_chapter.is_some() {
                existing.last_chapter = book.last_chapter;
            }
            if existing.update_time.is_none() && book.update_time.is_some() {
                existing.update_time = book.update_time;
            }
        } else {
            merged.insert(key, book);
        }
    }

    let mut result: Vec<SearchBook> = merged.into_values().collect();
    // Sort by name for consistent ordering
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

pub async fn explore_book(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<ExploreBookRequest>, body: Bytes) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let mut req = q;
    if !body.is_empty() {
        if let Ok(v) = serde_json::from_slice::<ExploreBookRequest>(&body) {
            req = v;
        } else if let Ok(s) = std::str::from_utf8(&body) {
            let mut rule_find_url: Option<String> = None;
            let mut page: Option<i32> = None;
            let mut book_source_url: Option<String> = None;
            for (k, v) in url::form_urlencoded::parse(s.as_bytes()) {
                match k.as_ref() {
                    "ruleFindUrl" => rule_find_url = Some(v.into_owned()),
                    "page" => page = v.parse::<i32>().ok(),
                    "bookSourceUrl" | "origin" => book_source_url = Some(v.into_owned()),
                    _ => {}
                }
            }
            if rule_find_url.is_some() || page.is_some() || book_source_url.is_some() {
                req.rule_find_url = rule_find_url.or(req.rule_find_url);
                req.page = page.or(req.page);
                req.book_source_url = book_source_url.or(req.book_source_url);
            }
        }
    }
    let rule_find_url = req.rule_find_url.ok_or_else(|| AppError::BadRequest("ruleFindUrl required".to_string()))?;
    let page = req.page.unwrap_or(1);
    let source = resolve_book_source(&state, &user_ns, req.book_source_url, req.book_source, Some(&rule_find_url)).await?;
    let list = state.book_service.explore_book(&source, &rule_find_url, page).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())))
}

pub async fn get_book_info(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<BookInfoRequest>, body: axum::body::Bytes) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let body_str = String::from_utf8_lossy(&body);
    println!("DEBUG: get_book_info handler reached: q={:?}, body={}", q, body_str);
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    
    let mut req = q;
    if !body.is_empty() {
        if let Ok(v) = serde_json::from_slice::<BookInfoRequest>(&body) {
            req = v;
        } else if let Ok(s) = std::str::from_utf8(&body) {
            for (k, v) in url::form_urlencoded::parse(s.as_bytes()) {
                match k.as_ref() {
                    "url" => req.url = Some(v.into_owned()),
                    "bookSourceUrl" | "origin" => req.book_source_url = Some(v.into_owned()),
                    _ => {}
                }
            }
        }
    }
    
    let url = req.url.ok_or_else(|| AppError::BadRequest("url required".to_string()))?;
    let source = resolve_book_source(&state, &user_ns, req.book_source_url, req.book_source, Some(&url)).await?;
    let book = state.book_service.get_book_info(&source, &url).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(book).unwrap_or_default())))
}

pub async fn get_chapter_list(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<ChapterListRequest>, body: axum::body::Bytes) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let body_str = String::from_utf8_lossy(&body);
    println!("DEBUG: get_chapter_list handler reached: q={:?}, body={}", q, body_str);
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;

    let mut req = q;
    if !body.is_empty() {
        if let Ok(v) = serde_json::from_slice::<ChapterListRequest>(&body) {
            req = v;
        } else if let Ok(s) = std::str::from_utf8(&body) {
            for (k, v) in url::form_urlencoded::parse(s.as_bytes()) {
                match k.as_ref() {
                    "tocUrl" => req.toc_url = Some(v.into_owned()),
                    "bookUrl" => req.book_url = Some(v.into_owned()),
                    "bookSourceUrl" | "origin" => req.book_source_url = Some(v.into_owned()),
                    _ => {}
                }
            }
        }
    }

    let source = resolve_book_source(&state, &user_ns, req.book_source_url.clone(), req.book_source.clone(), req.book_url.as_deref().or(req.toc_url.as_deref())).await?;
    let toc_url = if let Some(u) = req.toc_url {
        u
    } else if let Some(book_url) = req.book_url {
        let book = state.book_service.get_book_info(&source, &book_url).await?;
        book.toc_url.clone().unwrap_or(book_url)
    } else {
        return Err(AppError::BadRequest("tocUrl or bookUrl required".to_string()));
    };

    // Check if we have cached chapters
    if let Ok(Some(cached)) = state.book_service.load_chapter_list_cache(&user_ns, &toc_url).await {
        if !cached.is_empty() {
            return Ok(Json(ApiResponse::ok(serde_json::to_value(cached).unwrap_or_default())));
        }
    }

    // Get first page of chapters
    let (chapters, pagination) = state.book_service.get_chapter_list_first_page(&source, &toc_url).await?;

    // Save first page to cache immediately
    let _ = state.book_service.save_chapter_list_cache(&user_ns, &toc_url, &chapters).await;

    // If there are more pages to fetch, do it in background
    if !pagination.pending_urls.is_empty() {
        let state_clone = state.clone();
        let user_ns_clone = user_ns.clone();
        let toc_url_clone = toc_url.clone();

        tokio::spawn(async move {
            println!("DEBUG: Starting background fetch for remaining chapters");
            match state_clone.book_service.fetch_remaining_chapters(pagination).await {
                Ok(remaining) => {
                    if !remaining.is_empty() {
                        // Append to cache
                        match state_clone.book_service.append_chapter_list_cache(&user_ns_clone, &toc_url_clone, &remaining).await {
                            Ok(all_chapters) => {
                                println!("DEBUG: Background fetch complete, total chapters: {}", all_chapters.len());
                            }
                            Err(e) => {
                                println!("DEBUG: Failed to append chapters to cache: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("DEBUG: Background fetch failed: {:?}", e);
                }
            }
        });
    }

    Ok(Json(ApiResponse::ok(serde_json::to_value(chapters).unwrap_or_default())))
}

pub async fn get_book_content(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<BookContentRequest>, body: axum::body::Bytes) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    println!("DEBUG: get_book_content handler reached: q={:?}, body_len={}", q, body.len());
    if !body.is_empty() {
        println!("DEBUG: get_book_content body: {}", String::from_utf8_lossy(&body));
    }
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;

    let mut req = q;
    if !body.is_empty() {
        if let Ok(v) = serde_json::from_slice::<BookContentRequest>(&body) {
            // Merge with query params
            if req.chapter_url.is_none() { req.chapter_url = v.chapter_url; }
            if req.book_source_url.is_none() { req.book_source_url = v.book_source_url; }
            if req.book_source.is_none() { req.book_source = v.book_source; }
            if req.index.is_none() { req.index = v.index; }
        } else if let Ok(s) = std::str::from_utf8(&body) {
            for (k, v) in url::form_urlencoded::parse(s.as_bytes()) {
                match k.as_ref() {
                    "chapterUrl" | "href" => req.chapter_url = Some(v.into_owned()),
                    "bookSourceUrl" | "origin" => req.book_source_url = Some(v.into_owned()),
                    "index" => req.index = v.parse().ok(),
                    _ => {}
                }
            }
        }
    }

    // If we have url but no chapterUrl, treat url as bookUrl and use index
    let chapter_url = if let Some(url) = &req.chapter_url {
        // Check if url looks like a book URL (not a chapter URL) and we have an index
        if req.index.is_some() && !url.contains("/read/") && !url.contains("/chapter/") {
            // Need to get chapter from book URL + index
            let source = resolve_book_source(&state, &user_ns, req.book_source_url.clone(), req.book_source.clone(), Some(url)).await?;
            let book_info = state.book_service.get_book_info(&source, url).await?;
            let toc_url = book_info.toc_url.as_deref().unwrap_or(url);
            let chapters = state.book_service.get_chapter_list(&source, toc_url).await?;
            let idx = req.index.unwrap() as usize;
            if idx >= chapters.len() {
                return Err(AppError::BadRequest("chapter index out of range".to_string()));
            }
            chapters[idx].url.clone()
        } else {
            url.clone()
        }
    } else {
        return Err(AppError::BadRequest("chapterUrl required".to_string()));
    };

    println!("DEBUG: get_book_content resolved chapter_url: {}", chapter_url);
    let source = resolve_book_source(&state, &user_ns, req.book_source_url, req.book_source, Some(&chapter_url)).await?;
    let cache_key = format!("chapter:{}", md5_hex(&chapter_url));
    let content = state.book_service.get_content(&user_ns, &source, &chapter_url, &cache_key).await?;
    Ok(Json(ApiResponse::ok(serde_json::Value::String(content))))
}

pub async fn delete_book_cache(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<DeleteCacheRequest>, body: axum::body::Bytes) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;

    let mut req = q;
    if !body.is_empty() {
        if let Ok(v) = serde_json::from_slice::<DeleteCacheRequest>(&body) {
            // Merge with query params
            if req.chapter_url.is_none() { req.chapter_url = v.chapter_url; }
            if req.url.is_none() { req.url = v.url; }
            if req.book_url.is_none() { req.book_url = v.book_url; }
        } else if let Ok(s) = std::str::from_utf8(&body) {
            for (k, v) in url::form_urlencoded::parse(s.as_bytes()) {
                match k.as_ref() {
                    "chapterUrl" => req.chapter_url = Some(v.into_owned()),
                    "url" => req.url = Some(v.into_owned()),
                    "bookUrl" => req.book_url = Some(v.into_owned()),
                    _ => {}
                }
            }
        }
    }

    // Support chapterUrl, url, and bookUrl parameters
    let url = req.chapter_url.or(req.url).or(req.book_url)
        .ok_or_else(|| AppError::BadRequest("chapterUrl required".to_string()))?;

    let mut deleted_content = false;
    let mut deleted_chapter_list = false;

    // Try to delete chapter content cache
    let cache_key = format!("chapter:{}", md5_hex(&url));
    if state.book_service.cache_exists(&user_ns, &cache_key).await {
        state.book_service.delete_cache(&user_ns, &cache_key).await?;
        deleted_content = true;
    }

    // Try to delete chapter list cache (using URL as toc_url)
    if state.book_service.chapter_list_cache_exists(&user_ns, &url).await {
        state.book_service.delete_chapter_list_cache(&user_ns, &url).await?;
        deleted_chapter_list = true;
    }

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "deleted": true,
        "contentCache": deleted_content,
        "chapterListCache": deleted_chapter_list
    }))))
}

pub async fn get_bookshelf(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<AccessTokenQuery>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let list = state.book_service.get_bookshelf(&user_ns).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())))
}

pub async fn save_book(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<AccessTokenQuery>, Json(mut book): Json<Book>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    if book.book_url.trim().is_empty() {
        return Err(AppError::BadRequest("bookUrl required".to_string()));
    }
    if book.origin.trim().is_empty() {
        return Err(AppError::BadRequest("origin required".to_string()));
    }

    if book.toc_url.is_none() || book.name.trim().is_empty() {
        if let Some(source) = state.book_source_service.get(&user_ns, &book.origin).await? {
            if let Ok(info) = state.book_service.get_book_info(&source, &book.book_url).await {
                merge_book(&mut book, info);
            }
        }
    }

    let saved = state.book_service.save_book(&user_ns, book).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(saved).unwrap_or_default())))
}

pub async fn delete_book(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<AccessTokenQuery>, Json(book): Json<Book>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let deleted = state.book_service.delete_book(&user_ns, &book).await?;
    if !deleted {
        return Err(AppError::BadRequest("书架书籍不存在".to_string()));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!("删除书籍成功"))))
}

pub async fn delete_books(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<AccessTokenQuery>, Json(books): Json<Vec<Book>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let count = state.book_service.delete_books(&user_ns, books).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": count}))))
}

pub async fn save_book_progress(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<SaveBookProgressRequest>, body: Option<Json<SaveBookProgressRequest>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let req = if let Some(b) = body { b.0 } else { q };
    let book_url = req.url
        .or(req.book_url)
        .or(req.search_book.and_then(|s| s.book_url))
        .ok_or_else(|| AppError::BadRequest("url required".to_string()))?;
    let index = req.index.ok_or_else(|| AppError::BadRequest("index required".to_string()))?;

    let shelf_book = state.book_service.get_shelf_book(&user_ns, &book_url).await?
        .ok_or_else(|| AppError::BadRequest("书籍未加入书架".to_string()))?;

    let mut updated = shelf_book.clone();
    let mut chapter_title: Option<String> = None;
    if let (Some(toc_url), Ok(Some(source))) = (shelf_book.toc_url.clone(), state.book_source_service.get(&user_ns, &shelf_book.origin).await) {
        if let Ok(chapters) = state.book_service.get_chapter_list(&source, &toc_url).await {
            if index >= 0 && (index as usize) < chapters.len() {
                chapter_title = Some(chapters[index as usize].title.clone());
            }
            updated.total_chapter_num = Some(chapters.len() as i32);
            if let Some(last) = chapters.last() {
                updated.latest_chapter_title = Some(last.title.clone());
            }
        }
    }
    updated.dur_chapter_index = Some(index);
    updated.dur_chapter_time = Some(crate::util::time::now_ts());
    if let Some(title) = chapter_title {
        updated.dur_chapter_title = Some(title);
    }
    if let Some(pos) = req.position {
        updated.dur_chapter_pos = Some(pos);
    }

    let _ = state.book_service.save_book(&user_ns, updated).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!(""))))
}

pub async fn get_shelf_book(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<GetShelfBookRequest>, body: Option<Json<GetShelfBookRequest>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let req = if let Some(b) = body { b.0 } else { q };
    let url = req.url.ok_or_else(|| AppError::BadRequest("url required".to_string()))?;
    let book = state.book_service.get_shelf_book(&user_ns, &url).await?
        .ok_or_else(|| AppError::BadRequest("书籍不存在".to_string()))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(book).unwrap_or_default())))
}

pub async fn get_shelf_book_with_cache_info(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<AccessTokenQuery>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let books = state.book_service.get_bookshelf(&user_ns).await?;
    let mut result: Vec<Value> = Vec::with_capacity(books.len());
    for book in books {
        let mut cached_count = 0usize;
        if let (Some(toc_url), Ok(Some(source))) = (book.toc_url.clone(), state.book_source_service.get(&user_ns, &book.origin).await) {
            if let Ok(chapters) = state.book_service.get_chapter_list(&source, &toc_url).await {
                let urls: Vec<String> = chapters.into_iter().map(|c| c.url).collect();
                cached_count = state.book_service.cached_chapter_count(&user_ns, &urls).await.unwrap_or(0);
            }
        }
        let mut val = serde_json::to_value(&book).unwrap_or(serde_json::json!({}));
        if let Value::Object(ref mut map) = val {
            map.insert("cachedChapterCount".to_string(), serde_json::json!(cached_count));
        }
        result.push(val);
    }
    Ok(Json(ApiResponse::ok(serde_json::to_value(result).unwrap_or_default())))
}

pub async fn get_book_cover(State(state): State<AppState>, Query(q): Query<CoverQuery>) -> Result<Response, AppError> {
    let url = match q.path {
        Some(u) if !u.trim().is_empty() => u,
        _ => return Ok(StatusCode::NOT_FOUND.into_response()),
    };
    // Use "public" namespace for unauthenticated cover requests
    match state.book_service.get_cover("public", &url).await {
        Ok((bytes, content_type)) => {
            let mut resp = Response::new(Body::from(bytes));
            let headers = resp.headers_mut();
            headers.insert(header::CACHE_CONTROL, header::HeaderValue::from_static("86400"));
            if let Ok(v) = header::HeaderValue::from_str(&content_type) {
                headers.insert(header::CONTENT_TYPE, v);
            }
            Ok(resp)
        }
        Err(_) => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

pub async fn get_invalid_book_sources(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<AccessTokenQuery>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let path = std::path::PathBuf::from(&state.config.storage_dir)
        .join("cache")
        .join("invalid_book_sources.json");
    if !path.exists() {
        return Ok(Json(ApiResponse::ok(serde_json::json!([]))));
    }
    let data = tokio::fs::read_to_string(path).await.map_err(|e| AppError::Internal(e.into()))?;
    let val: Value = serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;
    if let Value::Array(_) = val {
        Ok(Json(ApiResponse::ok(val)))
    } else {
        Ok(Json(ApiResponse::ok(serde_json::json!([val]))))
    }
}

pub async fn cache_book_sse(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<CacheBookRequest>, body: Option<Json<CacheBookRequest>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let req = if let Some(b) = body { b.0 } else { q };
    let book_url = req.url.or(req.book_url).ok_or_else(|| AppError::BadRequest("url required".to_string()))?;
    let refresh = req.refresh.unwrap_or(0) > 0;
    let concurrent = req.concurrent_count.unwrap_or(24).max(1) as usize;

    let book = state.book_service.get_shelf_book(&user_ns, &book_url).await?
        .ok_or_else(|| AppError::BadRequest("请先加入书架".to_string()))?;
    if book.origin.trim().is_empty() {
        return Err(AppError::BadRequest("未配置书源".to_string()));
    }
    let source = state.book_source_service.get(&user_ns, &book.origin).await?
        .ok_or_else(|| AppError::BadRequest("未配置书源".to_string()))?;
    let toc_url = book.toc_url.clone().ok_or_else(|| AppError::BadRequest("tocUrl missing".to_string()))?;

    let chapters = state.book_service.get_chapter_list(&source, &toc_url).await?;
    let mut cached_count = 0usize;
    if !refresh {
        for ch in &chapters {
            if state.book_service.is_chapter_cached(&user_ns, &ch.url).await {
                cached_count += 1;
            }
        }
    }

    let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrent));
    let mut tasks = Vec::new();
    for ch in chapters {
        let already_cached = !refresh && state.book_service.is_chapter_cached(&user_ns, &ch.url).await;
        if already_cached {
            continue;
        }
        let permit = sem.clone().acquire_owned().await.map_err(|_| AppError::Internal(anyhow::anyhow!("semaphore closed")))?;
        let svc = state.book_service.clone();
        let src = source.clone();
        let url = ch.url.clone();
        let refresh_flag = refresh;
        let u_ns = user_ns.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            svc.cache_chapter(&u_ns, &src, &url, refresh_flag).await
        }));
    }
    let mut success = 0usize;
    let mut failed = 0usize;
    for t in tasks {
        match t.await {
            Ok(Ok(_)) => success += 1,
            _ => failed += 1,
        }
    }
    cached_count += success;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "cachedCount": cached_count,
        "successCount": success,
        "failedCount": failed
    }))))
}

pub async fn search_book_multi_sse(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<SearchBookMultiSseRequest>) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let key = q.key.unwrap_or_default();
    let last_index = q.last_index.unwrap_or(-1);
    let search_size = q.search_size.unwrap_or(50).max(1) as usize;
    let concurrent = q.concurrent_count.unwrap_or(24).max(1) as usize;
    let book_source_url = q.book_source_url.clone().and_then(|u| if u.trim().is_empty() { None } else { Some(u) });

    let (tx, rx) = mpsc::channel::<Event>(16);
    let state_clone = state.clone();

    tokio::spawn(async move {
        if key.trim().is_empty() {
            let _ = tx.send(Event::default().event("error").data(json_err("请输入搜索关键字"))).await;
            let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
            return;
        }

        let sources = if let Some(url) = book_source_url {
            match state_clone.book_source_service.get(&user_ns, &url).await {
                Ok(Some(s)) => vec![s],
                _ => {
                    let _ = tx.send(Event::default().event("error").data(json_err("未配置书源"))).await;
                    let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
                    return;
                }
            }
        } else {
            match state_clone.book_source_service.list(&user_ns).await {
                Ok(mut list) => {
                    if let Some(ref group) = q.book_source_group {
                        list.retain(|s| s.book_source_group.as_deref().unwrap_or("").contains(group));
                    }
                    if list.is_empty() {
                        let _ = tx.send(Event::default().event("error").data(json_err("未配置书源或分组为空"))).await;
                        let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
                        return;
                    }
                    list
                }
                _ => {
                    let _ = tx.send(Event::default().event("error").data(json_err("未配置书源"))).await;
                    let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
                    return;
                }
            }
        };

        let mut idx = last_index + 1;
        let mut last_idx = last_index;
        let mut result_map = std::collections::HashSet::<String>::new();
        let mut total = 0usize;
        let mut tasks: FuturesUnordered<_> = FuturesUnordered::new();
        let mut stop_adding = false;

        while (idx as usize) < sources.len() || !tasks.is_empty() {
            // Only add new tasks if we haven't reached search_size yet
            if !stop_adding && tasks.len() < concurrent && (idx as usize) < sources.len() {
                let source = sources[idx as usize].clone();
                let svc = state_clone.book_service.clone();
                let k = key.clone();
                let cur_idx = idx;
                tasks.push(tokio::spawn(async move {
                    let res = svc.search_book(&source, &k, 1).await;
                    (cur_idx, source.book_source_name, res)
                }));
                idx += 1;
                continue;
            }

            if let Some(res) = tasks.next().await {
                match res {
                    Ok((cur_idx, source_name, Ok(list))) => {
                        last_idx = cur_idx;
                        let mut batch = Vec::new();
                        for b in list {
                            let key = format!("{}_{}", b.name, b.author);
                            if !result_map.contains(&key) {
                                result_map.insert(key);
                                batch.push(b);
                            }
                        }
                        if !batch.is_empty() {
                            total += batch.len();
                            let payload = serde_json::json!({"lastIndex": cur_idx, "data": batch});
                            let _ = tx.send(Event::default().data(payload.to_string())).await;
                        }
                        // Stop adding new tasks when search_size is reached
                        if total >= search_size {
                            stop_adding = true;
                        }
                    }
                    Ok((cur_idx, _source_name, Err(e))) => {
                        last_idx = cur_idx;
                        tracing::error!("search_book error from {}: {:?}", _source_name, e);
                    }
                    Err(e) => {
                        tracing::error!("task join error: {:?}", e);
                    }
                }
            } else {
                break;
            }
        }

        let _ = tx.send(Event::default().event("end").data(json_end(last_idx))).await;
    });

    Ok(Sse::new(ReceiverStream::new(rx).map(Ok)))
}

pub async fn search_book_source_sse(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<SearchBookSourceSseRequest>) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let book_url = q.url.unwrap_or_default();
    let last_index = q.last_index.unwrap_or(-1);
    let search_size = q.search_size.unwrap_or(30).max(1) as usize;
    let refresh = q.refresh.unwrap_or(0) > 0;
    let concurrent = std::cmp::max(search_size * 2, 24);

    let (tx, rx) = mpsc::channel::<Event>(16);
    let state_clone = state.clone();

    tokio::spawn(async move {
        if book_url.trim().is_empty() {
            let _ = tx.send(Event::default().event("error").data(json_err("请输入书籍链接"))).await;
            let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
            return;
        }

        let book = match state_clone.book_service.get_shelf_book(&user_ns, &book_url).await {
            Ok(Some(b)) => b,
            _ => {
                let _ = tx.send(Event::default().event("error").data(json_err("书籍信息错误"))).await;
                let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
                return;
            }
        };

        let sources = match state_clone.book_source_service.list(&user_ns).await {
            Ok(mut list) => {
                if let Some(ref group) = q.book_source_group {
                    list.retain(|s| s.book_source_group.as_deref().unwrap_or("").contains(group));
                }
                if list.is_empty() {
                    let _ = tx.send(Event::default().event("error").data(json_err("未配置书源或分组为空"))).await;
                    let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
                    return;
                }
                list
            }
            _ => {
                let _ = tx.send(Event::default().event("error").data(json_err("未配置书源"))).await;
                let _ = tx.send(Event::default().event("end").data(json_end(last_index))).await;
                return;
            }
        };

        let mut idx = last_index + 1;
        let mut last_idx = last_index;
        let mut total = 0usize;
        let mut tasks: FuturesUnordered<_> = FuturesUnordered::new();
        let mut all_results: Vec<crate::model::search::SearchBook> = Vec::new();

        while (idx as usize) < sources.len() || !tasks.is_empty() {
            while tasks.len() < concurrent && (idx as usize) < sources.len() {
                let source = sources[idx as usize].clone();
                let svc = state_clone.book_service.clone();
                let target_name = book.name.clone();
                let target_author = book.author.clone();
                let cur_idx = idx;
                tasks.push(tokio::spawn(async move {
                    let res = svc.search_book(&source, &target_name, 1).await;
                    (cur_idx, res, target_name, target_author)
                }));
                last_idx = idx;
                idx += 1;
            }

            if let Some(res) = tasks.next().await {
                if let Ok((cur_idx, Ok(list), target_name, target_author)) = res {
                    let mut batch = Vec::new();
                    for b in list {
                        if b.name == target_name && b.author == target_author {
                            batch.push(b);
                        }
                    }
                    if !batch.is_empty() {
                        total += batch.len();
                        all_results.extend(batch.clone());
                        let payload = serde_json::json!({"lastIndex": cur_idx, "data": batch});
                        let _ = tx.send(Event::default().data(payload.to_string())).await;
                    }
                    if total >= search_size {
                        break;
                    }
                }
            } else {
                break;
            }
        }

        if refresh || !all_results.is_empty() {
            let _ = state_clone.book_service.save_book_sources_cache(&user_ns, &book.book_url, &all_results).await;
        }
        let _ = tx.send(Event::default().event("end").data(json_end(last_idx))).await;
    });

    Ok(Sse::new(ReceiverStream::new(rx).map(Ok)))
}

pub async fn get_available_book_source(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<GetAvailableBookSourceRequest>, body: Option<Json<GetAvailableBookSourceRequest>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let req = if let Some(b) = body { b.0 } else { q };
    let refresh = req.refresh.unwrap_or(0) > 0;

    // Try to find book by URL first, then by name+author
    let book_url = req.url.clone();

    if !refresh {
        if let Some(ref url) = book_url {
            if let Some(list) = state.book_service.load_book_sources_cache(&user_ns, url).await? {
                return Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())));
            }
        }
    }

    // Find book on shelf - try URL first, then name+author
    let book = if let Some(ref url) = book_url {
        state.book_service.get_shelf_book(&user_ns, url).await?
    } else {
        None
    };

    // If not found by URL, try name+author
    let book = match book {
        Some(b) => Some(b),
        None => {
            if let (Some(name), Some(author)) = (&req.name, &req.author) {
                state.book_service.find_shelf_book_by_name_author(&user_ns, name, author).await?
            } else {
                None
            }
        }
    };

    let book = book.ok_or_else(|| AppError::BadRequest("书籍信息错误".to_string()))?;
    let sources = state.book_source_service.list(&user_ns).await?;
    if sources.is_empty() {
        return Ok(Json(ApiResponse::ok(serde_json::json!([]))));
    }

    let mut tasks: FuturesUnordered<_> = FuturesUnordered::new();
    for source in sources {
        let svc = state.book_service.clone();
        let name = book.name.clone();
        let author = book.author.clone();
        tasks.push(tokio::spawn(async move {
            let res = svc.search_book(&source, &name, 1).await;
            (res, name, author)
        }));
    }
    let mut result: Vec<crate::model::search::SearchBook> = Vec::new();
    while let Some(res) = tasks.next().await {
        if let Ok((Ok(list), name, author)) = res {
            for b in list {
                if b.name == name && b.author == author {
                    result.push(b);
                }
            }
        }
    }
    let _ = state.book_service.save_book_sources_cache(&user_ns, &book.book_url, &result).await;
    Ok(Json(ApiResponse::ok(serde_json::to_value(result).unwrap_or_default())))
}

pub async fn book_source_debug_sse(State(state): State<AppState>, Query(access_q): Query<AccessTokenQuery>, Query(q): Query<BookSourceDebugRequest>) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns(access_q.access_token.as_deref(), access_q.secure_key.as_deref()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let book_source_url = q.book_source_url.unwrap_or_default();
    let keyword = q.keyword.unwrap_or_default();

    let (tx, rx) = mpsc::channel::<Event>(16);
    let state_clone = state.clone();

    tokio::spawn(async move {
        if book_source_url.trim().is_empty() {
            let _ = tx.send(Event::default().event("error").data(json_err("未配置书源"))).await;
            let _ = tx.send(Event::default().event("end").data(json_end(0))).await;
            return;
        }
        if keyword.trim().is_empty() {
            let _ = tx.send(Event::default().event("error").data(json_err("请输入搜索关键词"))).await;
            let _ = tx.send(Event::default().event("end").data(json_end(0))).await;
            return;
        }
        let source = match state_clone.book_source_service.get(&user_ns, &book_source_url).await {
            Ok(Some(s)) => s,
            _ => {
                let _ = tx.send(Event::default().event("error").data(json_err("未配置书源"))).await;
                let _ = tx.send(Event::default().event("end").data(json_end(0))).await;
                return;
            }
        };
        let _ = tx.send(Event::default().data(json_msg("start search"))).await;
        match state_clone.book_service.search_book(&source, &keyword, 1).await {
            Ok(list) => {
                let msg = format!("found {} items", list.len());
                let _ = tx.send(Event::default().data(json_msg(&msg))).await;
                let payload = serde_json::json!({"data": list});
                let _ = tx.send(Event::default().data(payload.to_string())).await;
            }
            Err(e) => {
                let _ = tx.send(Event::default().event("error").data(json_err(&e.to_string()))).await;
            }
        }
        let _ = tx.send(Event::default().event("end").data(json_end(0))).await;
    });

    Ok(Sse::new(ReceiverStream::new(rx).map(Ok)))
}

fn json_err(msg: &str) -> String {
    serde_json::json!({"errorMsg": msg}).to_string()
}

fn json_end(last_index: i32) -> String {
    serde_json::json!({"lastIndex": last_index}).to_string()
}

fn json_msg(msg: &str) -> String {
    serde_json::json!({"msg": msg}).to_string()
}


async fn resolve_book_source(state: &AppState, user_ns: &str, book_source_url: Option<String>, book_source: Option<BookSource>, book_url: Option<&str>) -> Result<BookSource, AppError> {
    println!("DEBUG: resolve_book_source: user_ns={}, url={:?}, src_is_some={}, book_url={:?}", user_ns, book_source_url, book_source.is_some(), book_url);
    if let Some(src) = book_source {
        return Ok(src);
    }
    if let Some(url) = book_source_url {
        if !url.trim().is_empty() {
            return state.book_source_service.get(&user_ns, &url).await?
                .ok_or_else(|| {
                    println!("ERROR: bookSource not found for url: {}", url);
                    AppError::NotFound("bookSource not found".to_string())
                });
        }
    }
    
    // Auto-discovery from book_url
    if let Some(b_url) = book_url {
        let b_host = match url::Url::parse(b_url) {
            Ok(u) => u.host_str().unwrap_or_default().to_string(),
            Err(_) => "".to_string(),
        };
        if !b_host.is_empty() {
            // Extract root domain for comparison (e.g., "22biqu" from "m.22biqu.com")
            let b_root = extract_root_domain(&b_host);
            let sources = state.book_source_service.list(&user_ns).await?;
            for s in sources {
                if let Ok(s_url) = url::Url::parse(&s.book_source_url) {
                    if let Some(s_host) = s_url.host_str() {
                        // Match by exact host or by root domain
                        let s_root = extract_root_domain(s_host);
                        if b_host.ends_with(s_host) || s_host.ends_with(&b_host) || (b_root == s_root && !b_root.is_empty()) {
                            println!("DEBUG: auto-discovered book source: {} for host: {}", s.book_source_url, b_host);
                            return Ok(s);
                        }
                    }
                }
            }
        }
    }

    Err(AppError::BadRequest("bookSource or bookSourceUrl required, and auto-discovery failed".to_string()))
}

/// Extract root domain for matching (e.g., "22biqu" from "m.22biqu.com" or "m.22biqu.net")
fn extract_root_domain(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        parts[parts.len() - 2].to_string()
    } else {
        host.to_string()
    }
}

fn merge_book(target: &mut Book, info: Book) {
    if target.name.trim().is_empty() {
        target.name = info.name;
    }
    if target.author.trim().is_empty() {
        target.author = info.author;
    }
    if target.cover_url.is_none() {
        target.cover_url = info.cover_url;
    }
    if target.toc_url.is_none() {
        target.toc_url = info.toc_url;
    }
    if target.intro.is_none() {
        target.intro = info.intro;
    }
    if target.latest_chapter_title.is_none() {
        target.latest_chapter_title = info.latest_chapter_title;
    }
    if target.word_count.is_none() {
        target.word_count = info.word_count;
    }
    if target.origin_name.is_none() {
        target.origin_name = info.origin_name;
    }
    if target.kind.is_none() {
        target.kind = info.kind;
    }
    if target.update_time.is_none() {
        target.update_time = info.update_time;
    }
}

pub async fn get_txt_toc_rules() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    Json(ApiResponse::ok(vec![]))
}
