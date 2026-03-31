use rquickjs::{Context, Runtime, Value, Object};
use rquickjs::function::Func;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::util::text::{apply_regex_replace, strip_whitespace};

static JS_KV: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

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
