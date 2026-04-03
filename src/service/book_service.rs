use crate::crawler::{http_client::HttpClient, fetcher::{RequestSpec, HttpMethod, fetch}};
use crate::error::error::AppError;
use crate::model::{book::Book, book_chapter::BookChapter, book_source::BookSource, search::SearchBook};
use crate::parser::rule_engine::RuleEngine;
use crate::parser::js::{eval_js, eval_js_search_with_source};
use crate::storage::cache::file_cache::FileCache;
use tokio::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use urlencoding::encode;
use crate::util::hash::md5_hex;
use crate::util::text::{normalize_source_url, repair_encoded_url};

/// State for background chapter fetching
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChapterPagination {
    pub user_ns: String,
    pub source: BookSource,
    pub toc_url: String,
    pub visited_urls: Vec<String>,
    pub pending_urls: Vec<String>,
    pub seen_chapter_urls: Vec<String>,
    pub next_index: i32,
}

#[derive(Clone)]
pub struct BookService {
    http: HttpClient,
    parser: RuleEngine,
    cache: FileCache,
    storage_dir: PathBuf,
    source_cookies: Arc<RwLock<HashMap<String, String>>>,
}

impl BookService {
    pub fn new(http: HttpClient, parser: RuleEngine, cache: FileCache, storage_dir: &str) -> Self {
        let storage_dir = PathBuf::from(storage_dir);
        Self {
            http,
            parser,
            cache,
            storage_dir,
            source_cookies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn http_client(&self) -> &reqwest::Client {
        self.http.client()
    }

    fn source_cookie_key(&self, user_ns: &str, source_url: &str) -> String {
        format!("{}::{}", user_ns, normalize_source_url(source_url))
    }

    async fn apply_source_cookie(
        &self,
        user_ns: &str,
        source: &BookSource,
        headers: &mut Vec<(String, String)>,
    ) {
        let key = self.source_cookie_key(user_ns, &source.book_source_url);
        if let Some(cookie) = self.source_cookies.read().await.get(&key).cloned() {
            if !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("cookie")) {
                headers.push(("Cookie".to_string(), cookie));
            }
        }
    }

    pub async fn set_source_cookie(&self, user_ns: &str, source_url: &str, cookie: &str) {
        let cookie = cookie.trim();
        if cookie.is_empty() {
            return;
        }
        let key = self.source_cookie_key(user_ns, source_url);
        self.source_cookies.write().await.insert(key, cookie.to_string());
    }

    pub async fn clear_source_cookie(&self, user_ns: &str, source_url: &str) {
        let key = self.source_cookie_key(user_ns, source_url);
        self.source_cookies.write().await.remove(&key);
    }

    pub async fn search_book(&self, user_ns: &str, source: &BookSource, key: &str, page: i32) -> Result<Vec<SearchBook>, AppError> {
        let search_url = source.search_url.clone().ok_or_else(|| AppError::BadRequest("missing search_url".to_string()))?;
        tracing::info!("searching book from {}: key={}, page={}, url={}", source.book_source_name, key, page, search_url);
        let mut spec = analyze_url(&search_url, key, page, &source.book_source_url).map_err(|e| {
            tracing::error!("analyze_url failed: {:?}", e);
            e
        })?;

        // Merge global headers from source
        if let Some(header_str) = &source.header {
            if let Ok(headers) = serde_json::from_str::<std::collections::HashMap<String, String>>(header_str) {
                for (k, v) in headers {
                    spec.headers.push((k, v));
                }
            }
        }
        self.apply_source_cookie(user_ns, source, &mut spec.headers).await;

        tracing::debug!("search_book fetched spec: {:?}", spec);
        let res = fetch(&self.http, spec).await.map_err(|e| {
            tracing::error!("fetch failed: {:?}", e);
            e
        })?;
        tracing::debug!("fetch success, body length: {}", res.body.len());
        let books = self.parser.search_books(source, &res.body, &source.book_source_url);
        tracing::info!("found {} books", books.len());
        Ok(books)
    }

    pub async fn explore_book(&self, user_ns: &str, source: &BookSource, rule_find_url: &str, page: i32) -> Result<Vec<SearchBook>, AppError> {
        if rule_find_url.trim().is_empty() {
            return Err(AppError::BadRequest("ruleFindUrl required".to_string()));
        }
        let mut spec = analyze_url(rule_find_url, "", page, &source.book_source_url)?;

        if let Some(header_str) = &source.header {
            if let Ok(headers) = serde_json::from_str::<std::collections::HashMap<String, String>>(header_str) {
                for (k, v) in headers {
                    spec.headers.push((k, v));
                }
            }
        }
        self.apply_source_cookie(user_ns, source, &mut spec.headers).await;

        let res = fetch(&self.http, spec).await?;
        Ok(self.parser.explore_books(source, &res.body, &res.url))
    }

    pub async fn login_book_source(&self, source: &BookSource) -> Result<serde_json::Value, AppError> {
        let login_url = source
            .login_url
            .clone()
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| AppError::BadRequest("missing loginUrl".to_string()))?;

        let mut spec = analyze_url(&login_url, "", 1, &source.book_source_url)?;
        if let Some(header_str) = &source.header {
            if let Ok(headers) = serde_json::from_str::<std::collections::HashMap<String, String>>(header_str) {
                for (k, v) in headers {
                    spec.headers.push((k, v));
                }
            }
        }

        let res = fetch(&self.http, spec).await?;
        let check_result = if let Some(login_check_js) = source.login_check_js.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(eval_js(login_check_js, &res.body, &res.url).unwrap_or_default())
        } else {
            None
        };

