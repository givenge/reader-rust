use axum::{extract::State, Json};
use serde_json::Value;
use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::error::error::{ApiResponse, AppError};
use crate::model::book_group::BookGroup;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GroupIdParam {
    #[serde(rename = "groupId")]
    group_id: Option<i64>,
}

pub async fn get_book_groups(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let groups = state.book_group_service.get_groups(&user_ns).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(groups).unwrap_or_default())))
}

pub async fn save_book_group(State(state): State<AppState>, auth: AuthContext, Json(group): Json<BookGroup>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    state.book_group_service.save_group(&user_ns, group).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

pub async fn delete_book_group(State(state): State<AppState>, auth: AuthContext, Json(param): Json<GroupIdParam>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let gid = param.group_id.ok_or_else(|| AppError::BadRequest("groupId required".to_string()))?;
    state.book_group_service.delete_group(&user_ns, gid).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

pub async fn save_book_group_order(State(state): State<AppState>, auth: AuthContext, Json(groups): Json<Vec<BookGroup>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    state.book_group_service.save_groups(&user_ns, &groups).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

#[derive(Debug, Deserialize)]
pub struct SaveBookGroupIdParam {
    #[serde(rename = "bookUrl")]
    book_url: Option<String>,
    #[serde(rename = "groupId")]
    group_id: Option<i64>,
}

pub async fn save_book_group_id(State(state): State<AppState>, auth: AuthContext, Json(param): Json<SaveBookGroupIdParam>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let url = param.book_url.ok_or_else(|| AppError::BadRequest("bookUrl required".to_string()))?;
    let gid = param.group_id.unwrap_or(0);
    let mut book = state.book_service.get_shelf_book(&user_ns, &url).await?.ok_or_else(|| AppError::NotFound("Book not found".to_string()))?;
    book.group = Some(gid);
    state.book_service.save_book(&user_ns, book).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

#[derive(Debug, Deserialize)]
pub struct MultiBookGroupParam {
    #[serde(rename = "bookUrls")]
    book_urls: Option<Vec<String>>,
    #[serde(rename = "groupId")]
    group_id: Option<i64>,
}

pub async fn add_book_group_multi(State(state): State<AppState>, auth: AuthContext, Json(param): Json<MultiBookGroupParam>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let urls = param.book_urls.unwrap_or_default();
    let gid = param.group_id.unwrap_or(0);
    // Here we should bitwise OR the group_id if it's bitfield, but reader original uses it as bitfield!
    // Wait! Original `legado` Reader uses bitwise grouping: book.group is a bitfield!
    // "addBookGroupMulti": books.forEach { it.group = it.group | groupId }
    for url in urls {
        if let Some(mut book) = state.book_service.get_shelf_book(&user_ns, &url).await? {
            let cur = book.group.unwrap_or(0);
            book.group = Some(cur | gid);
            let _ = state.book_service.save_book(&user_ns, book).await;
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}

pub async fn remove_book_group_multi(State(state): State<AppState>, auth: AuthContext, Json(param): Json<MultiBookGroupParam>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let urls = param.book_urls.unwrap_or_default();
    let gid = param.group_id.unwrap_or(0);
    // remove: book.group = book.group & ~groupId
    for url in urls {
        if let Some(mut book) = state.book_service.get_shelf_book(&user_ns, &url).await? {
            let cur = book.group.unwrap_or(0);
            book.group = Some(cur & !gid);
            let _ = state.book_service.save_book(&user_ns, book).await;
        }
    }
    Ok(Json(ApiResponse::ok(serde_json::json!("success"))))
}
