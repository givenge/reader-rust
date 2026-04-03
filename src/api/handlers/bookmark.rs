use axum::{extract::State, Json};
use serde_json::Value;
use crate::api::AppState;
use crate::api::auth::AuthContext;

use crate::error::error::{ApiResponse, AppError};
use crate::model::bookmark::Bookmark;
use tokio::fs;
use std::path::PathBuf;

pub async fn get_bookmarks(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let list = read_list::<Bookmark>(&state, &user_ns, "bookmark.json").await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())))
}

pub async fn save_bookmark(State(state): State<AppState>, auth: AuthContext, Json(bookmark): Json<Bookmark>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    if bookmark.book_name.is_empty() && bookmark.book_author.is_empty() {
        return Err(AppError::BadRequest("书籍信息错误".to_string()));
    }
    let mut list = read_list::<Bookmark>(&state, &user_ns, "bookmark.json").await?;
    upsert_by_key(&mut list, bookmark, |b| format!("{}_{}", b.book_name, b.book_author));
    write_list(&state, &user_ns, "bookmark.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn save_bookmarks(State(state): State<AppState>, auth: AuthContext, Json(mut bookmarks): Json<Vec<Bookmark>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<Bookmark>(&state, &user_ns, "bookmark.json").await?;
    bookmarks.retain(|b| !(b.book_name.is_empty() && b.book_author.is_empty()));
    for b in bookmarks {
        upsert_by_key(&mut list, b, |v| format!("{}_{}", v.book_name, v.book_author));
    }
    write_list(&state, &user_ns, "bookmark.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_bookmark(State(state): State<AppState>, auth: AuthContext, Json(bookmark): Json<Bookmark>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<Bookmark>(&state, &user_ns, "bookmark.json").await?;
    list.retain(|b| !(b.book_name == bookmark.book_name && b.book_author == bookmark.book_author));
    write_list(&state, &user_ns, "bookmark.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_bookmarks(State(state): State<AppState>, auth: AuthContext, Json(bookmarks): Json<Vec<Bookmark>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<Bookmark>(&state, &user_ns, "bookmark.json").await?;
    for b in bookmarks {
        list.retain(|v| !(v.book_name == b.book_name && v.book_author == b.book_author));
    }
    write_list(&state, &user_ns, "bookmark.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
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
