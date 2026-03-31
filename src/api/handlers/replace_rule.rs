use axum::{extract::State, Json};
use serde_json::Value;
use crate::api::AppState;
use crate::api::auth::AuthContext;

use crate::error::error::{ApiResponse, AppError};
use crate::model::replace_rule::ReplaceRule;
use tokio::fs;
use std::path::PathBuf;

pub async fn get_replace_rules(State(state): State<AppState>, auth: AuthContext) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let list = read_list::<ReplaceRule>(&state, &user_ns, "replaceRule.json").await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(list).unwrap_or_default())))
}

pub async fn save_replace_rule(State(state): State<AppState>, auth: AuthContext, Json(rule): Json<ReplaceRule>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    if rule.name.is_empty() {
        return Err(AppError::BadRequest("名称不能为空".to_string()));
    }
    if rule.pattern.is_empty() {
        return Err(AppError::BadRequest("替换规则不能为空".to_string()));
    }
    let mut list = read_list::<ReplaceRule>(&state, &user_ns, "replaceRule.json").await?;
    upsert_by_key(&mut list, rule, |r| r.name.clone());
    write_list(&state, &user_ns, "replaceRule.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn save_replace_rules(State(state): State<AppState>, auth: AuthContext, Json(mut rules): Json<Vec<ReplaceRule>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<ReplaceRule>(&state, &user_ns, "replaceRule.json").await?;
    rules.retain(|r| !r.name.is_empty() && !r.pattern.is_empty());
    for r in rules {
        upsert_by_key(&mut list, r, |v| v.name.clone());
    }
    write_list(&state, &user_ns, "replaceRule.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_replace_rule(State(state): State<AppState>, auth: AuthContext, Json(rule): Json<ReplaceRule>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<ReplaceRule>(&state, &user_ns, "replaceRule.json").await?;
    list.retain(|r| r.name != rule.name);
    write_list(&state, &user_ns, "replaceRule.json", &list).await?;
    Ok(Json(ApiResponse::ok(Value::String("".to_string()))))
}

pub async fn delete_replace_rules(State(state): State<AppState>, auth: AuthContext, Json(rules): Json<Vec<ReplaceRule>>) -> Result<Json<ApiResponse<Value>>, AppError> {
    let user_ns = resolve_user_ns(&state, auth.access_token(), auth.secure_key(), auth.user_ns()).await?;
    let mut list = read_list::<ReplaceRule>(&state, &user_ns, "replaceRule.json").await?;
    for r in rules {
        list.retain(|v| v.name != r.name);
    }
    write_list(&state, &user_ns, "replaceRule.json", &list).await?;
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
