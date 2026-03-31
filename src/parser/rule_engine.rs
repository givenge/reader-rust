use crate::model::{book::Book, book_chapter::BookChapter, book_source::BookSource, search::SearchBook};
use crate::model::rule::{SearchRule, BookInfoRule, TocRule};
use crate::parser::{html, jsonpath, js::eval_js};
use crate::util::text::apply_regex_replace;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct RuleEngine;

#[derive(Debug, Clone, PartialEq)]
enum ParseMode {
    Css,      // CSS selector
    XPath,    // XPath expression
    JsonPath, // JSONPath expression
    Regex,    // Regex pattern
    Js,       // JavaScript
}

impl RuleEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    /// Detect the parsing mode from the rule string
    fn detect_mode(&self, rule: &str, content: &str) -> ParseMode {
        let rule = rule.trim();

        // Explicit mode forcing
        if rule.starts_with("@css:") || rule.starts_with("@CSS:") {
            return ParseMode::Css;
        }
        if rule.starts_with("@xpath:") || rule.starts_with("@XPath:") || rule.starts_with("@XPATH:") {
            return ParseMode::XPath;
        }
        if rule.starts_with("@json:") || rule.starts_with("@Json:") || rule.starts_with("@JSON:") {
            return ParseMode::JsonPath;
        }
        if rule.starts_with("@regex:") || rule.starts_with("@Regex:") {
            return ParseMode::Regex;
        }
        if rule.starts_with("js:") || rule.starts_with("@js:") {
            return ParseMode::Js;
        }

        // Auto-detect from rule prefix
        if rule.starts_with('/') || rule.starts_with("./") {
            return ParseMode::XPath;
        }
        if rule.starts_with("$.") || rule.starts_with("$[") {
            return ParseMode::JsonPath;
        }
        if rule.starts_with(':') {
            return ParseMode::Regex;
        }

        // Auto-detect from content
        let content_trimmed = content.trim();
        if content_trimmed.starts_with('{') || content_trimmed.starts_with('[') {
            // Likely JSON content
            if rule.starts_with("$.") || rule.starts_with("$[") {
                return ParseMode::JsonPath;
            }
            // Try to parse as JSON
            if serde_json::from_str::<Value>(content_trimmed).is_ok() {
                return ParseMode::JsonPath;
            }
        }

        // Default to CSS
        ParseMode::Css
    }

    /// Strip mode prefix from rule
    fn strip_mode_prefix<'a>(&self, rule: &'a str) -> &'a str {
        let rule = rule.trim();
        for prefix in ["@css:", "@CSS:", "@xpath:", "@XPath:", "@XPATH:", "@json:", "@Json:", "@JSON:", "@regex:", "@Regex:", "@js:"] {
            if rule.starts_with(prefix) {
                return &rule[prefix.len()..];
            }
        }
        rule
    }

    pub fn search_books(&self, source: &BookSource, body: &str, base_url: &str) -> Vec<SearchBook> {
        let rule = source.rule_search.clone().unwrap_or_default();
        let mode = self.detect_mode(rule.book_list.as_deref().unwrap_or(""), body);
        match mode {
            ParseMode::JsonPath => self.search_books_json(source, body, base_url, &rule),
            ParseMode::XPath => self.search_books_xpath(source, body, base_url, &rule),
            _ => self.search_books_html(source, body, base_url, &rule),
        }
    }

    pub fn explore_books(&self, source: &BookSource, body: &str, base_url: &str) -> Vec<SearchBook> {
        let rule = source.rule_explore.clone().unwrap_or_else(|| source.rule_search.clone().unwrap_or_default());
        let mode = self.detect_mode(rule.book_list.as_deref().unwrap_or(""), body);
        match mode {
            ParseMode::JsonPath => self.search_books_json(source, body, base_url, &rule),
            ParseMode::XPath => self.search_books_xpath(source, body, base_url, &rule),
            _ => self.search_books_html(source, body, base_url, &rule),
        }
    }

    pub fn book_info(&self, source: &BookSource, body: &str, base_url: &str, book_url: &str) -> Book {
        let rule = source.rule_book_info.clone().unwrap_or_default();
        let mut context = HashMap::new();

        let mode = self.detect_mode(rule.name.as_deref().unwrap_or(""), body);
        match mode {
            ParseMode::JsonPath => {
                if let Ok(v) = serde_json::from_str::<Value>(body) {
                    return parse_book_info_json(source, &v, base_url, &rule, book_url, &mut context);
                }
            }
            ParseMode::XPath => {
                return parse_book_info_xpath(source, body, base_url, &rule, book_url, &mut context);
            }
            _ => {}
        }
        parse_book_info_html(source, body, base_url, &rule, book_url, &mut context)
    }

    pub fn chapter_list(&self, source: &BookSource, body: &str, base_url: &str) -> (Vec<BookChapter>, Vec<String>) {
        let rule = source.rule_toc.clone().unwrap_or_default();
        let mut context = HashMap::new();

        let mode = self.detect_mode(rule.chapter_list.as_deref().unwrap_or(""), body);
        match mode {
            ParseMode::JsonPath => parse_chapter_list_json(body, base_url, &rule, &mut context),
            ParseMode::XPath => parse_chapter_list_xpath(body, base_url, &rule, &mut context),
            _ => parse_chapter_list_html(body, base_url, &rule, &mut context),
        }
    }

    pub fn content(&self, source: &BookSource, body: &str, base_url: &str) -> String {
        let rule = source.rule_content.clone().unwrap_or_default();

        if let Some(content_rule) = rule.content.clone() {
            // Handle JavaScript content rules
            if content_rule.trim().starts_with("js:") || content_rule.trim().starts_with("@js:") {
                let script = self.strip_mode_prefix(&content_rule);
                if let Ok(res) = eval_js(script, body, base_url) {
                    return res;
                }
            }

            // Handle inline JS {{...}}
            let content_rule = self.process_inline_js(&content_rule, body, base_url);

            let mode = self.detect_mode(&content_rule, body);
            let mut content = match mode {
                ParseMode::JsonPath => {
                    if let Ok(v) = serde_json::from_str::<Value>(body) {
                        jsonpath::jsonpath_first_string(&v, self.strip_mode_prefix(&content_rule)).unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                ParseMode::XPath => {
                    html::select_xpath(body, self.strip_mode_prefix(&content_rule)).first().cloned().unwrap_or_default()
                }
                _ => {
                    let doc = html::parse_document(body);
                    let result = html::select_all_text(&doc, self.strip_mode_prefix(&content_rule));
                    result.unwrap_or_default()
                }
            };

            // Apply regex replacement
            if let Some(replace) = rule.replace_regex.as_deref() {
                content = apply_legado_regex(&content, replace);
            }

            return content;
        }

        String::new()
    }

    /// Process inline JavaScript {{...}} in rules
    fn process_inline_js(&self, rule: &str, body: &str, base_url: &str) -> String {
        let mut result = rule.to_string();

        // Find all {{...}} blocks and evaluate them
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        for cap in re.captures_iter(rule) {
            if let Some(js_code) = cap.get(1) {
                if let Ok(js_result) = eval_js(js_code.as_str(), body, base_url) {
                    result = result.replace(cap.get(0).unwrap().as_str(), &js_result);
                }
            }
        }

        result
    }

    /// Get the next content page URL if pagination exists
    pub fn next_content_url(&self, source: &BookSource, body: &str, base_url: &str) -> Option<String> {
        let rule = source.rule_content.clone().unwrap_or_default();
        let next_rule = rule.next_content_url.as_deref()?;
        if next_rule.is_empty() {
            return None;
        }

        let mode = self.detect_mode(next_rule, body);
        let next_url = match mode {
            ParseMode::JsonPath => {
                if let Ok(v) = serde_json::from_str::<Value>(body) {
                    jsonpath::jsonpath_first_string(&v, self.strip_mode_prefix(next_rule))
                } else {
                    None
                }
            }
            ParseMode::XPath => {
                html::select_xpath(body, self.strip_mode_prefix(next_rule)).first().cloned()
            }
            _ => {
                let doc = html::parse_document(body);
                html::select_text(&doc, self.strip_mode_prefix(next_rule))
            }
        };

        if next_url.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            return None;
        }

        let next_url = next_url?;
        Some(resolve_url(base_url, &next_url))
    }

    fn search_books_html(&self, source: &BookSource, body: &str, base_url: &str, rule: &SearchRule) -> Vec<SearchBook> {
        let list_sel = match &rule.book_list {
            Some(r) => r,
            None => return vec![],
        };
        let doc = html::parse_document(body);
        let items = html::select_list(&doc, self.strip_mode_prefix(list_sel));
        let mut out = Vec::with_capacity(items.len());

        for el in items {
            let name = rule.name.as_ref().and_then(|r| eval_field_html(r, &el, base_url)).unwrap_or_default();
            let author = rule.author.as_ref().and_then(|r| eval_field_html(r, &el, base_url)).unwrap_or_default();
            let book_url = rule.book_url.as_ref().and_then(|r| eval_field_html(r, &el, base_url)).unwrap_or_default();
            let cover_url = rule.cover_url.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let intro = rule.intro.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let kind = rule.kind.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let last_chapter = rule.last_chapter.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let update_time = rule.update_time.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let book_url_abs = resolve_url(base_url, &book_url);
            let cover_url_abs = cover_url.map(|u| resolve_url(base_url, &u));
            out.push(SearchBook {
                name,
                author,
                book_url: book_url_abs,
                origin: source.book_source_url.clone(),
                cover_url: cover_url_abs,
                intro,
                kind,
                last_chapter,
                update_time,
                book_source_urls: None,
            });
        }
        out
    }

    fn search_books_xpath(&self, source: &BookSource, body: &str, base_url: &str, rule: &SearchRule) -> Vec<SearchBook> {
        // For XPath, we need to get list of items and then extract fields
        // This is a simplified implementation
        // For now, return empty as full XPath element selection needs more work
        // Fall back to HTML parsing
        self.search_books_html(source, body, base_url, rule)
    }

    fn search_books_json(&self, source: &BookSource, body: &str, base_url: &str, rule: &SearchRule) -> Vec<SearchBook> {
        let v: Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(_) => return vec![],
        };
        let list_rule = rule.book_list.as_deref().unwrap_or("");
        let items = jsonpath::jsonpath_query(&v, self.strip_mode_prefix(list_rule));
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let name = eval_field_json(rule.name.as_deref().unwrap_or(""), &item, base_url);
            let author = eval_field_json(rule.author.as_deref().unwrap_or(""), &item, base_url);
            let book_url = eval_field_json(rule.book_url.as_deref().unwrap_or(""), &item, base_url);
            let cover_url = eval_field_json(rule.cover_url.as_deref().unwrap_or(""), &item, base_url);
            let intro = eval_field_json(rule.intro.as_deref().unwrap_or(""), &item, base_url);
            let kind = eval_field_json(rule.kind.as_deref().unwrap_or(""), &item, base_url);
            let last_chapter = eval_field_json(rule.last_chapter.as_deref().unwrap_or(""), &item, base_url);
            let update_time = eval_field_json(rule.update_time.as_deref().unwrap_or(""), &item, base_url);
            out.push(SearchBook {
                name: name.unwrap_or_default(),
                author: author.unwrap_or_default(),
                book_url: book_url.unwrap_or_default(),
                origin: source.book_source_url.clone(),
                cover_url,
                intro,
                kind,
                last_chapter,
                update_time,
                book_source_urls: None,
            });
        }
        out
    }
}

