use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
    #[error("db error")]
    Db(#[from] sqlx::Error),
    #[error("http error")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    #[serde(rename = "isSuccess")]
    pub is_success: bool,
    #[serde(rename = "errorMsg")]
    pub error_msg: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { is_success: true, error_msg: "".to_string(), data: Some(data) }
    }
    pub fn err(message: impl Into<String>) -> Self {
        Self { is_success: false, error_msg: message.into(), data: None }
    }
    pub fn err_with_data(message: impl Into<String>, data: T) -> Self {
        Self { is_success: false, error_msg: message.into(), data: Some(data) }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("ERROR: {:?}", self);
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string()),
            AppError::Http(_) => (StatusCode::BAD_GATEWAY, "upstream error".to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };
        let body = Json(ApiResponse::<serde_json::Value>::err(message));
        (status, body).into_response()
    }
}
