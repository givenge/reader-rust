use axum::{extract::State, Json};
use axum::extract::Multipart;
use serde::Deserialize;
use serde_json::Value;
use crate::api::AppState;
use crate::api::auth::AuthContext;

use crate::error::error::{ApiResponse, AppError};
use crate::model::rss::{RssSource, RssArticle};
use crate::util::time::now_ts;
use tokio::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct RemoteRssSourceParam {
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct RssArticlesRequest {
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    #[serde(rename = "sortName")]
    pub sort_name: Option<String>,
    #[serde(rename = "sortUrl")]
    pub sort_url: Option<String>,
    pub page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RssContentRequest {
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    pub link: Option<String>,
    pub origin: Option<String>,
}

pub async fn get_rss_sources(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let list = read_list::<RssSource>(&state, &user_ns, "rssSources.json").await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())))
}

pub async fn save_rss_source(State(state): State<AppState>, auth: AuthContext, Json(source): Json<RssSource>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    if source.source_url.is_empty() {
        return Err(AppError::BadRequest("RSS链接不能为空".to_string()));
    }
    if source.source_name.is_empty() {
        return Err(AppError::BadRequest("RSS名称不能为空".to_string()));
    }
    let mut list = read_list::<RssSource>(&state, &user_ns, "rssSources.json").await?;
    upsert_by_key(&mut list, source, |s| s.source_url.clone());
    write_list(&state, &user_ns, "rssSources.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn save_rss_sources(State(state): State<AppState>, auth: AuthContext, Json(mut sources): Json<Vec<RssSource>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<RssSource>(&state, &user_ns, "rssSources.json").await?;
    sources.retain(|s| !s.source_url.is_empty() && !s.source_name.is_empty());
    for s in sources {
        upsert_by_key(&mut list, s, |v| v.source_url.clone());
    }
    write_list(&state, &user_ns, "rssSources.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_rss_source(State(state): State<AppState>, auth: AuthContext, Json(source): Json<RssSource>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<RssSource>(&state, &user_ns, "rssSources.json").await?;
    list.retain(|s| s.source_url != source.source_url);
    write_list(&state, &user_ns, "rssSources.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn read_remote_rss_source_file(
    Json(param): Json<RemoteRssSourceParam>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| AppError::Internal(e.into()))?;

    let text = client.get(&param.url)
        .send().await
        .map_err(|e| AppError::BadRequest(format!("网络请求失败: {}", e)))?
        .text().await
        .map_err(|e| AppError::BadRequest(format!("读取响应失败: {}", e)))?;

    let sources: Vec<RssSource> = serde_json::from_str(&text)
        .or_else(|_| {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                extract_rss_sources(v)
            } else {
                Err(AppError::BadRequest("invalid rss sources json format".to_string()))
            }
        })?;

    let json_str = serde_json::to_string(&sources)
        .map_err(|e| AppError::BadRequest(format!("序列化RSS源失败: {}", e)))?;

    Ok(Json(ApiResponse::ok(vec![json_str])))
}

pub async fn read_rss_source_file(
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if let Some(file_name) = field.file_name() {
            if file_name.ends_with(".json") || file_name.ends_with(".txt") {
                let bytes = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                let text = String::from_utf8_lossy(&bytes);
                let sources: Vec<RssSource> = serde_json::from_str(&text)
                    .or_else(|_| {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                            extract_rss_sources(v)
                        } else {
                            Err(AppError::BadRequest("invalid rss sources json format".to_string()))
                        }
                    })?;
                return Ok(Json(serde_json::to_value(sources).unwrap_or_default()));
            }
        }
    }
    Err(AppError::BadRequest("No json file uploaded".to_string()))
}