fn parse_book_info_html(source: &BookSource, body: &str, base_url: &str, rule: &BookInfoRule, book_url: &str, ctx: &mut HashMap<String, String>) -> Book {
    let doc = html::parse_document(body);

    // Execute init rule if present
    if let Some(init) = &rule.init {
        let _ = eval_field_html_doc_with_ctx(init, &doc, base_url, ctx);
    }

    let name = rule.name.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).unwrap_or_default();
    let author = rule.author.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).unwrap_or_default();
    let intro = rule.intro.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let kind = rule.kind.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let last_chapter = rule.last_chapter.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let update_time = rule.update_time.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let cover_url = rule.cover_url.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).map(|u| resolve_url(base_url, &u));
    let word_count = rule.word_count.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let toc_url = rule.toc_url.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).map(|u| resolve_url(base_url, &u));

    let final_toc_url = toc_url.or_else(|| Some(book_url.to_string()));

    Book {
        name,
        author,
        book_url: book_url.to_string(),
        origin: source.book_source_url.clone(),
        origin_name: Some(source.book_source_name.clone()),
        cover_url,
        toc_url: final_toc_url,
        intro,
        latest_chapter_title: last_chapter,
        word_count,
        info_html: None,
        toc_html: None,
        kind,
        update_time,
        ..Default::default()
    }
}

