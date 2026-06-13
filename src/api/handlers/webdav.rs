use crate::api::auth::AuthContext;
use crate::api::AppState;
use crate::error::error::{ApiResponse, AppError};
use crate::util::time::now_ts;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    Json,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct WebdavPathRequest {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebdavDeleteListRequest {
    pub path: Option<Vec<String>>,
}

pub async fn get_webdav_file_list(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(req): Query<WebdavPathRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = require_webdav_user_ns(&state, auth.access_token()).await?;
    let home = webdav_home(&state, &user_ns).await?;
    let path = req.path.unwrap_or_else(|| "/".to_string());
    let parts = normalize_rel_path(&path)?;
    let full = join_parts(&home, &parts);
    if !full.exists() {
        return Ok(Json(ApiResponse::err("路径不存在")));
    }
    if !full.is_dir() {
        return Ok(Json(ApiResponse::err("路径不是目录")));
    }
    let mut list = Vec::new();
    let mut dir = fs::read_dir(full)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
    {
        let meta = entry
            .metadata()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let child_path = build_relative_path(&parts, &name);
        list.push(serde_json::json!({
            "name": name,
            "size": meta.len(),
            "path": child_path,
            "lastModified": meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(now_ts()),
            "isDirectory": meta.is_dir()
        }));
    }
    Ok(Json(ApiResponse::ok(Value::from(list))))
}

pub async fn get_webdav_file(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(req): Query<WebdavPathRequest>,
) -> Result<Response, AppError> {
    let user_ns = require_webdav_user_ns(&state, auth.access_token()).await?;
    let home = webdav_home(&state, &user_ns).await?;
    let path = req.path.unwrap_or_default();
    if path.is_empty() {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }
    let parts = normalize_rel_path(&path)?;
    let full = join_parts(&home, &parts);
    if !full.exists() || full.is_dir() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let bytes = fs::read(full)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Response::new(axum::body::Body::from(bytes)))
}

pub async fn upload_file_to_webdav(
    State(state): State<AppState>,
    auth: AuthContext,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = require_webdav_user_ns(&state, auth.access_token()).await?;
    let home = webdav_home(&state, &user_ns).await?;
    let mut file_list = Vec::new();
    let mut path = "/".to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "path" {
            let val = field.text().await.unwrap_or_default();
            if !val.is_empty() {
                path = val;
            }
            continue;
        }
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "file".to_string());
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let rel = normalize_rel_path(&path)?;
        let dir = join_parts(&home, &rel);
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let target = dir.join(&filename);
        fs::write(&target, data)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        let meta = fs::metadata(&target)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        file_list.push(serde_json::json!({
            "name": filename,
            "size": meta.len(),
            "path": target.to_string_lossy().replace(home.to_string_lossy().as_ref(), ""),
            "lastModified": meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(now_ts()),
            "isDirectory": meta.is_dir()
        }));
    }
    Ok(Json(ApiResponse::ok(Value::from(file_list))))
}

