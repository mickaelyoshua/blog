# Personal Blog & Resume

A personal website built with Rust, featuring a blog (markdown-based) and a resume page. All user-facing content is in Brazilian Portuguese.

## Architecture

This project follows the **HATEOAS / Hypermedia-Driven Application (HDA)** philosophy:

- The server is the single source of truth — it returns **HTML, not JSON**
- [HTMX](https://htmx.org/) enables partial page updates without client-side state management
- Links, forms, and HTMX attributes are the only API — no separate data endpoints
- Every page works with and without JavaScript (progressive enhancement)

See [CLAUDE.md](CLAUDE.md) for detailed architecture rules and patterns.

## Stack

| Layer      | Technology                              |
|------------|-----------------------------------------|
| Backend    | Rust (2024) + Axum 0.8 + axum-htmx     |
| Templates  | Askama (compile-time checked)           |
| Frontend   | HTMX 2.0.4 (vendored, no build tools)  |
| Database   | None in v1 — markdown files only        |
| Styling    | Handwritten CSS, dark theme             |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable, edition 2024)

### Run locally

```sh
git clone https://github.com/YOUR_USERNAME/blog.git
cd blog
cargo run                    # listens on $PORT (default 3000)
# Server starts at http://localhost:3000
```

### Blog posts

Blog posts live in `content/posts/` as markdown files with YAML frontmatter:

```markdown
---
title: Meu Primeiro Post
date: 2026-03-20
tags: [rust, web]
summary: Uma breve descrição do post.
---

Conteúdo do post em **markdown**.
```

The filename (without `.md`) becomes the URL slug: `content/posts/meu-primeiro-post.md` → `/blog/meu-primeiro-post`.

### Export resume as PDF

```sh
make resume                  # writes ./resume.pdf via headless Chromium
BROWSER=/usr/bin/google-chrome-stable make resume   # override browser
```

Requires a pure Chromium-based browser (`chromium`, `google-chrome-stable`, or `brave-browser`). On Arch: `sudo pacman -S chromium`. Vivaldi is **not** supported — its headless mode hangs on background sync services and writes empty PDFs. The Makefile boots the server, waits for `/cv` to respond, then prints the page using the `@media print` stylesheet.

## Deployment

The app compiles to a single static binary. Hosting options under consideration:

- **Fly.io** — managed, one-command deploy, ~$4-5/month
- **Render** — free tier available (with cold starts), $7/month always-on
- **Hetzner Cloud** — self-managed VPS, ~€3.79/month
- **Railway** — git-push deploys, $5/month hobby plan

## License

MIT