fn parse_book_info_xpath(source: &BookSource, body: &str, base_url: &str, rule: &BookInfoRule, book_url: &str, ctx: &mut HashMap<String, String>) -> Book {
    // Simplified XPath implementation - fall back to HTML for now
    parse_book_info_html(source, body, base_url, rule, book_url, ctx)
}

fn parse_book_info_json(source: &BookSource, v: &Value, base_url: &str, rule: &BookInfoRule, book_url: &str, ctx: &mut HashMap<String, String>) -> Book {
    if let Some(init) = &rule.init {
        let _ = eval_field_json_with_ctx(init, v, base_url, ctx);
    }
    let name = eval_field_json_with_ctx(rule.name.as_deref().unwrap_or(""), v, base_url, ctx).unwrap_or_default();
    let author = eval_field_json_with_ctx(rule.author.as_deref().unwrap_or(""), v, base_url, ctx).unwrap_or_default();
    let intro = eval_field_json_with_ctx(rule.intro.as_deref().unwrap_or(""), v, base_url, ctx);
    let kind = eval_field_json_with_ctx(rule.kind.as_deref().unwrap_or(""), v, base_url, ctx);
    let last_chapter = eval_field_json_with_ctx(rule.last_chapter.as_deref().unwrap_or(""), v, base_url, ctx);
    let update_time = eval_field_json_with_ctx(rule.update_time.as_deref().unwrap_or(""), v, base_url, ctx);
    let cover_url = eval_field_json_with_ctx(rule.cover_url.as_deref().unwrap_or(""), v, base_url, ctx);
    let word_count = eval_field_json_with_ctx(rule.word_count.as_deref().unwrap_or(""), v, base_url, ctx);
    let toc_url = eval_field_json_with_ctx(rule.toc_url.as_deref().unwrap_or(""), v, base_url, ctx);
    Book {
        name,
        author,
        book_url: book_url.to_string(),
        origin: source.book_source_url.clone(),
        origin_name: Some(source.book_source_name.clone()),
        cover_url,
        toc_url: toc_url.or_else(|| Some(book_url.to_string())),
        intro,
        latest_chapter_title: last_chapter,
        word_count,
        info_html: None,
        toc_html: None,
        kind,
        update_time,
        ..Default::default()
    }
}

