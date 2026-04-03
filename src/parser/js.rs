use rquickjs::{Context, Runtime, Value, Object};
use rquickjs::function::Func;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{Local, TimeZone};
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::Value as JsonValue;
use uuid::Uuid;
use base64::Engine;
use crate::util::text::{apply_regex_replace, strip_whitespace};
use crate::util::hash::md5_hex;

static JS_KV: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static JS_HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .expect("failed to build JS HTTP client")
});
static JS_DEVICE_ID: Lazy<String> = Lazy::new(|| {
    let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(existing) = map.get("__device_id") {
        return existing.clone();
    }
    let generated = Uuid::new_v4().to_string();
    map.insert("__device_id".to_string(), generated.clone());
    generated
});

pub fn eval_js(script: &str, input: &str, base_url: &str) -> anyhow::Result<String> {
    eval_js_inner(script, Some(input), Some(base_url), None, None)
}

pub fn eval_js_search_with_source(script: &str, key: &str, page: i32, source_key: &str) -> anyhow::Result<String> {
    eval_js_inner_with_source(script, None, None, Some(key), Some(page), Some(source_key))
}

fn eval_js_inner(script: &str, input: Option<&str>, base_url: Option<&str>, key: Option<&str>, page: Option<i32>) -> anyhow::Result<String> {
    eval_js_inner_with_source(script, input, base_url, key, page, None)
}

fn eval_js_inner_with_source(script: &str, input: Option<&str>, base_url: Option<&str>, key: Option<&str>, page: Option<i32>, source_key: Option<&str>) -> anyhow::Result<String> {
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;
    ctx.with(|ctx| {
        let globals = ctx.globals();
        if let Some(input) = input { globals.set("input", input)?; }
        if let Some(base_url) = base_url { globals.set("base_url", base_url)?; }
        if let Some(key) = key { globals.set("key", key)?; }
        if let Some(page) = page { globals.set("page", page)?; }

        // Default url variable for Legado compatibility
        globals.set("url", "")?;

        // Stubs for Legado compatibility
        let source_key_val = source_key.unwrap_or("").to_string();
        let source_obj = Object::new(ctx.clone())?;
        let sk_clone = source_key_val.clone();
        source_obj.set("key", source_key_val)?;
        source_obj.set("getKey", Func::new(move || sk_clone.clone()))?;
        globals.set("source", source_obj)?;

        let cookie_obj = Object::new(ctx.clone())?;
        cookie_obj.set("removeCookie", Func::new(|_key: String| -> String { "".to_string() }))?;
        globals.set("cookie", cookie_obj)?;

        let java_obj = Object::new(ctx.clone())?;
        java_obj.set("ajax", Func::new(|spec: String| -> String {
            java_ajax(&spec).unwrap_or_default()
        }))?;
        java_obj.set("md5Encode", Func::new(|input: String| -> String {
            md5_hex(&input)
        }))?;
        java_obj.set("timeFormat", Func::new(|timestamp: i64| -> String {
            java_time_format(timestamp)
        }))?;
        java_obj.set("androidId", Func::new(|| -> String {
            JS_DEVICE_ID.clone()
        }))?;
        java_obj.set("deviceID", Func::new(|| -> String {
            JS_DEVICE_ID.clone()
        }))?;
        java_obj.set("get", Func::new(|url: String| -> String {
            java_request_simple("GET", &url, None).unwrap_or_default()
        }))?;
        java_obj.set("post", Func::new(|url: String, body: String| -> String {
            java_request_simple("POST", &url, Some(body)).unwrap_or_default()
        }))?;
        java_obj.set("put", Func::new(|url: String, body: String| -> String {
            java_request_simple("PUT", &url, Some(body)).unwrap_or_default()
        }))?;
        java_obj.set("base64Encode", Func::new(|input: String| -> String {
            base64::engine::general_purpose::STANDARD.encode(input)
        }))?;
        java_obj.set("base64Decode", Func::new(|input: String| -> String {
            base64::engine::general_purpose::STANDARD
                .decode(input)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_default()
        }))?;
        java_obj.set("encodeURIComponent", Func::new(|input: String| -> String {
            urlencoding::encode(&input).into_owned()
        }))?;
        java_obj.set("decodeURIComponent", Func::new(|input: String| -> String {
            urlencoding::decode(&input).map(|s| s.into_owned()).unwrap_or_default()
        }))?;
        java_obj.set("encodeURI", Func::new(|input: String| -> String {
            urlencoding::encode(&input).into_owned()
        }))?;
        java_obj.set("decodeURI", Func::new(|input: String| -> String {
            urlencoding::decode(&input).map(|s| s.into_owned()).unwrap_or_default()
        }))?;
        java_obj.set("now", Func::new(|| -> i64 {
            chrono::Utc::now().timestamp_millis()
        }))?;
        java_obj.set("uuid", Func::new(|| -> String {
            Uuid::new_v4().to_string()
        }))?;
        globals.set("java", java_obj)?;

        globals.set("kv_get", Func::new(|key: String| -> Option<String> {
            let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
            map.get(&key).cloned()
        }))?;
        globals.set("kv_put", Func::new(|key: String, val: String| -> bool {
            let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
            map.insert(key, val);
            true
        }))?;
        globals.set("regex_replace", Func::new(|input: String, pattern: String, replace: String| -> String {
            apply_regex_replace(&input, &pattern, &replace)
        }))?;
        globals.set("strip_ws", Func::new(|input: String| -> String {
            strip_whitespace(&input)
        }))?;

        let v: Value = match ctx.eval(script) {
            Ok(v) => v,
            Err(e) => {
                if let Some(exception) = ctx.catch().into_exception() {
                    return Err(anyhow::anyhow!("JS Exception: {:?}", exception));
                }
                return Err(e.into());
            }
        };

        let result = if v.is_null() || v.is_undefined() {
            String::new()
        } else if let Some(s) = v.clone().into_string() {
            s.to_string().unwrap_or_default()
        } else {
            match ctx.json_stringify(v) {
                Ok(Some(json)) => json.to_string().unwrap_or_default(),
                _ => String::new(),
            }
        };
        Ok(result)
    })
}

