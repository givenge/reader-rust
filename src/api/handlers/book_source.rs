use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{
        header::{self, HeaderMap, HeaderValue},
        Method,
    },
    response::{IntoResponse, Response},
    Json,
};
use regex::{Captures, Regex};
use serde::Deserialize;
use url::Url;
use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::error::error::{ApiResponse, AppError};
use crate::model::book_source::BookSource;
use crate::util::text::{normalize_source_url, repair_encoded_url};

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

pub async fn get_default_book_source_owner(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    if !state.user_service.secure_enabled() {
        return Ok(Json(ApiResponse::ok(serde_json::json!({ "username": null }))));
    }
    let is_admin = state.user_service.is_admin(auth.access_token(), auth.secure_key()).await?;
    if !is_admin {
        return Ok(Json(ApiResponse::err_with_data("请输入管理密码", serde_json::Value::String("NEED_SECURE_KEY".to_string()))));
    }
    let username = state.book_source_service.get_default_owner().await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "username": username }))))
}

pub async fn login_book_source(State(state): State<AppState>, auth: AuthContext, Json(param): Json<BookSourceUrlParam>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let url = param.book_source_url.ok_or_else(|| AppError::BadRequest("bookSourceUrl required".to_string()))?;
    let source = state.book_source_service.get(&user_ns, &url).await?
        .ok_or_else(|| AppError::NotFound("bookSource not found".to_string()))?;
    let result = state.book_service.login_book_source(&source).await?;
    Ok(Json(ApiResponse::ok(result)))
}

#[derive(Debug, Deserialize)]
pub struct BookSourceProxyParam {
    #[serde(rename = "bookSourceUrl")]
    book_source_url: Option<String>,
    url: Option<String>,
}

pub async fn book_source_proxy(
    State(state): State<AppState>,
    auth: AuthContext,
    method: Method,
    headers: HeaderMap,
    Query(q): Query<BookSourceProxyParam>,
    body: Bytes,
) -> Result<Response, AppError> {
    let user_ns = state
        .user_service
        .resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns())
        .await
        .map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;

    let source_url = q
        .book_source_url
        .ok_or_else(|| AppError::BadRequest("bookSourceUrl required".to_string()))?;
    let raw_target_url = q
        .url
        .ok_or_else(|| AppError::BadRequest("url required".to_string()))?;

    let source = state
        .book_source_service
        .get(&user_ns, &source_url)
        .await?
        .ok_or_else(|| AppError::NotFound("bookSource not found".to_string()))?;

    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        state.book_service.set_source_cookie(&user_ns, &source.book_source_url, cookie).await;
    }

    let target_url = resolve_proxy_target_url(&raw_target_url, &source.book_source_url)?;
    let upstream_referer = extract_upstream_referer(&headers);
    let response = forward_book_source_request(
        &state,
        &source,
        auth.access_token(),
        &method,
        &headers,
        &target_url,
        upstream_referer.as_deref(),
        body,
    )
    .await?;

    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct BookSourceClientLogParam {
    message: Option<String>,
    source: Option<String>,
    lineno: Option<i64>,
    colno: Option<i64>,
    stack: Option<String>,
}

pub async fn book_source_client_log(
    Query(q): Query<BookSourceClientLogParam>,
) -> Json<ApiResponse<serde_json::Value>> {
    tracing::warn!(
        "bookSourceProxy client error: source={} line={} col={} message={} stack={}",
        q.source.as_deref().unwrap_or_default(),
        q.lineno.unwrap_or_default(),
        q.colno.unwrap_or_default(),
        q.message.as_deref().unwrap_or_default(),
        q.stack.as_deref().unwrap_or_default()
    );
    Json(ApiResponse::ok(serde_json::json!({ "logged": true })))
}

pub async fn delete_book_source(State(state): State<AppState>, auth: AuthContext, Json(param): Json<BookSourceUrlParam>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_ns = state.user_service.resolve_user_ns_with_override(auth.access_token(), auth.secure_key(), auth.user_ns()).await.map_err(|_| AppError::BadRequest("NEED_LOGIN".to_string()))?;
    let url = param.book_source_url.ok_or_else(|| AppError::BadRequest("bookSourceUrl required".to_string()))?;
    state.book_source_service.delete(&user_ns, &url).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted": true}))))
}

fn resolve_proxy_target_url(raw_target_url: &str, book_source_url: &str) -> Result<String, AppError> {
    let repaired = repair_encoded_url(raw_target_url);
    if let Ok(url) = Url::parse(&repaired) {
        return Ok(url.to_string());
    }

    let base = normalize_source_url(book_source_url);
    let base = Url::parse(&base).map_err(|e| AppError::BadRequest(format!("invalid bookSourceUrl: {}", e)))?;
    base.join(&repaired)
        .map(|u| u.to_string())
        .map_err(|e| AppError::BadRequest(format!("invalid proxy target url: {}", e)))
}