fn parse_chapter_list_html(body: &str, base_url: &str, rule: &TocRule, ctx: &mut HashMap<String, String>) -> (Vec<BookChapter>, Vec<String>) {
    let list_sel = match &rule.chapter_list {
        Some(r) => r,
        None => return (vec![], vec![]),
    };
    let doc = html::parse_document(body);

    // Execute init rule if present
    if let Some(init) = &rule.init {
        let _ = eval_field_html_doc_with_ctx(init, &doc, base_url, ctx);
    }

    let items = html::select_list(&doc, list_sel);

    // Use a set to deduplicate chapters by URL
    let mut seen_urls = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(items.len());

    for el in items {
        let title = rule.chapter_name.as_ref().and_then(|r| eval_field_html_with_ctx(r, &el, base_url, ctx)).unwrap_or_default();
        let url = rule.chapter_url.as_ref().and_then(|r| eval_field_html_with_ctx(r, &el, base_url, ctx)).unwrap_or_default();
        let url_abs = resolve_url(base_url, &url);

        // Skip duplicate chapters (same URL)
        if seen_urls.contains(&url_abs) {
            continue;
        }

        // Check if this element is inside #chapterlist (latest chapters section)
        // The #chapterlist div contains duplicate "latest chapters" that appear on every page
        let mut in_latest_div = false;
        for ancestor in el.ancestors() {
            if let Some(element) = ancestor.value().as_element() {
                if element.attr("id") == Some("chapterlist") {
                    in_latest_div = true;
                    break;
                }
            }
        }
        if in_latest_div {
            continue;
        }

        seen_urls.insert(url_abs.clone());

        out.push(BookChapter {
            title,
            url: url_abs,
            index: out.len() as i32,
        });
    }

    // Extract next_toc_url(s)
    let rule_str = rule.next_toc_url.as_deref().unwrap_or("");
    let raw_urls: Vec<String> = html::select_text_list(&doc, rule_str);
    let next_urls: Vec<String> = raw_urls
        .into_iter()
        .filter(|u| !u.is_empty())
        .map(|u| resolve_url(base_url, &u))
        .collect();

    (out, next_urls)
}