pub async fn delete_webdav_file(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<WebdavPathRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = require_webdav_user_ns(&state, auth.access_token()).await?;
    let home = webdav_home(&state, &user_ns).await?;
    let path = req.path.unwrap_or_default();
    if path.is_empty() {
        return Ok(Json(ApiResponse::err("参数错误")));
    }
    let rel = normalize_rel_path(&path)?;
    let target = join_parts(&home, &rel);
    if !target.exists() {
        return Ok(Json(ApiResponse::err("路径不存在")));
    }
    if target.is_dir() {
        fs::remove_dir_all(target)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    } else {
        fs::remove_file(target)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_webdav_file_list(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<WebdavDeleteListRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = require_webdav_user_ns(&state, auth.access_token()).await?;
    let home = webdav_home(&state, &user_ns).await?;
    let paths = req.path.unwrap_or_default();
    for p in paths {
        if p.is_empty() {
            continue;
        }
        let rel = normalize_rel_path(&p)?;
        let target = join_parts(&home, &rel);
        if target.exists() {
            if target.is_dir() {
                let _ = fs::remove_dir_all(target).await;
            } else {
                let _ = fs::remove_file(target).await;
            }
        }
    }
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn webdav_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    method: Method,
    Path(path): Path<String>,
    body: Bytes,
) -> Response {
    let user_ns = match resolve_webdav_user(&state, &headers).await {
        Ok(ns) => ns,
        Err(status) => return status.into_response(),
    };
    let home = match webdav_home(&state, &user_ns).await {
        Ok(h) => h,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let rel_path = format!("/{}", path);
    let rel = match normalize_rel_path(&rel_path) {
        Ok(p) => p,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let full = join_parts(&home, &rel);

    match method.as_str() {
        "OPTIONS" => StatusCode::OK.into_response(),
        "PROPFIND" => webdav_propfind(&full, &rel_path).await,
        "MKCOL" => webdav_mkcol(&full).await,
        "PUT" => webdav_put(&full, body).await,
        "GET" => webdav_get(&full).await,
        "DELETE" => webdav_delete(&full).await,
        "MOVE" => webdav_move(&home, &full, &headers).await,
        "COPY" => webdav_copy(&home, &full, &headers).await,
        "LOCK" => webdav_lock(&rel_path),
        "UNLOCK" => webdav_unlock(&headers),
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

async fn require_webdav_user_ns(
    state: &AppState,
    access_token: Option<&str>,
) -> Result<String, AppError> {
    state.user_service.require_webdav_user(access_token).await
}

async fn resolve_webdav_user(state: &AppState, headers: &HeaderMap) -> Result<String, StatusCode> {
    if !state.user_service.secure_enabled() {
        return Err(StatusCode::FORBIDDEN);
    }
    let auth = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !auth.to_ascii_lowercase().starts_with("basic ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let b64 = auth[6..].trim();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let decoded = String::from_utf8_lossy(&decoded);
    let parts: Vec<&str> = decoded.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let username = parts[0];
    let password = parts[1];
    match state
        .user_service
        .verify_basic_webdav(username, password)
        .await
    {
        Ok(Some(_)) => Ok(username.to_string()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn webdav_home(state: &AppState, user_ns: &str) -> Result<PathBuf, AppError> {
    let dir = PathBuf::from(&state.config.storage_dir)
        .join("webdav")
        .join(user_ns);
    fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(dir)
}

fn normalize_rel_path(path: &str) -> Result<Vec<String>, AppError> {
    let mut parts = Vec::new();
    for p in path.split('/') {
        if p.is_empty() || p == "." {
            continue;
        }
        if p == ".." {
            return Err(AppError::BadRequest("非法路径".to_string()));
        }
        parts.push(p.to_string());
    }
    Ok(parts)
}

fn join_parts(home: &PathBuf, parts: &Vec<String>) -> PathBuf {
    let mut p = home.clone();
    for part in parts {
        p = p.join(part);
    }
    p
}

fn build_relative_path(parts: &[String], name: &str) -> String {
    if parts.is_empty() {
        format!("/{}", name)
    } else {
        format!("/{}/{}", parts.join("/"), name)
    }
}

async fn webdav_propfind(full: &PathBuf, rel: &str) -> Response {
    if !full.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let mut response =
        String::from(r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:">"#);
    let add_entry = |resp: &mut String, href: String, is_dir: bool, size: u64, modified: String| {
        if is_dir {
            resp.push_str(&format!(r#"<D:response><D:href>{}</D:href><D:propstat><D:status>HTTP/1.1 200 OK</D:status><D:prop><D:getlastmodified>{}</D:getlastmodified><D:creationdate>{}</D:creationdate><D:resourcetype><D:collection /></D:resourcetype><D:displayname></D:displayname></D:prop></D:propstat></D:response>"#, href, modified, modified));
        } else {
            resp.push_str(&format!(r#"<D:response><D:href>{}</D:href><D:propstat><D:status>HTTP/1.1 200 OK</D:status><D:prop><D:getlastmodified>{}</D:getlastmodified><D:creationdate>{}</D:creationdate><D:resourcetype /><D:displayname></D:displayname><D:getcontentlength>{}</D:getcontentlength></D:prop></D:propstat></D:response>"#, href, modified, modified, size));
        }
    };
    let href_base = if rel.ends_with('/') {
        rel.to_string()
    } else {
        format!("{}/", rel)
    };
    let meta = std::fs::metadata(full).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    add_entry(
        &mut response,
        href_base.clone(),
        full.is_dir(),
        size,
        modified.to_string(),
    );
    if full.is_dir() {
        if let Ok(entries) = std::fs::read_dir(full) {
            for entry in entries.flatten() {
                let file = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let href = format!("{}{}", href_base, name);
                let meta = entry.metadata().ok();
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                add_entry(
                    &mut response,
                    href,
                    file.is_dir(),
                    size,
                    modified.to_string(),
                );
            }
        }
    }
    response.push_str("</D:multistatus>");
    let mut resp = Response::new(axum::body::Body::from(response));
    *resp.status_mut() = StatusCode::MULTI_STATUS;
    resp
}

async fn webdav_mkcol(full: &PathBuf) -> Response {
    if full.exists() {
        return StatusCode::CREATED.into_response();
    }
    match fs::create_dir_all(full).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn webdav_put(full: &PathBuf, body: Bytes) -> Response {
    if let Some(parent) = full.parent() {
        if !parent.exists() {
            return StatusCode::CONFLICT.into_response();
        }
    }
    if full.exists() && full.is_dir() {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    if let Err(_) = fs::write(full, body).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    StatusCode::CREATED.into_response()
}

async fn webdav_get(full: &PathBuf) -> Response {
    if !full.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    if full.is_dir() {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    match fs::read(full).await {
        Ok(data) => Response::new(axum::body::Body::from(data)),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn webdav_delete(full: &PathBuf) -> Response {
    if !full.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let res = if full.is_dir() {
        fs::remove_dir_all(full).await
    } else {
        fs::remove_file(full).await
    };
    match res {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn webdav_move(home: &PathBuf, full: &PathBuf, headers: &HeaderMap) -> Response {
    let destination = headers
        .get("Destination")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if destination.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let dest_path = destination
        .split("/reader3/webdav/")
        .nth(1)
        .unwrap_or("")
        .to_string();
    let rel = match normalize_rel_path(&format!("/{}", dest_path)) {
        Ok(p) => p,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let dest = join_parts(home, &rel);
    if dest.exists() {
        let overwrite = headers
            .get("Overwrite")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if overwrite.is_empty() {
            return StatusCode::PRECONDITION_FAILED.into_response();
        }
        let _ = if dest.is_dir() {
            std::fs::remove_dir_all(&dest)
        } else {
            std::fs::remove_file(&dest)
        };
    }
    match std::fs::rename(full, dest) {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn webdav_copy(home: &PathBuf, full: &PathBuf, headers: &HeaderMap) -> Response {
    let destination = headers
        .get("Destination")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if destination.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let dest_path = destination
        .split("/reader3/webdav/")
        .nth(1)
        .unwrap_or("")
        .to_string();
    let rel = match normalize_rel_path(&format!("/{}", dest_path)) {
        Ok(p) => p,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let dest = join_parts(home, &rel);
    if dest.exists() {
        let overwrite = headers
            .get("Overwrite")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if overwrite.is_empty() {
            return StatusCode::PRECONDITION_FAILED.into_response();
        }
        let _ = if dest.is_dir() {
            std::fs::remove_dir_all(&dest)
        } else {
            std::fs::remove_file(&dest)
        };
    }
    let res = if full.is_dir() {
        copy_dir(full, &dest)
    } else {
        std::fs::copy(full, &dest).map(|_| ())
    };
    match res {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn copy_dir(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

fn webdav_lock(path: &str) -> Response {
    let lock_token = format!("urn:uuid:{}", Uuid::new_v4());
    let response = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:prop xmlns:D="DAV:"><D:lockdiscovery><D:activelock><D:locktype><write /></D:locktype><D:lockscope><exclusive /></D:lockscope><D:locktoken><D:href>{}</D:href></D:locktoken><D:lockroot><D:href>{}</D:href></D:lockroot><D:depth>infinity</D:depth><D:timeout>Second-3600</D:timeout></D:activelock></D:lockdiscovery></D:prop>"#,
        lock_token, path
    );
    let mut resp = Response::new(axum::body::Body::from(response));
    if let Ok(v) = lock_token.parse() {
        resp.headers_mut().insert("Lock-Token", v);
    }
    *resp.status_mut() = StatusCode::OK;
    resp
}

fn webdav_unlock(headers: &HeaderMap) -> Response {
    let lock_token = headers
        .get("Lock-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mut resp = StatusCode::NO_CONTENT.into_response();
    if let Ok(v) = lock_token.parse() {
        resp.headers_mut().insert("Lock-Token", v);
    }
    resp
}

// ==================== 远程 WebDAV 客户端功能 ====================

use std::collections::HashMap;
use chrono;
use zip::write::FileOptions;
use zip::write::ZipWriter;
use zip::read::ZipArchive;
use std::io::Read;

// 远程 WebDAV 配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebdavRemoteConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveWebdavConfigRequest {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct WebdavConfigResponse {
    pub server_url: String,
    pub username: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct BackupResult {
    pub file_name: String,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreResult {
    pub restored: bool,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub connected: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct RemoteWebdavFileEntry {
    pub name: String,
    pub size: u64,
    pub path: String,
    pub last_modified: i64,
    pub is_directory: bool,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct BackupPayload {
    pub version: u32,
    pub created_at: String,
    pub app: String,
    pub bookshelf: BookshelfData,
    pub book_sources: Vec<serde_json::Value>,
    pub rss_sources: Vec<serde_json::Value>,
    pub bookmarks: Vec<serde_json::Value>,
    pub replace_rules: Vec<serde_json::Value>,
    pub local_state: HashMap<String, String>,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct BookshelfData {
    pub books: Vec<serde_json::Value>,
    pub groups: Vec<serde_json::Value>,
}

// ==================== 远程 WebDAV API 端点 ====================

pub async fn save_webdav_config(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<SaveWebdavConfigRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if !req.server_url.starts_with("http://") && !req.server_url.starts_with("https://") {
        return Ok(Json(ApiResponse::err("URL 格式不正确")));
    }
    let user_id = auth.user_id().ok_or_else(|| AppError::Unauthorized)?;
    state.webdav_config.lock().unwrap().insert(
        user_id.to_string(),
        WebdavRemoteConfig {
            server_url: req.server_url,
            username: req.username,
            password: req.password,
            enabled: true,
        }
    );
    Ok(Json(ApiResponse::ok(())))
}

pub async fn get_webdav_config(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<WebdavConfigResponse>>, AppError> {
    let user_id = auth.user_id().ok_or_else(|| AppError::Unauthorized)?;
    let config = state.webdav_config.lock().unwrap().get(user_id).cloned();
    match config {
        Some(c) => Ok(Json(ApiResponse::ok(WebdavConfigResponse {
            server_url: c.server_url,
            username: c.username,
            enabled: c.enabled,
        }))),
        None => Ok(Json(ApiResponse::ok(WebdavConfigResponse {
            server_url: String::new(),
            username: String::new(),
            enabled: false,
        }))),
    }
}

pub async fn test_webdav_connection(
    Json(req): Json<SaveWebdavConfigRequest>,
) -> Result<Json<ApiResponse<TestResult>>, AppError> {
    use reqwest::Client;
    let client = Client::new();
    let auth_header = format!("Basic {}", 
        base64::encode(format!("{}:{}", req.username, req.password)));
    let response = client.request(
        reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
        &req.server_url,
    )
    .header("Authorization", auth_header)
    .header("Depth", "0")
    .body("")
    .send()
    .await;
    match response {
        Ok(resp) if resp.status().as_u16() == 207 => Ok(Json(ApiResponse::ok(TestResult {
            connected: true,
            message: "连接成功".to_string(),
        }))),
        Ok(resp) => Ok(Json(ApiResponse::ok(TestResult {
            connected: false,
            message: format!("HTTP {}", resp.status()),
        }))),
        Err(e) => Ok(Json(ApiResponse::ok(TestResult {
            connected: false,
            message: format!("连接失败: {}", e),
        }))),
    }
}

pub async fn backup_to_remote_webdav(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<BackupRequest>,
) -> Result<Json<ApiResponse<BackupResult>>, AppError> {
    let user_id = auth.user_id().ok_or_else(|| AppError::Unauthorized)?;
    let config = state.webdav_config.lock().unwrap()
        .get(user_id).ok_or_else(|| AppError::BadRequest("未配置 WebDAV"))?;
    let filename = format!("reader-backup-{}.json", 
        chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let remote_path = format!("{}/{}", 
        req.path.trim_end_matches('/'), filename);
    let client = reqwest::Client::new();
    let auth_header = format!("Basic {}", 
        base64::encode(format!("{}:{}", config.username, config.password)));
    let backup_json = serde_json::json!({
        "version": 1,
        "createdAt": chrono::Utc::now().to_rfc3339(),
        "app": "reader-rust-frontend",
        "bookshelf": { "books": [], "groups": [] },
        "bookSources": [],
        "rssSources": [],
        "bookmarks": [],
        "replaceRules": [],
        "localState": {}
    });
    let json_str = serde_json::to_string_pretty(&backup_json)
        .map_err(|e| AppError::Internal(e.into()))?;
    let mut zip_buf = Vec::new();
    {
        let mut archive = ZipWriter::new(&mut zip_buf);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        archive.start_file("backup.json", options)
            .map_err(|e| AppError::Internal(e.into()))?;
        archive.write_all(json_str.as_bytes())
            .map_err(|e| AppError::Internal(e.into()))?;
        archive.finish()
            .map_err(|e| AppError::Internal(e.into()))?;
    }
    let response = client.put(format!("{}{}", config.server_url, remote_path))
        .header("Authorization", auth_header)
        .body(zip_buf)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    if !response.status().is_success() {
        return Ok(Json(ApiResponse::err(
            format!("上传失败: HTTP {}", response.status())
        )));
    }
    Ok(Json(ApiResponse::ok(BackupResult {
        file_name: filename,
        size: zip_buf.len() as u64,
    })))
}

pub async fn get_remote_webdav_file_list(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(req): Query<WebdavPathRequest>,
) -> Result<Json<ApiResponse<Vec<RemoteWebdavFileEntry>>>, AppError> {
    let user_id = auth.user_id().ok_or_else(|| AppError::Unauthorized)?;
    let config = state.webdav_config.lock().unwrap()
        .get(user_id).ok_or_else(|| AppError::BadRequest("未配置 WebDAV"))?;
    let client = reqwest::Client::new();
    let auth_header = format!("Basic {}", 
        base64::encode(format!("{}:{}", config.username, config.password)));
    let url = format!("{}{}", config.server_url, req.path);
    let response = client.request(
        reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
        &url,
    )
    .header("Authorization", auth_header)
    .header("Depth", "1")
    .body("")
    .send()
    .await
    .map_err(|e| AppError::Internal(e.into()))?;
    if response.status().as_u16() != 207 {
        return Ok(Json(ApiResponse::err("PROPFIND 失败")));
    }
    let body = response.text().await
        .map_err(|e| AppError::Internal(e.into()))?;
    let files = parse_webdav_response(&body, &config.server_url, &req.path);
    Ok(Json(ApiResponse::ok(files)))
}

pub async fn restore_from_remote_webdav(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<RestoreRequest>,
) -> Result<Json<ApiResponse<RestoreResult>>, AppError> {
    let user_id = auth.user_id().ok_or_else(|| AppError::Unauthorized)?;
    let config = state.webdav_config.lock().unwrap()
        .get(user_id).ok_or_else(|| AppError::BadRequest("未配置 WebDAV"))?;
    let client = reqwest::Client::new();
    let auth_header = format!("Basic {}", 
        base64::encode(format!("{}:{}", config.username, config.password)));
    let url = format!("{}{}", config.server_url, req.path);
    let response = client.get(&url)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    if !response.status().is_success() {
        return Ok(Json(ApiResponse::err("下载失败")));
    }
    let zip_data = response.bytes().await
        .map_err(|e| AppError::Internal(e.into()))?;
    let mut archive = ZipArchive::new(std::io::Cursor::new(zip_data))
        .map_err(|e| AppError::Internal(e.into()))?;
    let mut json_str = String::new();
    let mut found = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| AppError::Internal(e.into()))?;
        if file.name() == "backup.json" {
            std::io::Read::read_to_string(&mut file, &mut json_str)
                .map_err(|e| AppError::Internal(e.into()))?;
            found = true;
            break;
        }
    }
    if !found {
        return Ok(Json(ApiResponse::err("备份文件中未找到 backup.json")));
    }
    let _payload: BackupPayload = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Internal(e.into()))?;
    Ok(Json(ApiResponse::ok(RestoreResult { restored: true })))
}

fn parse_webdav_response(
    xml: &str,
    server_url: &str,
    base_path: &str,
) -> Vec<RemoteWebdavFileEntry> {
    let mut files = Vec::new();
    for response in xml.split("<D:response>") {
        if !response.contains("<D:href>") {
            continue;
        }
        let href = response.split("<D:href>")
            .nth(1)
            .and_then(|s| s.split("</D:href>").next())
            .unwrap_or("");
        if href.is_empty() || href == base_path || href == format!("{}/", base_path) {
            continue;
        }
        let name = href.split('/').last().unwrap_or("");
        let is_dir = response.contains("<D:collection />");
        let size_str = response.split("<D:getcontentlength>")
            .nth(1)
            .and_then(|s| s.split("</D:getcontentlength>").next())
            .unwrap_or("0");
        let size: u64 = size_str.parse().unwrap_or(0);
        let modified_str = response.split("<D:getlastmodified>")
            .nth(1)
            .and_then(|s| s.split("</D:getlastmodified>").next())
            .unwrap_or("");
        let last_modified: i64 = if !modified_str.is_empty() {
            chrono::DateTime::parse_from_rfc2822(modified_str)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(now_ts())
        } else {
            now_ts()
        };
        files.push(RemoteWebdavFileEntry {
            name: name.to_string(),
            size,
            path: href.to_string(),
            last_modified,
            is_directory: is_dir,
        });
    }
    files
}