fn extract_upstream_referer(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::REFERER)?.to_str().ok()?;
    let referer = Url::parse(raw).ok()?;
    let params: std::collections::HashMap<String, String> =
        referer.query_pairs().into_owned().collect();
    params.get("url").cloned().map(|v| repair_encoded_url(&v))
}

async fn forward_book_source_request(
    state: &AppState,
    source: &BookSource,
    access_token: Option<&str>,
    method: &Method,
    headers: &HeaderMap,
    target_url: &str,
    upstream_referer: Option<&str>,
    body: Bytes,
) -> Result<Response, AppError> {
    let client = state.book_service.http_client();
    let req_method = match *method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        _ => return Err(AppError::BadRequest("unsupported proxy method".to_string())),
    };

    let mut builder = client.request(req_method, target_url);

    if let Some(header_str) = &source.header {
        if let Ok(source_headers) = serde_json::from_str::<std::collections::HashMap<String, String>>(header_str) {
            for (k, v) in source_headers {
                builder = builder.header(k, v);
            }
        }
    }

    let mut has_content_type = false;
    let mut has_x_requested_with = false;
    for (name, value) in headers.iter() {
        if should_forward_request_header(name.as_str()) {
            if name.as_str().eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            if name.as_str().eq_ignore_ascii_case("x-requested-with") {
                has_x_requested_with = true;
            }
            builder = builder.header(name, value.clone());
        }
    }

    if let Some(cookie) = headers.get(header::COOKIE) {
        builder = builder.header(header::COOKIE, cookie.clone());
    }

    let referer_value = upstream_referer.unwrap_or(target_url);
    builder = builder.header(header::REFERER, referer_value);
    if let Ok(url) = Url::parse(referer_value) {
        let origin = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
        builder = builder.header(header::ORIGIN, origin);
    }

    if method == Method::POST && !has_content_type {
        builder = builder.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded; charset=UTF-8");
    }
    if is_ajax_api_target(target_url) && !has_x_requested_with {
        builder = builder.header("X-Requested-With", "XMLHttpRequest");
    }

    if method == Method::POST {
        builder = builder.body(body.clone());
    }

    tracing::info!(
        "bookSourceProxy upstream request: method={} target={} referer={} body_len={}",
        method,
        target_url,
        referer_value,
        body.len()
    );
    let upstream = builder.send().await.map_err(AppError::Http)?;
    let status = upstream.status();
    let final_url = upstream.url().to_string();
    let content_type = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let upstream_set_cookies: Vec<String> = upstream
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
        .collect();
    let bytes = upstream.bytes().await.map_err(AppError::Http)?;
    tracing::info!(
        "bookSourceProxy upstream response: method={} target={} status={} final_url={}",
        method,
        target_url,
        status,
        final_url
    );
    if is_ajax_api_target(target_url) {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(500)]).replace('\n', "\\n");
        tracing::info!(
            "bookSourceProxy upstream api body: target={} status={} preview={}",
            target_url,
            status,
            preview
        );
    }
    if status.is_client_error() || status.is_server_error() {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(400)]).replace('\n', "\\n");
        tracing::warn!(
            "bookSourceProxy upstream error body: target={} status={} preview={}",
            target_url,
            status,
            preview
        );
    }

    let mut response_headers = HeaderMap::new();
    if let Some(ct) = content_type.as_deref() {
        if let Ok(value) = HeaderValue::from_str(ct) {
            response_headers.insert(header::CONTENT_TYPE, value);
        }
    }
    for cookie in upstream_set_cookies {
        if let Some(rewritten) = rewrite_set_cookie_for_proxy(&cookie) {
            if let Ok(value) = HeaderValue::from_str(&rewritten) {
                response_headers.append(header::SET_COOKIE, value);
            }
        }
    }
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));

    let body = if is_html_response(content_type.as_deref(), &bytes) {
        let text = String::from_utf8_lossy(&bytes).to_string();
        rewrite_login_html(
            &text,
            &final_url,
            &source.book_source_url,
            access_token,
        )
        .into_bytes()
    } else {
        bytes.to_vec()
    };

    Ok((status, response_headers, body).into_response())
}

fn should_forward_request_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host"
            | "content-length"
            | "authorization"
            | "referer"
            | "origin"
            | "connection"
    )
}

fn is_ajax_api_target(target_url: &str) -> bool {
    if let Ok(url) = Url::parse(target_url) {
        let path = url.path().to_ascii_lowercase();
        return path.ends_with("_api") || path.contains("/api/");
    }
    false
}