        Ok(serde_json::json!({
            "success": true,
            "status": res.status,
            "url": res.url,
            "checkResult": check_result,
            "bodyPreview": res.body.chars().take(500).collect::<String>(),
            "bodyHtml": res.body
        }))
    }

    pub async fn get_book_info(&self, user_ns: &str, source: &BookSource, book_url: &str) -> Result<Book, AppError> {
        let mut headers = vec![];
        self.apply_source_cookie(user_ns, source, &mut headers).await;
        let res = fetch(&self.http, RequestSpec { url: book_url.to_string(), method: HttpMethod::GET, headers, body: None }).await?;
        Ok(self.parser.book_info(source, &res.body, &res.url, book_url))
    }

    pub async fn get_chapter_list(&self, user_ns: &str, source: &BookSource, toc_url: &str) -> Result<Vec<BookChapter>, AppError> {
        self.get_chapter_list_with_cache(user_ns, source, toc_url, false).await
    }

    pub async fn get_chapter_list_with_cache(&self, user_ns: &str, source: &BookSource, toc_url: &str, force_refresh: bool) -> Result<Vec<BookChapter>, AppError> {
        // Check cache first (unless force refresh)
        if !force_refresh {
            if let Ok(Some(cached)) = self.load_chapter_list_cache(user_ns, toc_url).await {
                if !cached.is_empty() {
                    return Ok(cached);
                }
            }
        }
        let (chapters, _) = self.get_chapter_list_with_pagination(source, toc_url).await?;
        // Save to cache
        let _ = self.save_chapter_list_cache(user_ns, toc_url, &chapters).await;
        Ok(chapters)
    }

    /// Get first page of chapters and pagination info for background fetching
    pub async fn get_chapter_list_first_page(&self, user_ns: &str, source: &BookSource, toc_url: &str) -> Result<(Vec<BookChapter>, ChapterPagination), AppError> {
        let mut headers = vec![];
        self.apply_source_cookie(user_ns, source, &mut headers).await;
        let res = fetch(&self.http, RequestSpec { url: toc_url.to_string(), method: HttpMethod::GET, headers, body: None }).await?;
        let (chapters, next_urls) = self.parser.chapter_list(source, &res.body, &res.url);

        let mut chapter_index = 0i32;
        let mut result = Vec::new();
        for mut ch in chapters {
            ch.index = chapter_index;
            chapter_index += 1;
            result.push(ch);
        }

        // The actual URL we fetched (after redirects) should be considered visited
        let actual_visited_url = res.url.clone();

        // Get chapter URLs from first page for deduplication
        let first_page_chapter_urls: std::collections::HashSet<String> = result.iter().map(|c| c.url.clone()).collect();

        // Filter out already visited URLs and the current page URL from pending_urls
        // Also filter out URLs that point to the same page (same path but different domain)
        let pending_urls: Vec<String> = next_urls.into_iter()
            .filter(|u| {
                // Filter out exact matches
                if u == &actual_visited_url || u == toc_url {
                    return false;
                }
                // Filter out URLs with the same path but different domain
                // This handles cases like m.22biqu.com vs m.22biqu.net
                if let (Ok(parsed_u), Ok(parsed_visited)) = (url::Url::parse(u), url::Url::parse(&actual_visited_url)) {
                    if parsed_u.path() == parsed_visited.path() {
                        return false;
                    }
                }
                true
            })
            .collect();

        let pagination = ChapterPagination {
            user_ns: user_ns.to_string(),
            source: source.clone(),
            toc_url: toc_url.to_string(),
            visited_urls: vec![toc_url.to_string(), actual_visited_url],
            pending_urls,
            seen_chapter_urls: first_page_chapter_urls.iter().cloned().collect(),
            next_index: chapter_index,
        };

        Ok((result, pagination))
    }

    /// Continue fetching remaining chapters from pagination state
    pub async fn fetch_remaining_chapters(&self, pagination: ChapterPagination) -> Result<Vec<BookChapter>, AppError> {
        let mut all_chapters = Vec::new();
        let mut visited_page_urls: std::collections::HashSet<String> = pagination.visited_urls.iter().cloned().collect();
        let mut seen_chapter_urls: std::collections::HashSet<String> = pagination.seen_chapter_urls.iter().cloned().collect();
        let mut chapter_index = pagination.next_index;

        let pending_urls: Vec<String> = pagination.pending_urls.into_iter()
            .filter(|u| !visited_page_urls.contains(u))
            .collect();

        if pending_urls.len() > 1 {
            // Multiple URLs from option dropdown - fetch all pages
            for url in pending_urls {
                if visited_page_urls.contains(&url) { continue; }
                visited_page_urls.insert(url.clone());

                let mut headers = vec![];
                self.apply_source_cookie(&pagination.user_ns, &pagination.source, &mut headers).await;
                let res = fetch(&self.http, RequestSpec { url: url.clone(), method: HttpMethod::GET, headers, body: None }).await?;
                let (chapters, _) = self.parser.chapter_list(&pagination.source, &res.body, &res.url);

                // Check if this page is a duplicate (all chapters already seen)
                // This handles cases where the first page URL differs from toc_url (e.g., different domain)
                let all_seen = chapters.iter().all(|ch| seen_chapter_urls.contains(&ch.url));
                if all_seen && !chapters.is_empty() {
                    tracing::debug!("Skipping duplicate page: {}", url);
                    continue;
                }

                for ch in chapters {
                    if seen_chapter_urls.contains(&ch.url) {
                        continue;
                    }
                    seen_chapter_urls.insert(ch.url.clone());

                    all_chapters.push(BookChapter {
                        title: ch.title,
                        url: ch.url,
                        index: chapter_index,
                    });
                    chapter_index += 1;
                }
            }
        } else if pending_urls.len() == 1 {
            // Single next page link - follow sequentially
            let mut current_url = pending_urls[0].clone();
            loop {
                if visited_page_urls.contains(&current_url) { break; }
                visited_page_urls.insert(current_url.clone());

                let mut headers = vec![];
                self.apply_source_cookie(&pagination.user_ns, &pagination.source, &mut headers).await;
                let res = fetch(&self.http, RequestSpec { url: current_url.clone(), method: HttpMethod::GET, headers, body: None }).await?;
                let (chapters, next_urls) = self.parser.chapter_list(&pagination.source, &res.body, &res.url);

                // Check if this page is a duplicate
                let all_seen = chapters.iter().all(|ch| seen_chapter_urls.contains(&ch.url));
                if all_seen && !chapters.is_empty() {
                    tracing::debug!("Skipping duplicate page: {}", current_url);
                    break; // Stop following pagination if we hit a duplicate page
                }

                for ch in chapters {
                    if seen_chapter_urls.contains(&ch.url) {
                        continue;
                    }
                    seen_chapter_urls.insert(ch.url.clone());

                    all_chapters.push(BookChapter {
                        title: ch.title,
                        url: ch.url,
                        index: chapter_index,
                    });
                    chapter_index += 1;
                }

                // Get next page
                let next = next_urls.into_iter().find(|u| !visited_page_urls.contains(u));
                match next {
                    Some(url) if !url.is_empty() => current_url = url,
                    _ => break,
                }
            }
        }

        Ok(all_chapters)
    }

    async fn get_chapter_list_with_pagination(&self, source: &BookSource, toc_url: &str) -> Result<(Vec<BookChapter>, Vec<String>), AppError> {
        let mut all_chapters = Vec::new();
        let mut visited_page_urls = std::collections::HashSet::new();
        let mut seen_chapter_urls = std::collections::HashSet::new();
        let mut chapter_index = 0i32;

        // Fetch first page
        let res = fetch(&self.http, RequestSpec { url: toc_url.to_string(), method: HttpMethod::GET, headers: vec![], body: None }).await?;
        let (chapters, next_urls) = self.parser.chapter_list(source, &res.body, &res.url);

        visited_page_urls.insert(toc_url.to_string());

        // Add first page chapters with deduplication
        for ch in chapters {
            if seen_chapter_urls.contains(&ch.url) {
                continue;
            }
            seen_chapter_urls.insert(ch.url.clone());
            all_chapters.push(BookChapter {
                title: ch.title,
                url: ch.url,
                index: chapter_index,
            });
            chapter_index += 1;
        }

        // Determine how to handle pagination
        // Filter out already visited URLs
        let pending_urls: Vec<String> = next_urls.into_iter()
            .filter(|u| !visited_page_urls.contains(u))
            .collect();

        if pending_urls.len() > 1 {
            // Multiple URLs from option dropdown - fetch all pages
            for url in pending_urls {
                if visited_page_urls.contains(&url) { continue; }
                visited_page_urls.insert(url.clone());

                let res = fetch(&self.http, RequestSpec { url: url.clone(), method: HttpMethod::GET, headers: vec![], body: None }).await?;
                let (chapters, _) = self.parser.chapter_list(source, &res.body, &res.url);

                for ch in chapters {
                    if seen_chapter_urls.contains(&ch.url) {
                        continue;
                    }
                    seen_chapter_urls.insert(ch.url.clone());
                    all_chapters.push(BookChapter {
                        title: ch.title,
                        url: ch.url,
                        index: chapter_index,
                    });
                    chapter_index += 1;
                }
            }
        } else if pending_urls.len() == 1 {
            // Single next page link - follow sequentially
            let mut current_url = pending_urls[0].clone();
            loop {
                if visited_page_urls.contains(&current_url) { break; }
                visited_page_urls.insert(current_url.clone());

                let res = fetch(&self.http, RequestSpec { url: current_url.clone(), method: HttpMethod::GET, headers: vec![], body: None }).await?;
                let (chapters, next_urls) = self.parser.chapter_list(source, &res.body, &res.url);

                for ch in chapters {
                    if seen_chapter_urls.contains(&ch.url) {
                        continue;
                    }
                    seen_chapter_urls.insert(ch.url.clone());
                    all_chapters.push(BookChapter {
                        title: ch.title,
                        url: ch.url,
                        index: chapter_index,
                    });
                    chapter_index += 1;
                }

                // Get next page
                let next = next_urls.into_iter().find(|u| !visited_page_urls.contains(u));
                match next {
                    Some(url) if !url.is_empty() => current_url = url,
                    _ => break,
                }
            }
        }

        Ok((all_chapters, visited_page_urls.into_iter().collect()))
    }

    pub async fn get_content(&self, user_ns: &str, book_url: &str, source: &BookSource, chapter_url: &str) -> Result<String, AppError> {
        let book_key = md5_hex(book_url);
        tracing::debug!("get_content called, chapter_url={}, book_key={}", chapter_url, book_key);
        if let Ok(Some(cached)) = self.cache.get(user_ns, &book_key, chapter_url).await {
            tracing::debug!("get_content returning cached content, len={}", cached.len());
            return Ok(cached);
        }
        tracing::debug!("get_content cache miss, fetching from network");

        let mut all_content = String::new();
        let mut visited_urls = std::collections::HashSet::new();
        let mut current_url = chapter_url.to_string();

        // Follow pagination to get all content pages
        loop {
            if visited_urls.contains(&current_url) {
                tracing::debug!("get_content detected loop, breaking");
                break;
            }
            visited_urls.insert(current_url.clone());

            tracing::debug!("get_content fetching: {}", current_url);
            let mut headers = vec![];
            self.apply_source_cookie(user_ns, source, &mut headers).await;
            let res = fetch(&self.http, RequestSpec { url: current_url.clone(), method: HttpMethod::GET, headers, body: None }).await?;
            tracing::debug!("get_content fetch done, body len={}", res.body.len());
            let content = self.parser.content(source, &res.body, &res.url);
            tracing::debug!("get_content parsed content len={}", content.len());

            if !content.is_empty() {
                if !all_content.is_empty() {
                    all_content.push('\n');
                }
                all_content.push_str(&content);
            }

            // Check for next page
            if let Some(next_url) = self.parser.next_content_url(source, &res.body, &res.url) {
                tracing::debug!("get_content found next_url: {}", next_url);
                current_url = next_url;
            } else {
                tracing::debug!("get_content no more pages");
                break;
            }
        }

        tracing::debug!("get_content final content len={}", all_content.len());
        if !all_content.is_empty() {
            let _ = self.cache.put(user_ns, &book_key, chapter_url, &all_content).await;
        }
        Ok(all_content)
    }

    /// Delete all chapter content cache for a book
    pub async fn delete_book_cache(&self, user_ns: &str, book_url: &str) -> Result<bool, AppError> {
        let book_key = md5_hex(book_url);
        self.cache.remove_book(user_ns, &book_key).await.map_err(|e| AppError::Internal(e.into()))
    }

    /// Check if a specific chapter is cached
    pub async fn is_chapter_cached(&self, user_ns: &str, book_url: &str, chapter_url: &str) -> bool {
        let book_key = md5_hex(book_url);
        self.cache.exists(user_ns, &book_key, chapter_url).await
    }

    pub async fn chapter_list_cache_exists(&self, user_ns: &str, toc_url: &str) -> bool {
        let path = self.chapter_list_cache_path(user_ns, toc_url);
        path.exists()
    }

    pub async fn get_bookshelf(&self, user_ns: &str) -> Result<Vec<Book>, AppError> {
        self.read_bookshelf(user_ns).await
    }

    pub async fn get_shelf_book(&self, user_ns: &str, book_url: &str) -> Result<Option<Book>, AppError> {
        let list = self.read_bookshelf(user_ns).await?;
        Ok(list.into_iter().find(|b| b.book_url == book_url))
    }

    /// Find book by chapter URL (chapter URL typically shares domain with book URL)
    pub async fn get_shelf_book_by_chapter(&self, user_ns: &str, chapter_url: &str) -> Result<Option<Book>, AppError> {
        let list = self.read_bookshelf(user_ns).await?;

        // Extract domain from chapter_url
        let chapter_domain = url::Url::parse(chapter_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()));

        for book in list {
            // Check if chapter URL starts with book URL (common pattern)
            if chapter_url.starts_with(&book.book_url) {
                return Ok(Some(book));
            }

            // Check if they share the same domain
            if let (Some(ref ch_domain), Ok(book_url_parsed)) = (&chapter_domain, url::Url::parse(&book.book_url)) {
                if let Some(book_domain) = book_url_parsed.host_str() {
                    if ch_domain == book_domain {
                        // Check if chapter URL path contains book URL path prefix
                        if let (Ok(ch_parsed), Ok(b_parsed)) = (url::Url::parse(chapter_url), url::Url::parse(&book.book_url)) {
                            let ch_path = ch_parsed.path();
                            let b_path = b_parsed.path();
                            // Check if paths share a common prefix (e.g., /biqu104/)
                            if ch_path.starts_with(b_path.trim_end_matches('/')) ||
                               b_path.trim_end_matches('/').starts_with(ch_path.trim_end_matches('/')) {
                                return Ok(Some(book));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Find book by name and author (for cases where book_url might differ)
    pub async fn find_shelf_book_by_name_author(&self, user_ns: &str, name: &str, author: &str) -> Result<Option<Book>, AppError> {
        let list = self.read_bookshelf(user_ns).await?;
        Ok(list.into_iter().find(|b| {
            b.name.trim() == name.trim() && b.author.trim() == author.trim()
        }))
    }

    pub async fn save_book(&self, user_ns: &str, mut book: Book) -> Result<Book, AppError> {
        sanitize_book_urls(&mut book);
        if book.origin.trim().is_empty() {
            return Err(AppError::BadRequest("missing origin".to_string()));
        }
        if book.book_url.trim().is_empty() {
            return Err(AppError::BadRequest("bookUrl required".to_string()));
        }

        let mut list = self.read_bookshelf(user_ns).await?;
        let mut exist_idx: Option<usize> = None;
        for (i, b) in list.iter().enumerate() {
            if b.book_url == book.book_url || (!b.name.is_empty() && b.name == book.name && b.author == book.author) {
                exist_idx = Some(i);
                break;
            }
        }

        if let Some(i) = exist_idx {
            let exist = list[i].clone();
            if book.dur_chapter_index.is_none() {
                book.dur_chapter_index = exist.dur_chapter_index;
            }
            if book.dur_chapter_title.is_none() {
                book.dur_chapter_title = exist.dur_chapter_title;
            }
            if book.dur_chapter_time.is_none() {
                book.dur_chapter_time = exist.dur_chapter_time;
            }
            if book.dur_chapter_pos.is_none() {
                book.dur_chapter_pos = exist.dur_chapter_pos;
            }
            if book.total_chapter_num.is_none() {
                book.total_chapter_num = exist.total_chapter_num;
            }
            if book.last_check_time.is_none() {
                book.last_check_time = exist.last_check_time;
            }
            if book.group.is_none() {
                book.group = exist.group;
            }
            list[i] = book.clone();
        } else {
            list.push(book.clone());
        }

        self.write_bookshelf(user_ns, &list).await?;
        Ok(book)
    }

    pub async fn save_books(&self, user_ns: &str, books: Vec<Book>) -> Result<Vec<Book>, AppError> {
        let mut normalized = Vec::with_capacity(books.len());
        for mut book in books {
            sanitize_book_urls(&mut book);
            if book.origin.trim().is_empty() {
                return Err(AppError::BadRequest("missing origin".to_string()));
            }
            if book.book_url.trim().is_empty() {
                return Err(AppError::BadRequest("bookUrl required".to_string()));
            }
            normalized.push(book);
        }
        self.write_bookshelf(user_ns, &normalized).await?;
        Ok(normalized)
    }

    pub async fn delete_book(&self, user_ns: &str, book: &Book) -> Result<bool, AppError> {
        let mut list = self.read_bookshelf(user_ns).await?;
        let orig_len = list.len();
        let removed: Vec<Book> = list.iter().filter(|b| {
            if !book.book_url.is_empty() && b.book_url == book.book_url {
                return true;
            }
            if !book.name.is_empty() && !book.author.is_empty() && b.name == book.name && b.author == book.author {
                return true;
            }
            false
        }).cloned().collect();
        list.retain(|b| {
            if !book.book_url.is_empty() && b.book_url == book.book_url {
                return false;
            }
            if !book.name.is_empty() && !book.author.is_empty() && b.name == book.name && b.author == book.author {
                return false;
            }
            true
        });
        let deleted = list.len() != orig_len;
        if deleted {
            self.write_bookshelf(user_ns, &list).await?;
            for removed_book in &removed {
                let _ = self.clear_book_related_cache(user_ns, removed_book).await;
            }
        }
        Ok(deleted)
    }

    pub async fn delete_books(&self, user_ns: &str, books: Vec<Book>) -> Result<usize, AppError> {
        let mut list = self.read_bookshelf(user_ns).await?;
        let mut deleted = 0usize;
        let mut removed_books: Vec<Book> = Vec::new();
        for book in books {
            let matched: Vec<Book> = list.iter().filter(|b| {
                if !book.book_url.is_empty() && b.book_url == book.book_url {
                    return true;
                }
                if !book.name.is_empty() && !book.author.is_empty() && b.name == book.name && b.author == book.author {
                    return true;
                }
                false
            }).cloned().collect();
            removed_books.extend(matched);
            let before = list.len();
            list.retain(|b| {
                if !book.book_url.is_empty() && b.book_url == book.book_url {
                    return false;
                }
                if !book.name.is_empty() && !book.author.is_empty() && b.name == book.name && b.author == book.author {
                    return false;
                }
                true
            });
            if list.len() != before {
                deleted += 1;
            }
        }
        if deleted > 0 {
            self.write_bookshelf(user_ns, &list).await?;
            for removed_book in &removed_books {
                let _ = self.clear_book_related_cache(user_ns, removed_book).await;
            }
        }
        Ok(deleted)
    }

    pub async fn cached_chapter_count(&self, user_ns: &str, book_url: &str, chapter_urls: &[String]) -> Result<usize, AppError> {
        let book_key = md5_hex(book_url);
        let mut count = 0usize;
        for url in chapter_urls {
            if self.cache.exists(user_ns, &book_key, url).await {
                count += 1;
            }
        }
        Ok(count)
    }

    pub async fn cache_chapter(&self, user_ns: &str, book_url: &str, source: &BookSource, chapter_url: &str, refresh: bool) -> Result<(), AppError> {
        let book_key = md5_hex(book_url);
        if refresh {
            let _ = self.cache.remove(user_ns, &book_key, chapter_url).await;
        }
        let _ = self.get_content(user_ns, book_url, source, chapter_url).await?;
        Ok(())
    }

    pub async fn get_cover(&self, user_ns: &str, url: &str) -> Result<(Vec<u8>, String), AppError> {
        let ext = file_ext_from_url(url).unwrap_or_else(|| "png".to_string());
        let name = md5_hex(url);
        let path = self.storage_dir.join("cache").join(user_ns).join("cover").join(format!("{}.{}", name, ext));
        if path.exists() {
            let data = fs::read(&path).await.map_err(|e| AppError::Internal(e.into()))?;
            let content_type = content_type_from_ext(&ext);
            return Ok((data, content_type));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| AppError::Internal(e.into()))?;
        }

        // Extract referer from URL for anti-hotlinking bypass
        let referer = url::Url::parse(url)
            .ok()
            .and_then(|u| {
                let scheme = u.scheme();
                let host = u.host_str()?;
                Some(format!("{}://{}", scheme, host))
            });

        let mut req = self.http.client().get(url);

        // Add necessary headers to bypass anti-hotlinking
        req = req
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8");

        if let Some(ref referer) = referer {
            req = req.header("Referer", referer);
        }

        let res = req.send().await.map_err(|e| AppError::Internal(e.into()))?;
        if !res.status().is_success() {
            return Err(AppError::NotFound("cover not found".to_string()));
        }
        let content_type = res.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| content_type_from_ext(&ext));
        let bytes = res.bytes().await.map_err(|e| AppError::Internal(e.into()))?.to_vec();
        let _ = fs::write(&path, &bytes).await;
        Ok((bytes, content_type))
    }

    pub async fn load_book_sources_cache(&self, user_ns: &str, book_url: &str) -> Result<Option<Vec<SearchBook>>, AppError> {
        let path = self.book_source_cache_path(user_ns, book_url);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path).await.map_err(|e| AppError::Internal(e.into()))?;
        let list: Vec<SearchBook> = serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(Some(list))
    }

    pub async fn save_book_sources_cache(&self, user_ns: &str, book_url: &str, list: &Vec<SearchBook>) -> Result<(), AppError> {
        let path = self.book_source_cache_path(user_ns, book_url);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| AppError::Internal(e.into()))?;
        }
        let data = serde_json::to_string(list).map_err(|e| AppError::BadRequest(e.to_string()))?;
        fs::write(&path, data).await.map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    pub async fn delete_book_sources_cache(&self, user_ns: &str, book_url: &str) -> Result<(), AppError> {
        let path = self.book_source_cache_path(user_ns, book_url);
        if path.exists() {
            fs::remove_file(&path).await.map_err(|e| AppError::Internal(e.into()))?;
        }
        Ok(())
    }

    fn book_source_cache_path(&self, user_ns: &str, book_url: &str) -> PathBuf {
        let name = md5_hex(book_url);
        self.storage_dir.join("data").join(user_ns).join("book_sources").join(format!("{}.json", name))
    }

    fn bookshelf_path(&self, user_ns: &str) -> PathBuf {
        self.storage_dir.join("data").join(user_ns).join("bookshelf.json")
    }

    async fn read_bookshelf(&self, user_ns: &str) -> Result<Vec<Book>, AppError> {
        let path = self.bookshelf_path(user_ns);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&path).await
            .map_err(|e| AppError::Internal(e.into()))?;
        let mut list: Vec<Book> = serde_json::from_str(&data)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        for book in &mut list {
            sanitize_book_urls(book);
        }
        Ok(list)
    }

    async fn write_bookshelf(&self, user_ns: &str, list: &Vec<Book>) -> Result<(), AppError> {
        let path = self.bookshelf_path(user_ns);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| AppError::Internal(e.into()))?;
        }
        let data = serde_json::to_string(list).map_err(|e| AppError::BadRequest(e.to_string()))?;
        fs::write(&path, data).await.map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    // Chapter list cache methods
    fn chapter_list_cache_path(&self, user_ns: &str, toc_url: &str) -> PathBuf {
        let name = md5_hex(toc_url);
        self.storage_dir.join("data").join(user_ns).join("chapters").join(format!("{}.json", name))
    }

    pub async fn load_chapter_list_cache(&self, user_ns: &str, toc_url: &str) -> Result<Option<Vec<BookChapter>>, AppError> {
        let path = self.chapter_list_cache_path(user_ns, toc_url);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path).await.map_err(|e| AppError::Internal(e.into()))?;
        let list: Vec<BookChapter> = serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(Some(list))
    }

    pub async fn save_chapter_list_cache(&self, user_ns: &str, toc_url: &str, chapters: &Vec<BookChapter>) -> Result<(), AppError> {
        let path = self.chapter_list_cache_path(user_ns, toc_url);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| AppError::Internal(e.into()))?;
        }
        let data = serde_json::to_string(chapters).map_err(|e| AppError::BadRequest(e.to_string()))?;
        fs::write(&path, data).await.map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    pub async fn append_chapter_list_cache(&self, user_ns: &str, toc_url: &str, new_chapters: &Vec<BookChapter>) -> Result<Vec<BookChapter>, AppError> {
        let mut existing = self.load_chapter_list_cache(user_ns, toc_url).await?.unwrap_or_default();
        let start_index = existing.len() as i32;
        for (i, ch) in new_chapters.iter().enumerate() {
            let mut ch = ch.clone();
            ch.index = start_index + i as i32;
            existing.push(ch);
        }
        self.save_chapter_list_cache(user_ns, toc_url, &existing).await?;
        Ok(existing)
    }

    pub async fn delete_chapter_list_cache(&self, user_ns: &str, toc_url: &str) -> Result<(), AppError> {
        let path = self.chapter_list_cache_path(user_ns, toc_url);
        if path.exists() {
            fs::remove_file(&path).await.map_err(|e| AppError::Internal(e.into()))?;
        }
        Ok(())
    }

    async fn clear_book_related_cache(&self, user_ns: &str, book: &Book) -> Result<(), AppError> {
        if !book.book_url.is_empty() {
            let _ = self.delete_book_cache(user_ns, &book.book_url).await;
            let _ = self.delete_book_sources_cache(user_ns, &book.book_url).await;
            let _ = self.delete_chapter_list_cache(user_ns, &book.book_url).await;
        }
        if let Some(toc_url) = &book.toc_url {
            if !toc_url.is_empty() {
                let _ = self.delete_chapter_list_cache(user_ns, toc_url).await;
            }
        }
        Ok(())
    }
}

