use axum::{extract::{State, Query}, Json};
use serde::Deserialize;
use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::error::error::{ApiResponse, AppError};
use crate::model::book_source::BookSource;

#[derive(Debug, Deserialize)]
pub struct BookSourceUrlParam {
    #[serde(rename = "bookSourceUrl")]
    book_source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsernameParam {
    pub username: Option<String>,
}

pub async fn save_book_source(State(state): State<AppState>, auth: AuthContext, Json(source): Json<BookSource>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    state.book_source_service.save(&user_ns, source).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"saved": true}))))
}

pub async fn save_book_sources(State(state): State<AppState>, auth: AuthContext, Json(payload): Json<serde_json::Value>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let sources = extract_sources(payload)?;
    if sources.is_empty() {
        return Err(AppError::BadRequest("empty book sources".to_string()));
    }
    let count = sources.len();
    state.book_source_service.save_many(&user_ns, sources).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"saved": true, "count": count}))))
}

pub async fn get_book_source(State(state): State<AppState>, auth: AuthContext, Query(q): Query<BookSourceUrlParam>, body: Option<Json<BookSourceUrlParam>>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let url = q.book_source_url.or_else(|| body.map(|b| b.0.book_source_url).flatten());
    let url = url.ok_or_else(|| AppError::BadRequest("bookSourceUrl required".to_string()))?;
    let source = state.book_source_service.get(&user_ns, &url).await?
        .ok_or_else(|| AppError::NotFound("bookSource not found".to_string()))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(source).unwrap_or_default())))
}

pub async fn get_book_sources(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let list = state.book_source_service.list(&user_ns).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())))
}

pub async fn delete_book_source(State(state): State<AppState>, auth: AuthContext, Json(param): Json<BookSourceUrlParam>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let url = param.book_source_url.ok_or_else(|| AppError::BadRequest("bookSourceUrl required".to_string()))?;
    state.book_source_service.delete(&user_ns, &url).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": true}))))
}

pub async fn delete_book_sources(State(state): State<AppState>, auth: AuthContext, Json(list): Json<Vec<BookSourceUrlParam>>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    for item in list {
        if let Some(url) = item.book_source_url {
            state.book_source_service.delete(&user_ns, &url).await?;
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": true}))))
}

pub async fn delete_all_book_sources(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    state.book_source_service.delete_all(&user_ns).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": true}))))
}

fn extract_sources(payload: serde_json::Value) -> Result<Vec<BookSource>, AppError> {
    if payload.is_array() {
        return serde_json::from_value::<Vec<BookSource>>(payload)
            .map_err(|e| AppError::BadRequest(e.to_string()));
    }
    if let Some(obj) = payload.as_object() {
        for key in ["bookSourceList", "bookSources", "data", "sources"] {
            if let Some(v) = obj.get(key) {
                if v.is_array() {
                    return serde_json::from_value::<Vec<BookSource>>(v.clone())
                        .map_err(|e| AppError::BadRequest(e.to_string()));
                }
            }
        }
    }
    Err(AppError::BadRequest("invalid book sources payload".to_string()))
}

#[derive(Debug, Deserialize)]
pub struct RemoteSourceParam {
    url: String,
}

pub async fn read_remote_source_file(
    Json(param): Json<RemoteSourceParam>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| AppError::Internal(e.into()))?;

    let text = client.get(&param.url)
        .send().await
        .map_err(|e| {
            tracing::error!("Failed to fetch remote source from {}: {:?}", param.url, e);
            AppError::BadRequest(format!("网络请求失败: {}", e))
        })?
        .text().await
        .map_err(|e| AppError::BadRequest(format!("读取响应失败: {}", e)))?;

    println!("DEBUG: remote source file length: {}", text.len());
    println!("DEBUG: remote source file preview: {}", &text.chars().take(500).collect::<String>());

    let sources: Vec<BookSource> = serde_json::from_str(&text)
        .or_else(|e| {
            println!("DEBUG: direct parse failed: {:?}", e);
            // some sources are wrapped in a list or object
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                extract_sources(v)
            } else {
                Err(AppError::BadRequest("invalid book sources json format".to_string()))
            }
        })?;

    println!("DEBUG: parsed {} book sources", sources.len());

    // Return as array of JSON strings (frontend expects each item to be a JSON string)
    let json_str = serde_json::to_string(&sources)
        .map_err(|e| AppError::BadRequest(format!("序列化书源失败: {}", e)))?;

    Ok(Json(ApiResponse::ok(vec![json_str])))
}

use axum::extract::Multipart;

pub async fn read_source_file(
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if let Some(file_name) = field.file_name() {
            if file_name.ends_with(".json") || file_name.ends_with(".txt") {
                let bytes = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                let text = String::from_utf8_lossy(&bytes);
                let sources: Vec<BookSource> = serde_json::from_str(&text)
                    .or_else(|_| {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                            extract_sources(v)
                        } else {
                            Err(AppError::BadRequest("invalid book sources json format".to_string()))
                        }
                    })?;
                return Ok(Json(serde_json::to_value(sources).unwrap_or_default()));
            }
        }
    }
    Err(AppError::BadRequest("No json file uploaded".to_string()))
}

pub async fn set_as_default_book_sources(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(param): Json<UsernameParam>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    // Check if admin
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", serde_json::Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let username = param.username.ok_or_else(|| AppError::BadRequest("username required".to_string()))?;
    let count = state.book_source_service.set_as_default(&username).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"success": true, "count": count}))))
}
