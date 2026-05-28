# AGENTS.md — Personal Blog & Resume

Developer guide for Antigravity when working on this repository.

## Stack & Architecture

- **Language:** Rust (Edition 2024)
- **Framework:** Axum 0.8
- **Templates:** Askama (compile-time checked Jinja2-like templates)
- **Architecture:** HATEOAS / Hypermedia-Driven Application (HDA). The server is the single source of truth and returns only HTML. No client-side JS state management or JSON APIs for the frontend.
- **Database:** None in v1 (blog posts are markdown files in `content/posts/`, resume is hardcoded).

### Core Rules

1. **Server returns HTML, never JSON.** No `/api/` endpoints for the frontend.
2. **HTML is the API.** The client needs zero out-of-band knowledge.
3. **No client-side state.** No JavaScript state management. The DOM is the state.
4. **Available actions change with state.** HTML responses encode all state actions.
5. **No separate `.js` files.** Keep behavior local to templates if JS is ever added (Locality of Behavior).

## Project Structure

```
src/
  main.rs          # Entry point, router, server setup
  routes/          # Axum route handlers (mod per feature)
  templates/       # Askama template structs
  error.rs         # App-wide error types
templates/         # Askama HTML templates (.html)
  base.html        # Base layout (wraps every page)
  blog/            # Blog templates: list.html, post.html
  resume.html      # Resume page
static/            # Static assets served directly
  css/             # style.css (Carbonfox theme)
  fonts/           # Self-hosted IBM Plex Sans/Mono (woff2)
content/
  posts/           # Blog posts as .md files (filename = slug)
```

## Commands

```sh
cargo run                    # Run dev server (default port 3000)
cargo build --release        # Build release binary
cargo test                   # Run tests
cargo clippy                 # Lint (warnings as errors)
cargo fmt                    # Format code
```

## Conventions

- **Language:**
  - **User-facing content (templates, blog posts):** Brazilian Portuguese (pt-BR). HTML `lang="pt-BR"`. Date: `20 de março de 2026`.
  - **Code, comments, docs, commits:** English.
- **Code:** Use early returns and guard clauses. Run `cargo fmt` before committing.
- **Style & Typography:**
  - CSS in `static/css/style.css` (Carbonfox Dark theme: background `#161616`, primary `#78a9ff`).
  - Fonts: IBM Plex Sans & Mono self-hosted.
- **Deploy:** Live at **https://yoshua.fly.dev** (Fly.io). Push to `master` triggers Github Actions CI/CD.

## URL Design

```
GET  /                  # Home page
GET  /blog              # Blog listing (supports ?q=, ?tag=, ?page=)
GET  /blog/:slug        # Individual post
GET  /cv                # Resume page
```
