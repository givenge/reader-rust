use axum::{extract::{State, Query, Multipart}, Json};
use serde::Deserialize;
use serde_json::Value;
use tokio::fs;
use std::path::PathBuf;
use crate::api::AppState;
use crate::api::auth::AuthContext;

use crate::error::error::{ApiResponse, AppError};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "isLogin")]
    pub is_login: Option<bool>,
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileTypeQuery {
    #[serde(rename = "type")]
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    #[serde(rename = "enableWebdav")]
    pub enable_webdav: Option<bool>,
    #[serde(rename = "enableLocalStore")]
    pub enable_local_store: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteFileRequest {
    pub url: Option<String>,
}

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let username = req.username.unwrap_or_default();
    let password = req.password.unwrap_or_default();
    let is_login = req.is_login.unwrap_or(false);
    let is_new_user = !is_login && !username.is_empty(); // registration attempt
    let data = state.user_service.login(&username, &password, is_login, req.code.as_deref()).await?;
    // If this was a new user registration, copy default book sources
    if is_new_user {
        let _ = state.book_source_service.copy_default_to_user(&username).await;
    }
    Ok(Json(ApiResponse::ok(data)))
}

pub async fn logout(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::err("不支持的操作")));
    }
    if let Some(token) = auth.access_token() {
        let _ = state.user_service.logout(token).await;
    }
    Ok(Json(ApiResponse::err_with_data("请重新登录", Value::String("NEED_LOGIN".to_string()))))
}

pub async fn get_user_info(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    let (user_info, secure, secure_key) = state.user_service.get_user_info(auth.access_token()).await?;
    let data = serde_json::json!({
        "userInfo": user_info,
        "secure": secure,
        "secureKey": secure_key,
    });
    Ok(Json(ApiResponse::ok(data)))
}

pub async fn save_user_config(State(state): State<AppState>, auth: AuthContext, Json(body): Json<Value>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = match state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await {
        Ok(ns) => ns,
        Err(_) => return Ok(Json(ApiResponse::err_with_data("请登录后使用", Value::String("NEED_LOGIN".to_string())))),
    };
    state.user_service.save_user_config(&user_ns, body).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn get_user_config(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = match state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await {
        Ok(ns) => ns,
        Err(_) => return Ok(Json(ApiResponse::err_with_data("请登录后使用", Value::String("NEED_LOGIN".to_string())))),
    };
    let cfg = state.user_service.get_user_config(&user_ns).await?;
    Ok(Json(ApiResponse::ok(cfg)))
}

pub async fn get_user_list(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::err("不支持的操作")));
    }
    // Check if admin (either by is_admin flag or secure key)
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let list = state.user_service.get_user_list().await?;
    Ok(Json(ApiResponse::ok(Value::from(list))))
}

pub async fn add_user(State(state): State<AppState>, auth: AuthContext, Json(req): Json<AddUserRequest>) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::err("不支持的操作")));
    }
    // Check if admin (either by is_admin flag or secure key)
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let username = req.username.unwrap_or_default();
    let password = req.password.unwrap_or_default();
    let list = state.user_service.add_user(&username, &password).await?;
    Ok(Json(ApiResponse::ok(Value::from(list))))
}

pub async fn reset_password(State(state): State<AppState>, auth: AuthContext, Json(req): Json<ResetPasswordRequest>) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::err("不支持的操作")));
    }
    // Check if admin (either by is_admin flag or secure key)
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let username = req.username.unwrap_or_default();
    let password = req.password.unwrap_or_default();
    state.user_service.reset_password(&username, &password).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_users(State(state): State<AppState>, auth: AuthContext, Json(list): Json<Vec<String>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::err("不支持的操作")));
    }
    // Check if admin (either by is_admin flag or secure key)
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let users = state.user_service.delete_users(&list).await?;
    Ok(Json(ApiResponse::ok(Value::from(users))))
}

pub async fn update_user(State(state): State<AppState>, auth: AuthContext, Json(req): Json<UpdateUserRequest>) -> Result<Json<ApiResponse<Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::err("不支持的操作")));
    }
    // Check if admin (either by is_admin flag or secure key)
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let username = req.username.unwrap_or_default();
    let list = state.user_service.update_user(&username, req.enable_webdav, req.enable_local_store).await?;
    Ok(Json(ApiResponse::ok(Value::from(list))))
}

pub async fn upload_file(State(state): State<AppState>, auth: AuthContext, Query(q): Query<FileTypeQuery>, mut multipart: Multipart) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = match state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await {
        Ok(ns) => ns,
        Err(_) => return Ok(Json(ApiResponse::err_with_data("请登录后使用", Value::String("NEED_LOGIN".to_string())))),
    };
    let mut file_list = Vec::new();
    let mut file_type = "images".to_string();
    if let Some(t) = q.file_type.as_deref() {
        if !t.is_empty() {
            file_type = t.to_string();
        }
    }
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "file".to_string());
        let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
        let dir = PathBuf::from(&state.config.storage_dir).join("assets").join(&user_ns).join(&file_type);
        fs::create_dir_all(&dir).await.map_err(|e| AppError::Internal(e.into()))?;
        let path = dir.join(&name);
        fs::write(&path, data).await.map_err(|e| AppError::Internal(e.into()))?;
        let url = format!("/assets/{}/{}/{}", user_ns, file_type, name);
        file_list.push(Value::String(url));
    }
    Ok(Json(ApiResponse::ok(Value::from(file_list))))
}

pub async fn delete_file(State(state): State<AppState>, auth: AuthContext, Json(req): Json<DeleteFileRequest>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = match state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await {
        Ok(ns) => ns,
        Err(_) => return Ok(Json(ApiResponse::err_with_data("请登录后使用", Value::String("NEED_LOGIN".to_string())))),
    };
    let url = req.url.unwrap_or_default();
    if url.is_empty() {
        return Ok(Json(ApiResponse::err("请输入文件链接")));
    }
    let prefix = format!("/assets/{}/", user_ns);
    if !url.starts_with(&prefix) {
        return Ok(Json(ApiResponse::err("文件链接错误")));
    }
    let full_path = PathBuf::from(&state.config.storage_dir).join(url.trim_start_matches('/'));
    let _ = fs::remove_file(full_path).await;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}
