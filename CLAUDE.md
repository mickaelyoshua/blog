# Project: Personal Blog & Resume

## Stack

- **Language:** Rust (Edition 2024)
- **Framework:** Axum 0.8 + `axum-htmx` (HTMX extractors/responders)
- **Templates:** Askama (compile-time checked Jinja2-like templates)
- **Frontend:** HTMX 2.0.4 (vendored in `static/vendor/`)
- **Database:** PostgreSQL 16 + SQLx 0.8 (not needed for v1 — blog posts are markdown files, resume is hardcoded)
- **Dev database:** docker-compose (PostgreSQL 16)

## Architecture: HATEOAS / Hypermedia-Driven Application

This project follows the **Hypermedia-Driven Application (HDA)** architecture. The server is the single source of truth for all application state. The browser renders HTML and follows hypermedia controls — it never manages state independently.

### Core Rules

1. **Server returns HTML, never JSON.** Every response is HTML — either a full page or a fragment. No `/api/` JSON endpoints for the frontend.
2. **HTML is the API.** Links (`<a>`), forms (`<form>`), and HTMX attributes (`hx-get`, `hx-post`, etc.) encode all available actions. The client needs zero out-of-band knowledge.
3. **No client-side state.** No JavaScript state management, no client-side data models. The DOM is the state.
4. **Available actions change with state.** If a blog post has tags, the server renders tag links. If there are more pages, the server renders a "load more" button. The client never decides what to show — the HTML response encodes it.

### Response Pattern: Full Page vs Fragment

Route handlers must support two response modes:

- **Full page** (direct navigation, no `HX-Request` header): Return complete HTML with layout, nav, head, etc.
- **HTMX partial** (`HX-Request: true` header): Return only the HTML fragment the target element needs.

Use the `axum-htmx` crate's `HxRequest` extractor:

```rust
use axum_htmx::HxRequest;

async fn blog_list(
    HxRequest(is_htmx): HxRequest,
) -> Result<impl IntoResponse, AppError> {
    let posts = load_posts()?;
    if is_htmx {
        Ok(BlogListFragment { posts }.into_response())
    } else {
        Ok(BlogListPage { posts }.into_response())
    }
}
```

**Exception:** `hx-boost` requests send `HX-Request: true` but expect a full page (HTMX extracts the `<body>`). Check `HxBoosted` when needed.

### HTMX Patterns

- **Navigation:** `hx-boost="true"` on `<body>` — all links become AJAX, browser history works via `hx-push-url`
- **Search/filter:** `hx-get` with `hx-trigger="input changed delay:500ms"` — server returns filtered HTML fragment
- **Pagination:** "Load more" button with `hx-get="/blog?page=N"` swapping itself out with new content
- **Loading states:** `hx-indicator` pointing to a spinner element (HTMX manages visibility via CSS classes)
- **Errors:** Server returns error UI as HTML (e.g., re-renders form with error messages on 422)

### Anti-Patterns (Do NOT Do)

- No `fetch()` or `XMLHttpRequest` calls returning JSON
- No client-side routing or URL matching in JavaScript
- No JavaScript state management (Redux, stores, signals, etc.)
- No generic data API (`/api/v1/...`) for the frontend's own consumption
- No JavaScript that renders HTML from data — the server renders all HTML
- No separate `.js` files for behavior — use HTMX attributes and inline scripts only when necessary (Locality of Behavior principle)

## Project Structure

```
src/
  main.rs          # Entry point, router, server setup
  routes/          # Axum route handlers (mod per feature: blog, resume, etc.)
  templates/       # Askama template structs (full pages + fragments)
  error.rs         # App-wide error types
templates/         # Askama HTML templates (.html)
  base.html        # Base layout (wraps full-page responses)
  blog/            # Blog-related templates (page + fragment variants)
  resume.html      # Resume page
static/            # Static assets served directly
  css/
  vendor/          # Vendored JS (htmx.min.js)
content/
  posts/           # Blog posts as .md files (filename = slug)
migrations/        # SQLx migrations (future)
```

## Commands

```sh
cargo run                    # Run dev server
cargo build --release        # Build release binary
cargo test                   # Run tests
cargo clippy                 # Lint
cargo fmt                    # Format
docker compose up -d         # Start local PostgreSQL (when needed)
```

## Conventions

### Code

- Use `cargo fmt` before committing
- Use `cargo clippy` — treat warnings as errors in CI
- Templates go in `templates/` at project root; Askama template structs go in `src/templates/`
- Route handlers return `Result<impl IntoResponse, AppError>`
- Every route that serves HTML must handle both full-page and fragment responses (via `HxRequest` extractor)

### Content

- Blog posts are markdown files in `content/posts/` with YAML frontmatter (title, date, tags, summary)
- CSS follows a single-file approach (`static/css/style.css`) — no build tooling
- HTMX attributes live in templates, not in separate JS files

### Language

- **User-facing content (templates, blog posts):** Brazilian Portuguese (pt-BR)
- **Code, comments, docs, variable names, commit messages:** English
- HTML `lang="pt-BR"` in base template
- Date formatting follows Brazilian convention: `20 de março de 2026`

### URL Design (Resource-Oriented)

```
GET  /                  # Home page (Página Inicial)
GET  /blog              # Blog listing, supports ?q=, ?tag=, ?page=
GET  /blog/:slug        # Individual post
GET  /cv                # Resume page
```

## Color Scheme (Dark Theme)

| Role             | Hex       |
|------------------|-----------|
| Background       | `#0b0b14` |
| Surface          | `#151525` |
| Border           | `#2a2a3e` |
| Text primary     | `#e4e4ef` |
| Text secondary   | `#8888a4` |
| Cyan (primary)   | `#22d3ee` |
| Cyan hover       | `#67e8f9` |
| Purple (accent)  | `#a78bfa` |
| Purple hover     | `#c4b5fd` |
| Gradient         | `#22d3ee → #a78bfa` |
| Success (teal)   | `#2dd4bf` |
| Error (rose)     | `#f472b6` |