pub async fn get_rss_articles(State(state): State<AppState>, auth: AuthContext, body: Option<Json<RssArticlesRequest>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let req = body.map(|b| b.0).unwrap_or(RssArticlesRequest { source_url: None, sort_name: None, sort_url: None, page: None });
    let source_url = req.source_url.ok_or_else(|| AppError::BadRequest("RSS源链接不能为空".to_string()))?;
    let sort_url = req.sort_url.unwrap_or_else(|| source_url.clone());
    let sort_name = req.sort_name.unwrap_or_default();
    let page = req.page.unwrap_or(1).max(1);

    let list = read_list::<RssSource>(&state, &user_ns, "rssSources.json").await?;
    let _rss_source = list.into_iter().find(|s| s.source_url == source_url).ok_or_else(|| AppError::BadRequest("RSS源不存在".to_string()))?;

    let res = state.book_service.http_client().get(&sort_url).send().await.map_err(|e| AppError::Internal(e.into()))?;
    let bytes = res.bytes().await.map_err(|e| AppError::Internal(e.into()))?;
    let feed = feed_rs::parser::parse(&bytes[..]).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut items = Vec::new();
    for entry in feed.entries {
        let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
        let link = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
        let description = entry.summary.as_ref().map(|s| s.content.clone());
        let content = entry.content.as_ref().and_then(|c| c.body.clone());
        let pub_date = entry.published.or(entry.updated).map(|d| d.to_rfc3339());
    let image = entry.media.first().and_then(|m| m.thumbnails.first()).map(|t| t.image.uri.clone());
        let order = entry.published.or(entry.updated).map(|d| d.timestamp()).unwrap_or(now_ts());
        items.push(RssArticle {
            origin: sort_url.clone(),
            sort: sort_name.clone(),
            title,
            order,
            link,
            pub_date,
            description,
            content,
            image,
            read: Some(false),
            variable: None,
        });
    }

    let page_size = 50usize;
    let start = ((page - 1) as usize) * page_size;
    let end = std::cmp::min(start + page_size, items.len());
    let page_items = if start < items.len() { items[start..end].to_vec() } else { Vec::new() };
    let data = serde_json::json!({"first": page_items, "second": null});
    Ok(Json(ApiResponse::ok(data)))
}

pub async fn get_rss_content(State(state): State<AppState>, auth: AuthContext, body: Option<Json<RssContentRequest>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let req = body.map(|b| b.0).unwrap_or(RssContentRequest { source_url: None, link: None, origin: None });
    let source_url = req.source_url.ok_or_else(|| AppError::BadRequest("RSS链接不能为空".to_string()))?;
    let link = req.link.ok_or_else(|| AppError::BadRequest("RSS文章链接不能为空".to_string()))?;
    let _origin = req.origin.ok_or_else(|| AppError::BadRequest("RSS文章来源不能为空".to_string()))?;

    let list = read_list::<RssSource>(&state, &user_ns, "rssSources.json").await?;
    let _rss_source = list.into_iter().find(|s| s.source_url == source_url).ok_or_else(|| AppError::BadRequest("RSS源不存在".to_string()))?;

    let res = state.book_service.http_client().get(&link).send().await.map_err(|e| AppError::Internal(e.into()))?;
    let body = res.text().await.map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(ApiResponse::ok(Value::String(body))))
}

async fn resolve_user_ns(state: &AppState, access_token: Option<&str>, secure_key: Option<&str>, user_ns: Option<&str>) -> Result<String, AppError> {
    match state.user_service.resolve_user_ns_with_override(access_token, secure_key, user_ns).await {
        Ok(ns) => Ok(ns),
        Err(_) => Err(AppError::BadRequest("NEED_LOGIN".to_string())),
    }
}

async fn read_list<T: for<'de> serde::Deserialize<'de>>(state: &AppState, user_ns: &str, name: &str) -> Result<Vec<T>, AppError> {
    let path = PathBuf::from(&state.config.storage_dir).join("data").join(user_ns).join(name);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path).await.map_err(|e| AppError::Internal(e.into()))?;
    let list = serde_json::from_str(&data).map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(list)
}

async fn write_list<T: serde::Serialize>(state: &AppState, user_ns: &str, name: &str, list: &Vec<T>) -> Result<(), AppError> {
    let dir = PathBuf::from(&state.config.storage_dir).join("data").join(user_ns);
    fs::create_dir_all(&dir).await.map_err(|e| AppError::Internal(e.into()))?;
    let path = dir.join(name);
    let data = serde_json::to_string(list).map_err(|e| AppError::BadRequest(e.to_string()))?;
    fs::write(path, data).await.map_err(|e| AppError::Internal(e.into()))?;
    Ok(())
}

fn extract_rss_sources(payload: serde_json::Value) -> Result<Vec<RssSource>, AppError> {
    if payload.is_array() {
        return serde_json::from_value::<Vec<RssSource>>(payload)
            .map_err(|e| AppError::BadRequest(e.to_string()));
    }
    if let Some(obj) = payload.as_object() {
        for key in ["rssSources", "rssSourceList", "data", "sources"] {
            if let Some(v) = obj.get(key) {
                if v.is_array() {
                    return serde_json::from_value::<Vec<RssSource>>(v.clone())
                        .map_err(|e| AppError::BadRequest(e.to_string()));
                }
            }
        }
    }
    Err(AppError::BadRequest("invalid rss sources payload".to_string()))
}

fn upsert_by_key<T, F>(list: &mut Vec<T>, item: T, key_fn: F)
where
    F: Fn(&T) -> String,
{
    let key = key_fn(&item);
    if let Some(pos) = list.iter().position(|v| key_fn(v) == key) {
        list[pos] = item;
    } else {
        list.push(item);
    }
}