fn parse_chapter_list_xpath(body: &str, base_url: &str, rule: &TocRule, ctx: &mut HashMap<String, String>) -> (Vec<BookChapter>, Vec<String>) {
    // Fall back to HTML for now
    parse_chapter_list_html(body, base_url, rule, ctx)
}

fn parse_chapter_list_json(body: &str, base_url: &str, rule: &TocRule, ctx: &mut HashMap<String, String>) -> (Vec<BookChapter>, Vec<String>) {
    let v: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return (vec![], vec![]),
    };

    if let Some(init) = &rule.init {
        let _ = eval_field_json_with_ctx(init, &v, base_url, ctx);
    }

    let list_rule = rule.chapter_list.as_deref().unwrap_or("");
    let items = jsonpath::jsonpath_query(&v, list_rule);

    let mut seen_urls = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let title = eval_field_json_with_ctx(rule.chapter_name.as_deref().unwrap_or(""), &item, base_url, ctx).unwrap_or_default();
        let url = eval_field_json_with_ctx(rule.chapter_url.as_deref().unwrap_or(""), &item, base_url, ctx).unwrap_or_default();

        if seen_urls.contains(&url) {
            continue;
        }
        seen_urls.insert(url.clone());

        out.push(BookChapter { title, url, index: out.len() as i32 });
    }

    let next_urls: Vec<String> = rule.next_toc_url.as_ref()
        .map(|r| jsonpath::jsonpath_query(&v, r))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .filter(|u| !u.is_empty())
        .map(|u| resolve_url(base_url, &u))
        .collect();

    (out, next_urls)
}

fn pick_json_field(v: &Value, rule: Option<&str>) -> Option<String> {
    let rule = rule?;
    if rule.trim_start().starts_with('$') {
        return jsonpath::jsonpath_first_string(v, rule);
    }
    if let Some(obj) = v.as_object() {
        if let Some(val) = obj.get(rule) {
            return jsonpath::value_to_string(val);
        }
    }
    None
}

fn resolve_url(base: &str, url: &str) -> String {
    let url = strip_url_config(url);

    if url.is_empty() { return base.to_string(); }
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    if url.starts_with("//") {
        return format!("https:{}", url);
    }

    let base_url = match url::Url::parse(base) {
        Ok(u) => u,
        Err(_) => return url.to_string(),
    };

    if url.starts_with('/') {
        let mut out = base_url.clone();
        out.set_path(url);
        return out.to_string();
    }

    match base_url.join(url) {
        Ok(u) => u.to_string(),
        Err(_) => {
            let base = base.trim_end_matches('/');
            format!("{}/{}", base, url.trim_start_matches('/'))
        }
    }
}

fn strip_url_config(url: &str) -> &str {
    if let Some(idx) = url.find("##$##") {
        &url[..idx]
    } else if let Some(idx) = url.find(",{'webView'") {
        &url[..idx]
    } else if let Some(idx) = url.find(",{\"webView\"") {
        &url[..idx]
    } else {
        url
    }
}

fn extract_js(rule: &str) -> (&str, Option<&str>) {
    if let Some(idx) = rule.find("<js>") {
        if let Some(end_idx) = rule.rfind("</js>") {
            if end_idx > idx {
                let pure = rule[..idx].trim();
                let js = &rule[idx + 4..end_idx];
                return (pure, Some(js));
            }
        }
    }
    if let Some(idx) = rule.find("@js:") {
        let pure = rule[..idx].trim();
        let js = &rule[idx + 4..];
        return (pure, Some(js));
    }
    (rule, None)
}

