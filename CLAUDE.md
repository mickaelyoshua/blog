# Project: Personal Blog & Resume

## Stack

- **Language:** Rust (Edition 2024)
- **Framework:** Axum 0.8
- **Templates:** Askama (compile-time checked Jinja2-like templates)
- **Frontend:** Server-rendered HTML, no client-side JavaScript.
- **Database:** none in v1 (blog posts are markdown files in `content/posts/`, resume is hardcoded). Will revisit if dynamic content is added.

## Architecture: HATEOAS / Hypermedia-Driven Application

This project follows the **Hypermedia-Driven Application (HDA)** architecture. The server is the single source of truth for all application state. The browser renders HTML and follows hypermedia controls — it never manages state independently.

### Core Rules

1. **Server returns HTML, never JSON.** Every response is a complete HTML page. No `/api/` JSON endpoints for the frontend.
2. **HTML is the API.** Links (`<a>`) and forms (`<form>`) encode all available actions. The client needs zero out-of-band knowledge.
3. **No client-side state.** No JavaScript state management, no client-side data models. The DOM is the state.
4. **Available actions change with state.** If a blog post has tags, the server renders tag links. If there are more pages, the server renders a pagination link. The client never decides what to show — the HTML response encodes it.

### Response Pattern

Route handlers return a single full-page Askama template:

```rust
async fn blog_list(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let store = state.posts()?;
    let html = BlogListTemplate {
        posts: store.all.clone(),
        layout: LayoutContext::new("/blog"),
    }
    .render()?;
    Ok(Html(html))
}
```

If interactive features (search-as-you-type, infinite scroll, partial swaps) become necessary later, prefer adding **HTMX** as a thin progressive-enhancement layer over reaching for an SPA framework. Yew or Leptos are options but should only be considered for genuinely client-heavy widgets, not for navigation or content rendering.

### Anti-Patterns (Do NOT Do)

- No `fetch()` or `XMLHttpRequest` calls returning JSON
- No client-side routing or URL matching in JavaScript
- No JavaScript state management (Redux, stores, signals, etc.)
- No generic data API (`/api/v1/...`) for the frontend's own consumption
- No JavaScript that renders HTML from data — the server renders all HTML
- No separate `.js` files for behavior — keep behavior local to the templates if/when JS is added (Locality of Behavior principle)

## Project Structure

```
src/
  main.rs          # Entry point, router, server setup
  routes/          # Axum route handlers (mod per feature: blog, resume, etc.)
  templates/       # Askama template structs
  error.rs         # App-wide error types
templates/         # Askama HTML templates (.html)
  base.html        # Base layout (wraps every page)
  blog/            # Blog templates: list.html, post.html
  resume.html      # Resume page
static/            # Static assets served directly
  css/             # style.css
  fonts/           # Self-hosted IBM Plex Sans/Mono (woff2)
content/
  posts/           # Blog posts as .md files (filename = slug)
```

## Commands

```sh
cargo run                    # Run dev server (listens on $PORT, default 3000)
cargo build --release        # Build release binary
cargo test                   # Run tests
cargo clippy                 # Lint
cargo fmt                    # Format
```

## Conventions

### Code

- Use `cargo fmt` before committing
- Use `cargo clippy` — treat warnings as errors in CI
- Templates go in `templates/` at project root; Askama template structs go in `src/templates/`
- Route handlers return `Result<impl IntoResponse, AppError>` and render a single full-page template

### Content

- Blog posts are markdown files in `content/posts/` with YAML frontmatter (title, date, tags, summary)
- CSS follows a single-file approach (`static/css/style.css`) — no build tooling

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

## Color Scheme (Dark Theme — Carbonfox)

| Role             | Hex       |
|------------------|-----------|
| Background       | `#161616` |
| Surface          | `#252525` |
| Border           | `#3d3d3d` |
| Text primary     | `#f2f4f8` |
| Text secondary   | `#909dab` |
| Blue (primary)   | `#78a9ff` |
| Blue hover       | `#a6c8ff` |
| Violet (accent)  | `#be95ff` |
| Violet hover     | `#d4baff` |
| Gradient         | `#78a9ff → #be95ff` |
| Green            | `#25be6a` |
| Pink             | `#ee5396` |
| Teal             | `#3ddbd9` |
| Merlot (error)   | `#bf4a60` |

## Typography

- **Body:** IBM Plex Sans (400, 500, 600, 700) — self-hosted in `static/fonts/`
- **Code:** IBM Plex Mono (400, 500) — self-hosted in `static/fonts/`
- **Base size:** 18px (desktop), 17px (tablet), 16px (phone)
- **Syntax highlighting:** syntect with CSS classes (prefix `sy-`), Carbonfox color mapping