fn is_html_response(content_type: Option<&str>, body: &[u8]) -> bool {
    if let Some(ct) = content_type {
        if ct.to_ascii_lowercase().contains("text/html") {
            return true;
        }
    }
    let prefix = String::from_utf8_lossy(&body[..body.len().min(256)]).to_ascii_lowercase();
    prefix.contains("<html") || prefix.contains("<!doctype html")
}

fn rewrite_login_html(
    html: &str,
    upstream_url: &str,
    book_source_url: &str,
    access_token: Option<&str>,
) -> String {
    let base_href = html_escape_attr(upstream_url);
    let proxy_script = build_proxy_script(upstream_url, book_source_url, access_token);
    let mut output = if html.contains("<head") {
        html.replace(
            "</head>",
            &format!(r#"<base href="{base_href}">{proxy_script}</head>"#),
        )
    } else {
        format!(
            r#"<!DOCTYPE html><html><head><base href="{base_href}">{proxy_script}</head><body>{html}</body></html>"#
        )
    };

    output = rewrite_proxy_actions(&output, upstream_url, book_source_url, access_token);
    output = rewrite_script_root_relative_urls(&output, upstream_url, book_source_url, access_token);
    output
}

fn build_proxy_script(upstream_url: &str, book_source_url: &str, access_token: Option<&str>) -> String {
    let upstream_json = serde_json::to_string(upstream_url).unwrap_or_else(|_| "\"\"".to_string());
    let source_json = serde_json::to_string(book_source_url).unwrap_or_else(|_| "\"\"".to_string());
    let token_json = serde_json::to_string(access_token.unwrap_or("")).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"<script>
(function() {{
  const upstreamBase = {upstream_json};
  const bookSourceUrl = {source_json};
  const accessToken = {token_json};
  const proxyPath = "/reader3/bookSourceProxy";
  const alreadyProxyPattern = /^\/reader3\/bookSourceProxy(?:\?|$)/i;
  const skipPattern = /^(#|javascript:|data:|mailto:|tel:)/i;
  function toAbsolute(url) {{
    try {{ return new URL(url, upstreamBase).href; }} catch (_e) {{ return url; }}
  }}
  function toProxy(url) {{
    if (!url || skipPattern.test(url) || alreadyProxyPattern.test(url)) return url;
    const absolute = toAbsolute(url);
    if (String(absolute).indexOf("/reader3/bookSourceProxy?") !== -1) return absolute;
    const params = new URLSearchParams();
    if (accessToken) params.set("accessToken", accessToken);
    params.set("bookSourceUrl", bookSourceUrl);
    params.set("url", absolute);
    return proxyPath + "?" + params.toString();
  }}
  window.__readerBookSourceProxy = {{ toProxy, upstreamBase }};
  const rawFetch = window.fetch ? window.fetch.bind(window) : null;
  if (rawFetch) {{
    window.fetch = function(input, init) {{
      try {{
        if (input instanceof Request) {{
          return rawFetch(new Request(toProxy(input.url), input), init);
        }}
        return rawFetch(toProxy(String(input)), init);
      }} catch (_e) {{
        return rawFetch(input, init);
      }}
    }};
  }}
  const rawOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function(method, url) {{
    arguments[1] = toProxy(String(url));
    return rawOpen.apply(this, arguments);
  }};
  document.addEventListener("submit", function(event) {{
    const form = event.target;
    if (!(form instanceof HTMLFormElement)) return;
    const action = form.getAttribute("action") || upstreamBase;
    form.setAttribute("action", toProxy(action));
  }}, true);
  document.addEventListener("click", function(event) {{
    const anchor = event.target && event.target.closest ? event.target.closest("a[href]") : null;
    if (!anchor) return;
    const href = anchor.getAttribute("href");
    if (!href || skipPattern.test(href)) return;
    anchor.setAttribute("href", toProxy(href));
  }}, true);
  function reportClientError(payload) {{
    try {{
      const params = new URLSearchParams();
      Object.entries(payload || {{}}).forEach(function(entry) {{
        const key = entry[0];
        const value = entry[1];
        if (value !== undefined && value !== null && value !== "") {{
          params.set(key, String(value));
        }}
      }});
      const url = "/reader3/bookSourceClientLog?" + params.toString();
      if (navigator.sendBeacon) {{
        navigator.sendBeacon(url);
      }} else if (rawFetch) {{
        rawFetch(url, {{ method: "POST" }});
      }}
    }} catch (_e) {{}}
  }}
  window.addEventListener("error", function(event) {{
    reportClientError({{
      message: event.message || "window error",
      source: event.filename || "",
      lineno: event.lineno || 0,
      colno: event.colno || 0,
      stack: event.error && event.error.stack ? event.error.stack : ""
    }});
  }});
  window.addEventListener("unhandledrejection", function(event) {{
    const reason = event.reason;
    reportClientError({{
      message: reason && reason.message ? reason.message : String(reason || "unhandled rejection"),
      stack: reason && reason.stack ? reason.stack : ""
    }});
  }});
}})();
</script>"#
    )
}

fn rewrite_proxy_actions(
    html: &str,
    upstream_url: &str,
    book_source_url: &str,
    access_token: Option<&str>,
) -> String {
    let tag_re = Regex::new(r#"(?is)<[^>]+>"#).unwrap();
    let double_quoted = Regex::new(r#"(?i)\b(action|href|src)\s*=\s*"([^"]+)""#).unwrap();
    let single_quoted = Regex::new(r#"(?i)\b(action|href|src)\s*=\s*'([^']+)'"#).unwrap();

    tag_re
        .replace_all(html, |tag_caps: &Captures| {
            let tag = tag_caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let output = double_quoted.replace_all(tag, |caps: &Captures| {
                rewrite_proxy_attr(&caps, upstream_url, book_source_url, access_token, "\"")
            });
            single_quoted
                .replace_all(&output, |caps: &Captures| {
                    rewrite_proxy_attr(&caps, upstream_url, book_source_url, access_token, "'")
                })
                .into_owned()
        })
        .into_owned()
}

fn rewrite_proxy_attr(
    caps: &Captures,
    upstream_url: &str,
    book_source_url: &str,
    access_token: Option<&str>,
    quote: &str,
) -> String {
    let attr = caps.get(1).map(|m| m.as_str()).unwrap_or("href");
    let value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    let proxied = build_proxy_url(value, upstream_url, book_source_url, access_token)
        .unwrap_or_else(|| value.to_string());
    format!(r#"{attr}={quote}{proxied}{quote}"#)
}

fn build_proxy_url(
    raw_value: &str,
    upstream_url: &str,
    book_source_url: &str,
    access_token: Option<&str>,
) -> Option<String> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("javascript:")
        || trimmed.starts_with("data:")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
        || trimmed.starts_with("/reader3/bookSourceProxy")
    {
        return None;
    }

    let absolute = Url::parse(trimmed)
        .or_else(|_| Url::parse(upstream_url).and_then(|base| base.join(trimmed)))
        .ok()?;
    let mut params = vec![
        format!("bookSourceUrl={}", urlencoding::encode(book_source_url)),
        format!("url={}", urlencoding::encode(absolute.as_str())),
    ];
    if let Some(token) = access_token.filter(|v| !v.is_empty()) {
        params.push(format!("accessToken={}", urlencoding::encode(token)));
    }
    Some(format!("/reader3/bookSourceProxy?{}", params.join("&")))
}

fn rewrite_script_root_relative_urls(
    html: &str,
    upstream_url: &str,
    book_source_url: &str,
    access_token: Option<&str>,
) -> String {
    let script_re = Regex::new(r#"(?is)<script\b[^>]*>.*?</script>"#).unwrap();
    let double_quoted = Regex::new(r#""(/[^"\\\s<]*)""#).unwrap();
    let single_quoted = Regex::new(r#"'(/[^'\\\s<]*)'"#).unwrap();

    script_re
        .replace_all(html, |script_caps: &Captures| {
            let script = script_caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let output = double_quoted.replace_all(script, |caps: &Captures| {
                rewrite_script_url_literal(&caps, upstream_url, book_source_url, access_token, "\"")
            });
            single_quoted
                .replace_all(&output, |caps: &Captures| {
                    rewrite_script_url_literal(&caps, upstream_url, book_source_url, access_token, "'")
                })
                .into_owned()
        })
        .into_owned()
}

fn rewrite_script_url_literal(
    caps: &Captures,
    upstream_url: &str,
    book_source_url: &str,
    access_token: Option<&str>,
    quote: &str,
) -> String {
    let raw_value = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let proxied = build_proxy_url(raw_value, upstream_url, book_source_url, access_token)
        .unwrap_or_else(|| raw_value.to_string());
    format!("{quote}{proxied}{quote}")
}

fn html_escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn rewrite_set_cookie_for_proxy(raw: &str) -> Option<String> {
    let mut parts = raw
        .split(';')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty());
    let first = parts.next()?;
    if !first.contains('=') {
        return None;
    }

    let mut attrs = vec![first.to_string(), "Path=/reader3/bookSourceProxy".to_string()];
    for attr in parts {
        let lower = attr.to_ascii_lowercase();
        if lower.starts_with("domain=")
            || lower.starts_with("path=")
            || lower == "secure"
        {
            continue;
        }
        attrs.push(attr.to_string());
    }
    Some(attrs.join("; "))
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