fn eval_field_html(rule: &str, el: &scraper::ElementRef, base_url: &str) -> Option<String> {
    eval_field_html_with_ctx(rule, el, base_url, &mut HashMap::new())
}

fn eval_field_html_with_ctx(rule: &str, el: &scraper::ElementRef, base_url: &str, ctx: &mut HashMap<String, String>) -> Option<String> {
    // Handle mode forcing prefixes
    let rule = rule.trim();
    if rule.starts_with("@css:") {
        let pure = &rule[5..];
        return eval_field_html_with_ctx(pure, el, base_url, ctx);
    }
    if rule.starts_with("@xpath:") {
        // XPath from element - not directly supported, return None
        return None;
    }
    if rule.starts_with("@json:") {
        // JSON from element - not applicable
        return None;
    }

    // Handle @put/@get
    if let Some(res) = try_put_get_html(rule, el, base_url, ctx) {
        return Some(res);
    }

    let (pure_rule, regex_part) = split_legado_regex(rule);
    let (pure, js) = extract_js(&pure_rule);

    let mut text = if pure.is_empty() {
        "".to_string()
    } else {
        html::select_text_from_element(el, pure).unwrap_or_default()
    };

    if let Some(script) = js {
        if let Ok(res) = eval_js(script, &text, base_url) {
            text = res;
        }
    }

    if let Some(reg) = regex_part {
        text = apply_legado_regex(&text, reg);
    }

    if text.is_empty() { None } else { Some(text) }
}

fn eval_field_html_doc_with_ctx(rule: &str, doc: &scraper::Html, base_url: &str, ctx: &mut HashMap<String, String>) -> Option<String> {
    // Handle mode forcing prefixes
    let rule = rule.trim();
    if rule.starts_with("@css:") {
        let pure = &rule[5..];
        return eval_field_html_doc_with_ctx(pure, doc, base_url, ctx);
    }
    if rule.starts_with("@xpath:") {
        let pure = &rule[7..];
        return html::select_xpath(&doc.html(), pure).first().cloned();
    }

    if let Some(res) = try_put_get_html_doc(rule, doc, base_url, ctx) {
        return Some(res);
    }

    let (pure, js) = extract_js(rule);
    let text = if pure.is_empty() {
        "".to_string()
    } else {
        html::select_text(doc, pure).unwrap_or_default()
    };

    if let Some(script) = js {
        if let Ok(res) = eval_js(script, &text, base_url) {
            return Some(res);
        }
        return Some(text);
    }

    if text.is_empty() { None } else { Some(text) }
}

fn eval_field_json(rule: &str, v: &Value, base_url: &str) -> Option<String> {
    eval_field_json_with_ctx(rule, v, base_url, &mut HashMap::new())
}

fn eval_field_json_with_ctx(rule: &str, v: &Value, base_url: &str, ctx: &mut HashMap<String, String>) -> Option<String> {
    if let Some(res) = try_put_get_json(rule, v, base_url, ctx) {
        return Some(res);
    }

    let (pure_rule, regex_part) = split_legado_regex(rule);
    let (pure, js) = extract_js(&pure_rule);

    let mut text = if pure.is_empty() {
        "".to_string()
    } else {
        pick_json_field(v, Some(pure)).unwrap_or_default()
    };

    if let Some(script) = js {
        if let Ok(res) = eval_js(script, &text, base_url) {
            text = res;
        }
    }

    if let Some(reg) = regex_part {
        text = apply_legado_regex(&text, reg);
    }

    if text.is_empty() { None } else { Some(text) }
}

fn try_put_get_html(rule: &str, el: &scraper::ElementRef, base_url: &str, ctx: &mut HashMap<String, String>) -> Option<String> {
    if rule.starts_with("@put:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len()-1];
            for part in inner.split(',') {
                if let Some(idx) = part.find(':') {
                    let key = part[..idx].trim();
                    let val_rule = part[idx+1..].trim().trim_matches('"');
                    let val = eval_field_html_with_ctx(val_rule, el, base_url, ctx).unwrap_or_default();
                    ctx.insert(key.to_string(), val);
                }
            }
        }
        return Some("".to_string());
    }
    if rule.starts_with("@get:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let key = &content[1..content.len()-1].trim();
            return ctx.get(*key).cloned();
        }
    }
    None
}

