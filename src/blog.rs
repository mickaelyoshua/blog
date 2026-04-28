use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use chrono::NaiveDate;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use serde::Deserialize;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use tracing::warn;

use crate::error::AppError;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

// `class` is added to the allowlist for `code` and `span` because syntect
// emits `<span class="sy-…">` markers for syntax highlighting that ammonia
// would otherwise strip. This is the trust-boundary contract between the
// highlighter (which we trust) and user-authored markdown (which we don't).
static SANITIZER: LazyLock<ammonia::Builder<'static>> = LazyLock::new(|| {
    let mut b = ammonia::Builder::default();
    b.add_tag_attributes("code", &["class"]);
    b.add_tag_attributes("span", &["class"]);
    b
});

pub struct Post {
    pub slug: String,
    pub title: String,
    pub date: NaiveDate,
    pub summary: String,
    pub content_html: String,
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    date: NaiveDate,
    summary: String,
}

pub struct BlogStore {
    pub all: Vec<Arc<Post>>,
    pub by_slug: HashMap<String, Arc<Post>>,
}

impl BlogStore {
    pub fn load(content_dir: &str) -> Result<Self, AppError> {
        let posts = load_all_posts(content_dir)?;
        let mut all = Vec::with_capacity(posts.len());
        let mut by_slug = HashMap::with_capacity(posts.len());

        for post in posts {
            let key = post.slug.clone();
            let shared = Arc::new(post);

            all.push(Arc::clone(&shared));
            by_slug.insert(key, Arc::clone(&shared));
        }

        Ok(Self { all, by_slug })
    }
}

fn split_frontmatter<'a>(slug: &str, raw: &'a str) -> Result<(Frontmatter, &'a str), AppError> {
    let bad = |reason: &str| AppError::BadPost {
        slug: slug.to_string(),
        reason: reason.to_string(),
    };

    let after_opening = raw
        .strip_prefix("---\n")
        .ok_or_else(|| bad("missing opening --- fence"))?;

    let (yml, body) = after_opening
        .split_once("\n---")
        .ok_or_else(|| bad("missing closing --- fence"))?;

    let fm = serde_yaml_ng::from_str::<Frontmatter>(yml).map_err(|e| AppError::BadPost {
        slug: slug.to_string(),
        reason: format!("yml: {e}"),
    })?;

    Ok((fm, body.trim_start_matches('\n')))
}

fn render_markdown_to_html(body: &str) -> String {
    let parser = pulldown_cmark::Parser::new(body);
    let mut html = String::new();
    let mut code_buffer = String::new();
    let mut current_lang: Option<String> = None;
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let lang_str = lang.as_ref().trim();
                        if lang_str.is_empty() {
                            None
                        } else {
                            Some(lang_str.to_string())
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
                code_buffer.clear();
            }
            Event::Text(text) if in_code_block => {
                code_buffer.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) => {
                let highlighted = highlight_code(&code_buffer, current_lang.as_deref());
                html.push_str(&highlighted);
                code_buffer.clear();
                current_lang = None;
                in_code_block = false;
            }
            other => {
                pulldown_cmark::html::push_html(&mut html, std::iter::once(other));
            }
        }
    }

    html
}

pub fn parse_post(slug: &str, raw_markdown: &str) -> Result<Post, AppError> {
    let (fm, body) = split_frontmatter(slug, raw_markdown)?;
    // Pipeline order matters: parse markdown → highlight code blocks → sanitize.
    // Sanitization runs last so syntect-generated HTML passes through the same
    // trust boundary as the rest of the user content.
    let html = render_markdown_to_html(body);
    let html = SANITIZER.clean(&html).to_string();

    Ok(Post {
        slug: slug.to_string(),
        title: fm.title,
        date: fm.date,
        summary: fm.summary,
        content_html: html,
    })
}

