# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Build and Run Commands

### Rust Backend
```bash
cargo run                    # Run in development mode
cargo build --release        # Build for release
cargo test                   # Run tests
cargo test --lib <test_name> # Run a specific test
```

### Web Frontend (Vue 2)
```bash
cd web
npm install                   # Install dependencies
npm run serve                # Development server
npm run build                # Production build
npm run lint                 # Lint code
```

## Configuration

Configuration is loaded from environment variables. Default values are defined in `src/app/config.rs`. Key settings:

- `SERVER_HOST` / `SERVER_PORT` - Server binding (default: `0.0.0.0:8080`)
- `DATABASE_URL` - SQLite path (default: `sqlite:storage/reader.db?mode=rwc`)
- `WEB_ROOT` - Static web files directory (default: `../reader/web`)
- `STORAGE_DIR` / `ASSETS_DIR` - Storage paths
- `LOG_LEVEL` - Logging verbosity (default: `info`)
- `REQUEST_TIMEOUT_SECS` - HTTP request timeout (default: 15)

Environment variable separator is `__` (double underscore).

## Architecture Overview

This is a Rust implementation of "阅读3.0" (Reader 3.0) - a book reading API server. It provides book source management, search, chapter retrieval, and content parsing.

### Module Structure

- **`src/api/`** - HTTP handlers and routing (axum)
  - `router.rs` - Route definitions for all `/reader3/*` endpoints
  - `handlers/` - Request handlers by domain (book, bookmark, rss, user, etc.)
  - `AppState` - Shared application state (config, services)

- **`src/service/`** - Business logic layer
  - `book_service.rs` - Book search, info, chapters, content
  - `book_source_service.rs` - CRUD for book sources (stored in SQLite)
  - `user_service.rs` - User management and authentication

- **`src/parser/`** - Content extraction engine
  - `rule_engine.rs` - Main entry point, auto-detects content type
  - `html.rs` - CSS selector parsing (using scraper crate)
  - `jsonpath.rs` - JSONPath queries (using jsonpath_lib)
  - `js.rs` - JavaScript execution (rquickjs) for `js:` prefixed rules

- **`src/crawler/`** - HTTP fetching
  - `http_client.rs` - Configurable reqwest client with compression support
  - `fetcher.rs` - Page fetching with URL resolution

- **`src/model/`** - Data structures
  - `book_source.rs` - Book source JSON schema with rule definitions
  - `rule.rs` - SearchRule, BookInfoRule, TocRule, ContentRule types

- **`src/storage/`** - Persistence
  - `db/` - SQLite via sqlx, with `BookSourceRepo`
  - `cache/` - File-based chapter content cache (MD5 key)
  - `fs/` - Filesystem operations for storage/assets

### Request Flow

1. `api/handlers/` receives HTTP request
2. Handler calls `service/` layer
3. Service fetches remote content via `crawler/http_client.rs`
4. Content is parsed using `parser/rule_engine.rs` with rules from `BookSource`
5. Results returned as JSON

### Rule Parsing

Book sources define parsing rules in JSON. The `RuleEngine` auto-detects parsing mode:

- **CSS selectors** - Default for HTML (`.class`, `#id`, `tag`)
- **JSONPath** - Auto-detected for JSON content (`$.data.list`)
- **XPath** - Lines starting with `/` or `./`
- **JavaScript** - Rules prefixed with `js:` or `@js:`
- **Regex** - Rules starting with `:`

Rule prefixes can be explicit: `@css:`, `@json:`, `@xpath:`, `@regex:`.

### Book Source Format

Book sources are JSON objects containing:
- `bookSourceUrl` / `bookSourceName` - Source identity
- `searchUrl` / `exploreUrl` - Search/discovery URLs with `${key}` placeholders
- `ruleSearch` / `ruleBookInfo` / `ruleToc` / `ruleContent` - Parsing rules for each stage
- Each rule has fields like `bookList`, `name`, `author`, `bookUrl`, etc.

## Web Frontend

The `web/` directory contains a Vue 2 frontend (阅读3.0 web client). It connects to the Rust backend API. Build output goes to `web/dist/`, which the backend serves as static files.