fn sanitize_book_urls(book: &mut Book) {
    book.book_url = repair_encoded_url(&book.book_url);
    book.origin = normalize_source_url(&book.origin);
    if let Some(toc_url) = &book.toc_url {
        book.toc_url = Some(repair_encoded_url(toc_url));
    }
    if let Some(cover_url) = &book.cover_url {
        book.cover_url = Some(repair_encoded_url(cover_url));
    }
}

fn file_ext_from_url(url: &str) -> Option<String> {
    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);
    let pos = url.rfind('.')?;
    let ext = &url[pos + 1..];
    if ext.len() > 0 && ext.len() <= 8 {
        Some(ext.to_ascii_lowercase())
    } else {
        None
    }
}

fn content_type_from_ext(ext: &str) -> String {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn analyze_url(m_url: &str, key: &str, page: i32, base_url: &str) -> Result<RequestSpec, AppError> {
    tracing::debug!("analyzing url: {}", m_url);
    let mut rule_url = m_url.to_string();
    let base_url = normalize_source_url(base_url);

    // 1. Evaluate {{js}} blocks
    while let Some(start) = rule_url.find("{{") {
        if let Some(end) = rule_url[start..].find("}}") {
            let script = &rule_url[start + 2..start + end];
            tracing::debug!("evaluating search js: {}", script);
            let res = eval_js_search_with_source(script, key, page, &base_url).map_err(|e| {
                tracing::error!("js eval failed for search url: {:?}, script: {}", e, script);
                AppError::Internal(e)
            })?;
            rule_url.replace_range(start..start + end + 2, &res);
        } else {
            break;
        }
    }

    // 2. Simple replacements for compatibility
    let key_enc = encode(key);
    rule_url = rule_url.replace("{key}", &key_enc)
                       .replace("{page}", &page.to_string());

    // 3. Split comma options
    // Legacy legato format: url , { "method": "POST", "body": "..." }
    let parts: Vec<&str> = rule_url.splitn(2, ',').collect();
    let mut url = parts[0].trim().to_string();
    if !url.starts_with("http") {
        url = resolve_url(&base_url, &url);
    }

    let mut method = HttpMethod::GET;
    let mut headers = Vec::new();
    let mut body = None;

    if parts.len() > 1 {
        let options_str = parts[1].trim();
        if options_str.starts_with('{') {
            if let Ok(options) = serde_json::from_str::<serde_json::Value>(options_str) {
                if let Some(m) = options.get("method").and_then(|v| v.as_str()) {
                    if m.to_uppercase() == "POST" {
                        method = HttpMethod::POST;
                    }
                }
                if let Some(b) = options.get("body").and_then(|v| v.as_str()) {
                    body = Some(b.to_string());
                }
                if let Some(h) = options.get("headers").and_then(|v| v.as_object()) {
                    for (k, v) in h {
                        if let Some(vs) = v.as_str() {
                            headers.push((k.clone(), vs.to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(RequestSpec {
        url,
        method,
        headers,
        body,
    })
}

fn resolve_url(base: &str, url: &str) -> String {
    let base = normalize_source_url(base);
    let url = normalize_source_url(url);
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    if url.starts_with("//") {
        return format!("https:{}", url);
    }
    let base_parsed = url::Url::parse(&base).ok();
    if url.starts_with('/') {
        if let Some(mut b) = base_parsed {
            b.set_fragment(None);
            b.set_query(None);
            return format!("{}://{}{}", b.scheme(), b.host_str().unwrap_or(""), url);
        }
    }
    if let Some(mut b) = base_parsed {
        b.set_fragment(None);
        if let Ok(joined) = b.join(&url) {
            return joined.to_string();
        }
    }
    url.to_string()
}