fn highlight_code(code: &str, lang: Option<&str>) -> String {
    let ss = &*SYNTAX_SET;
    let syntax = lang
        .and_then(|l| ss.find_syntax_by_token(l))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        ss,
        ClassStyle::SpacedPrefixed { prefix: "sy-" },
    );

    for line in LinesWithEndings::from(code) {
        let _ = generator.parse_html_for_line_which_includes_newline(line);
    }

    let inner = generator.finalize();
    // `lang` is interpolated directly into `class="language-{}"`. Restrict it
    // to a safe byte set so a fenced-block language tag can't break out of the
    // attribute (XSS defense). Anything else collapses to "text".
    let lang_attr = match lang {
        Some(l)
            if l.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'+' || b == b'-') =>
        {
            l
        }
        _ => "text",
    };
    format!("<pre><code class=\"highlight language-{lang_attr}\">{inner}</code></pre>")
}

pub fn load_all_posts(content_dir: &str) -> Result<Vec<Post>, AppError> {
    let mut posts = Vec::new();

    let entries = std::fs::read_dir(content_dir)
        .map_err(|e| AppError::Internal(format!("content dir unreadable: {e}")))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "skipping unreadable dir entry");
                continue;
            }
        };

        let path = entry.path();

        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }

        // One bad post must not 500 the whole index — log and move on.
        match load_one_post(&path) {
            Ok(post) => posts.push(post),
            Err(e) => warn!(path = %path.display(), error = ?e, "skipping broken post"),
        }
    }

    posts.sort_by_key(|b| std::cmp::Reverse(b.date));
    Ok(posts)
}

fn load_one_post(path: &std::path::Path) -> Result<Post, AppError> {
    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::Internal(format!("invalid filename: {}", path.display())))?;
    let raw = std::fs::read_to_string(path)?;
    parse_post(slug, &raw)
}

pub struct Slug(String);

impl Slug {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Slug {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Slugs are the canonical URL form: ASCII lowercase + digits + single
        // hyphens, no leading/trailing/double `-`. The 64-byte cap matches
        // typical filename limits and bounds template/log output.
        // Invalid slugs return NotFound (not BadRequest) so we don't reveal
        // whether a slug was malformed vs. simply absent.
        const MAX_LEN: usize = 64;

        let valid = !value.is_empty()
            && value.len() <= MAX_LEN
            && value
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && !value.starts_with('-')
            && !value.ends_with('-')
            && !value.contains("--");