fn java_time_format(timestamp: i64) -> String {
    let secs = if timestamp > 1_000_000_000_000 {
        timestamp / 1000
    } else {
        timestamp
    };
    match Local.timestamp_opt(secs, 0).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => String::new(),
    }
}

fn java_ajax(spec: &str) -> anyhow::Result<String> {
    let (url, options) = split_ajax_spec(spec);
    if url.trim().is_empty() {
        return Ok(String::new());
    }

    let options_json = options
        .and_then(|raw| serde_json::from_str::<JsonValue>(raw).ok())
        .unwrap_or(JsonValue::Null);

    let method = options_json
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_uppercase();
    let method = Method::from_bytes(method.as_bytes()).unwrap_or(Method::GET);

    let mut req = JS_HTTP_CLIENT.request(method, url.trim());

    if let Some(headers) = options_json.get("headers").and_then(|v| v.as_object()) {
        for (key, value) in headers {
            if let Some(value) = value.as_str() {
                req = req.header(key, value);
            } else if !value.is_null() {
                req = req.header(key, value.to_string());
            }
        }
    }

    if let Some(body) = options_json.get("body") {
        if let Some(body) = body.as_str() {
            req = req.body(body.to_string());
        } else if !body.is_null() {
            req = req.body(body.to_string());
        }
    }

    let response = req.send()?;
    Ok(response.text().unwrap_or_default())
}

fn java_request_simple(method: &str, url: &str, body: Option<String>) -> anyhow::Result<String> {
    let method = Method::from_bytes(method.as_bytes()).unwrap_or(Method::GET);
    let mut req = JS_HTTP_CLIENT.request(method, url.trim());
    if let Some(body) = body {
        req = req.body(body);
    }
    let response = req.send()?;
    Ok(response.text().unwrap_or_default())
}

fn split_ajax_spec(spec: &str) -> (&str, Option<&str>) {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;

    for (idx, ch) in spec.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                escaped = true;
            }
            '"' | '\'' if in_string && ch == quote => {
                in_string = false;
                quote = '\0';
            }
            '"' | '\'' if !in_string => {
                in_string = true;
                quote = ch;
            }
            '{' | '[' if !in_string => depth += 1,
            '}' | ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let left = &spec[..idx];
                let right = &spec[idx + ch.len_utf8()..];
                return (left, Some(right.trim()));
            }
            _ => {}
        }
    }

    (spec, None)
}