fn try_put_get_html_doc(rule: &str, doc: &scraper::Html, base_url: &str, ctx: &mut HashMap<String, String>) -> Option<String> {
    if rule.starts_with("@put:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len()-1];
            for part in inner.split(',') {
                if let Some(idx) = part.find(':') {
                    let key = part[..idx].trim();
                    let val_rule = part[idx+1..].trim().trim_matches('"');
                    let val = eval_field_html_doc_with_ctx(val_rule, doc, base_url, ctx).unwrap_or_default();
                    ctx.insert(key.to_string(), val);
                }
            }
        }
        return Some("".to_string());
    }
    if rule.starts_with("@get:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let key = &content[1..content.len()-1].trim();
            return ctx.get(*key).cloned();
        }
    }
    None
}

fn try_put_get_json(rule: &str, v: &Value, base_url: &str, ctx: &mut HashMap<String, String>) -> Option<String> {
    if rule.starts_with("@put:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len()-1];
            for part in inner.split(',') {
                if let Some(idx) = part.find(':') {
                    let key = part[..idx].trim();
                    let val_rule = part[idx+1..].trim().trim_matches('"');
                    let val = eval_field_json_with_ctx(val_rule, v, base_url, ctx).unwrap_or_default();
                    ctx.insert(key.to_string(), val);
                }
            }
        }
        return Some("".to_string());
    }
    if rule.starts_with("@get:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let key = &content[1..content.len()-1].trim();
            return ctx.get(*key).cloned();
        }
    }
    None
}

fn split_legado_regex(rule: &str) -> (String, Option<&str>) {
    if let Some(idx) = rule.find("##") {
        let (pure, reg) = rule.split_at(idx);
        return (pure.trim().to_string(), Some(reg));
    }
    (rule.to_string(), None)
}

fn apply_legado_regex(text: &str, regex_part: &str) -> String {
    if regex_part.trim().is_empty() { return text.to_string(); }

    // Handle ### suffix for first-match-only replacement
    let (regex_part, first_only) = if regex_part.ends_with("###") {
        (&regex_part[..regex_part.len()-3], true)
    } else {
        (regex_part, false)
    };

    let parts: Vec<&str> = regex_part.split("##").collect();

    // Support: ##regex##replace
    let start_idx = if regex_part.starts_with("##") { 1 } else { 0 };

    let mut out = text.to_string();
    let mut i = start_idx;
    while i + 1 < parts.len() {
        let regex = parts[i];
        if regex.is_empty() {
            i += 1;
            continue;
        }

        let replace = parts[i + 1];

        if first_only && i + 2 >= parts.len() {
            // Last replacement with ### suffix - first match only
            out = apply_regex_replace_first(&out, regex, replace);
        } else {
            out = apply_regex_replace(&out, regex, replace);
        }
        i += 2;
    }
    out
}

/// Apply regex replacement to first match only
fn apply_regex_replace_first(text: &str, pattern: &str, replacement: &str) -> String {
    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return text.to_string(),
    };
    re.replace(text, replacement).to_string()
}

/// Extract chapter number from title
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mode() {
        let engine = RuleEngine::new().unwrap();

        assert_eq!(engine.detect_mode("@css:.test", ""), ParseMode::Css);
        assert_eq!(engine.detect_mode("@xpath://div", ""), ParseMode::XPath);
        assert_eq!(engine.detect_mode("$.data.list", ""), ParseMode::JsonPath);
        assert_eq!(engine.detect_mode("/html/body/div", ""), ParseMode::XPath);
        assert_eq!(engine.detect_mode(".class", ""), ParseMode::Css);
    }

    #[test]
    fn test_apply_legado_regex() {
        let text = "Hello World 123 456";

        // Test basic replacement (all matches)
        let result = apply_legado_regex(text, "##\\d+##NUM");
        assert_eq!(result, "Hello World NUM NUM");

        // Test first match only (###)
        let result = apply_legado_regex(text, "##\\d+##NUM###");
        assert_eq!(result, "Hello World NUM 456");
    }
}