        if valid {
            Ok(Slug(value))
        } else {
            Err(AppError::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod slug {
        use super::*;

        fn assert_rejected(input: &str) {
            match Slug::try_from(input.to_string()) {
                Ok(_) => panic!("expected {input:?} to be rejected, but it was accepted"),
                Err(AppError::NotFound) => {}
                Err(other) => panic!(
                    "expected NotFound for {input:?}, got {other:?} (anti-oracle: malformed slugs must be indistinguishable from absent slugs)"
                ),
            }
        }

        #[test]
        fn accepts_lowercase_alphanumeric_with_single_hyphens() {
            let slug = Slug::try_from("hello-world-2026".to_string()).unwrap();
            assert_eq!(slug.as_str(), "hello-world-2026");
        }

        #[test]
        fn accepts_single_word() {
            assert_eq!(Slug::try_from("rust".to_string()).unwrap().as_str(), "rust");
        }

        #[test]
        fn accepts_digits_only() {
            assert_eq!(Slug::try_from("2026".to_string()).unwrap().as_str(), "2026");
        }

        #[test]
        fn rejects_empty() {
            assert_rejected("");
        }

        #[test]
        fn rejects_uppercase() {
            assert_rejected("Hello-World");
            assert_rejected("HELLO");
        }

        #[test]
        fn rejects_path_traversal_dotdot() {
            assert_rejected("..");
            assert_rejected("../etc/passwd");
            assert_rejected("foo/../bar");
        }

        #[test]
        fn rejects_forward_slash() {
            assert_rejected("foo/bar");
            assert_rejected("/foo");
        }

        #[test]
        fn rejects_leading_hyphen() {
            assert_rejected("-foo");
        }

        #[test]
        fn rejects_trailing_hyphen() {
            assert_rejected("foo-");
        }

        #[test]
        fn rejects_double_hyphen() {
            assert_rejected("foo--bar");
            assert_rejected("--");
        }

        #[test]
        fn rejects_non_ascii() {
            // pt-BR users may try slugs with accents — these must be rejected
            // because the on-disk filename never has them.
            assert_rejected("olá");
            assert_rejected("café");
        }

        #[test]
        fn rejects_special_chars() {
            assert_rejected("foo bar"); // space
            assert_rejected("foo.bar"); // dot
            assert_rejected("foo_bar"); // underscore
            assert_rejected("foo!bar");
            assert_rejected("foo?bar");
            assert_rejected("foo%2fbar"); // url-encoded slash
            assert_rejected("foo<script>");
        }

        #[test]
        fn rejects_too_long() {
            // MAX_LEN is 64 bytes — exactly 64 'a's is allowed, 65 is not.
            let max = "a".repeat(64);
            assert!(Slug::try_from(max).is_ok());

            let over = "a".repeat(65);
            assert_rejected(&over);
        }
    }

    mod frontmatter {
        use super::*;

        const VALID: &str = "---\n\
title: \"Olá\"\n\
date: 2026-04-01\n\
summary: \"Resumo\"\n\
---\n\
# Conteúdo\n";

        #[test]
        fn parses_valid_frontmatter_and_returns_body() {
            let (fm, body) = split_frontmatter("post", VALID).unwrap();
            assert_eq!(fm.title, "Olá");
            assert_eq!(fm.date, NaiveDate::from_ymd_opt(2026, 4, 1).unwrap());
            assert_eq!(fm.summary, "Resumo");
            // `trim_start_matches('\n')` should strip the newline that follows
            // the closing fence so callers can hand the body straight to the
            // markdown parser.
            assert!(body.starts_with("# Conteúdo"));
        }

        #[test]
        fn missing_opening_fence_returns_bad_post_with_slug() {
            let raw = "title: foo\ndate: 2026-04-01\nsummary: x\n";
            match split_frontmatter("my-slug", raw) {
                Err(AppError::BadPost { slug, reason }) => {
                    assert_eq!(slug, "my-slug");
                    assert!(reason.contains("opening"), "reason was: {reason}");
                }
                Ok(_) => panic!("expected BadPost, got Ok"),
                Err(other) => panic!("expected BadPost, got {other:?}"),
            }
        }

        #[test]
        fn missing_closing_fence_returns_bad_post() {
            let raw = "---\ntitle: foo\ndate: 2026-04-01\nsummary: x\n";
            match split_frontmatter("my-slug", raw) {
                Err(AppError::BadPost { slug, reason }) => {
                    assert_eq!(slug, "my-slug");
                    assert!(reason.contains("closing"), "reason was: {reason}");
                }
                Ok(_) => panic!("expected BadPost, got Ok"),
                Err(other) => panic!("expected BadPost, got {other:?}"),
            }
        }

        #[test]
        fn invalid_yaml_returns_bad_post_tagged_yml() {
            // `date` is not a date — yaml will deserialize but serde will
            // reject it on the typed struct.
            let raw = "---\ntitle: foo\ndate: not-a-date\nsummary: x\n---\n";
            match split_frontmatter("my-slug", raw) {
                Err(AppError::BadPost { slug, reason }) => {
                    assert_eq!(slug, "my-slug");
                    assert!(reason.starts_with("yml:"), "reason was: {reason}");
                }
                Ok(_) => panic!("expected BadPost, got Ok"),
                Err(other) => panic!("expected BadPost, got {other:?}"),
            }
        }

        #[test]
        fn missing_required_field_returns_bad_post() {
            // No `summary` — deserialize must fail.
            let raw = "---\ntitle: foo\ndate: 2026-04-01\n---\n";
            match split_frontmatter("p", raw) {
                Err(AppError::BadPost { reason, .. }) => {
                    assert!(reason.starts_with("yml:"), "reason was: {reason}");
                }
                Ok(_) => panic!("expected BadPost, got Ok"),
                Err(other) => panic!("expected BadPost, got {other:?}"),
            }
        }
    }

    mod parse_post {
        use super::*;

        fn raw(body: &str) -> String {
            format!("---\ntitle: \"T\"\ndate: 2026-04-01\nsummary: \"S\"\n---\n{body}")
        }

        #[test]
        fn returns_post_with_frontmatter_fields() {
            let post = super::super::parse_post("hello", &raw("hi")).unwrap();
            assert_eq!(post.slug, "hello");
            assert_eq!(post.title, "T");
            assert_eq!(post.summary, "S");
            assert_eq!(post.date, NaiveDate::from_ymd_opt(2026, 4, 1).unwrap());
        }

        #[test]
        fn strips_script_tags() {
            let post = super::super::parse_post("p", &raw("<script>alert(1)</script>")).unwrap();
            assert!(
                !post.content_html.contains("<script"),
                "ammonia must strip <script>, body was: {}",
                post.content_html
            );
            assert!(!post.content_html.contains("alert(1)"));
        }

        #[test]
        fn strips_iframe_tags() {
            let post = super::super::parse_post("p", &raw("<iframe src=\"http://evil\"></iframe>"))
                .unwrap();
            assert!(!post.content_html.contains("<iframe"));
        }

        #[test]
        fn strips_javascript_urls() {
            let post = super::super::parse_post("p", &raw("[click](javascript:alert(1))")).unwrap();
            // ammonia rewrites or removes the href; either way the protocol
            // string must not survive into the rendered HTML.
            assert!(
                !post.content_html.to_lowercase().contains("javascript:"),
                "javascript: URL leaked into output: {}",
                post.content_html
            );
        }

        #[test]
        fn preserves_syntect_span_classes() {
            // syntect emits <span class="sy-…"> for highlighted tokens.
            // The sanitizer's allowlist must let these through, otherwise
            // syntax highlighting silently breaks for all posts.
            let post = super::super::parse_post("p", &raw("```rust\nfn main() {}\n```\n")).unwrap();
            assert!(
                post.content_html.contains("class=\"sy-"),
                "syntect span classes were stripped: {}",
                post.content_html
            );
        }

        #[test]
        fn preserves_code_block_language_class() {
            let post = super::super::parse_post("p", &raw("```rust\nfn main() {}\n```\n")).unwrap();
            assert!(
                post.content_html
                    .contains("class=\"highlight language-rust\""),
                "language class was stripped: {}",
                post.content_html
            );
        }

        #[test]
        fn preserves_paragraphs_and_headings() {
            let post = super::super::parse_post("p", &raw("# Título\n\nUm parágrafo.\n")).unwrap();
            assert!(post.content_html.contains("<h1>Título</h1>"));
            assert!(post.content_html.contains("<p>Um parágrafo.</p>"));
        }
    }

    mod highlight {
        use super::*;

        #[test]
        fn allows_alphanumeric_lang() {
            let html = highlight_code("x", Some("rust"));
            assert!(html.contains("language-rust"));
        }

        #[test]
        fn allows_lang_with_plus_and_hyphen() {
            // Real syntect tokens like "c++" and "objective-c" must not be
            // collapsed to "text".
            assert!(highlight_code("x", Some("c++")).contains("language-c++"));
            assert!(highlight_code("x", Some("objective-c")).contains("language-objective-c"));
        }

        #[test]
        fn allows_lang_with_underscore() {
            assert!(highlight_code("x", Some("c_sharp")).contains("language-c_sharp"));
        }

        #[test]
        fn collapses_to_text_when_lang_is_none() {
            assert!(highlight_code("x", None).contains("language-text"));
        }

        #[test]
        fn collapses_to_text_when_lang_has_quote() {
            // Attempt to break out of the class attribute.
            let payload = "rust\" onerror=\"alert(1)";
            let html = highlight_code("x", Some(payload));
            assert!(
                html.contains("language-text"),
                "lang with quote must collapse to text, got: {html}"
            );
            assert!(!html.contains("onerror"));
        }

        #[test]
        fn collapses_to_text_when_lang_has_angle_brackets() {
            let html = highlight_code("x", Some("<script>"));
            assert!(html.contains("language-text"));
            assert!(!html.contains("<script"));
        }

        #[test]
        fn collapses_to_text_when_lang_has_space() {
            // Space is not in the allowlist; would otherwise let attacker
            // inject a second attribute.
            let html = highlight_code("x", Some("rust autofocus"));
            assert!(html.contains("language-text"));
            assert!(!html.contains("autofocus"));
        }
    }

    mod store {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        fn write_post(dir: &TempDir, slug: &str, date: &str, title: &str) {
            let body =
                format!("---\ntitle: \"{title}\"\ndate: {date}\nsummary: \"s\"\n---\n# {title}\n");
            fs::write(dir.path().join(format!("{slug}.md")), body).unwrap();
        }

        #[test]
        fn loads_valid_posts_into_all_and_by_slug() {
            let dir = TempDir::new().unwrap();
            write_post(&dir, "a", "2026-01-01", "A");
            write_post(&dir, "b", "2026-02-01", "B");

            let store = BlogStore::load(dir.path().to_str().unwrap()).unwrap();

            assert_eq!(store.all.len(), 2);
            assert!(store.by_slug.contains_key("a"));
            assert!(store.by_slug.contains_key("b"));
            assert_eq!(store.by_slug.get("a").unwrap().title, "A");
        }

        #[test]
        fn sorts_posts_by_date_descending() {
            let dir = TempDir::new().unwrap();
            write_post(&dir, "old", "2025-01-01", "Old");
            write_post(&dir, "new", "2026-04-01", "New");
            write_post(&dir, "mid", "2026-01-15", "Mid");

            let store = BlogStore::load(dir.path().to_str().unwrap()).unwrap();

            let titles: Vec<&str> = store.all.iter().map(|p| p.title.as_str()).collect();
            assert_eq!(
                titles,
                vec!["New", "Mid", "Old"],
                "posts must be sorted by date desc — newest first"
            );
        }

        #[test]
        fn skips_non_markdown_files() {
            let dir = TempDir::new().unwrap();
            write_post(&dir, "real", "2026-04-01", "Real");
            fs::write(dir.path().join("README.txt"), "not a post").unwrap();
            fs::write(dir.path().join("draft.markdown"), "wrong ext").unwrap();
            fs::write(dir.path().join("noext"), "no extension").unwrap();

            let store = BlogStore::load(dir.path().to_str().unwrap()).unwrap();

            assert_eq!(store.all.len(), 1);
            assert_eq!(store.all[0].slug, "real");
        }

        #[test]
        fn skips_broken_posts_without_failing_index() {
            // The documented invariant at blog.rs (load_all_posts comment):
            // "One bad post must not 500 the whole index — log and move on."
            let dir = TempDir::new().unwrap();
            write_post(&dir, "good", "2026-04-01", "Good");
            fs::write(dir.path().join("broken.md"), "no frontmatter here").unwrap();
            fs::write(
                dir.path().join("bad-yaml.md"),
                "---\ntitle: foo\ndate: not-a-date\nsummary: s\n---\n",
            )
            .unwrap();

            let store = BlogStore::load(dir.path().to_str().unwrap()).unwrap();

            assert_eq!(store.all.len(), 1);
            assert_eq!(store.all[0].slug, "good");
            assert!(!store.by_slug.contains_key("broken"));
            assert!(!store.by_slug.contains_key("bad-yaml"));
        }

        #[test]
        fn missing_directory_returns_internal_error() {
            let result = BlogStore::load("/nonexistent/path/that/should/never/exist/here");
            match result {
                Err(AppError::Internal(msg)) => {
                    assert!(
                        msg.contains("content dir unreadable"),
                        "expected diagnostic mentioning content dir, got: {msg}"
                    );
                }
                other => panic!(
                    "expected AppError::Internal for missing dir, got {other:?}",
                    other = match other {
                        Ok(_) => "Ok".to_string(),
                        Err(e) => format!("{e:?}"),
                    }
                ),
            }
        }

        #[test]
        fn empty_directory_yields_empty_store() {
            let dir = TempDir::new().unwrap();
            let store = BlogStore::load(dir.path().to_str().unwrap()).unwrap();
            assert!(store.all.is_empty());
            assert!(store.by_slug.is_empty());
        }

        #[test]
        fn all_and_by_slug_share_the_same_arc() {
            // Memory-efficiency invariant: each post is stored once, behind
            // an Arc, with both views aliasing the same allocation.
            let dir = TempDir::new().unwrap();
            write_post(&dir, "only", "2026-04-01", "Only");

            let store = BlogStore::load(dir.path().to_str().unwrap()).unwrap();
            let from_all = &store.all[0];
            let from_map = store.by_slug.get("only").unwrap();
            assert!(Arc::ptr_eq(from_all, from_map));
        }
    }
}
