use std::sync::LazyLock;

use chrono::NaiveDate;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use serde::Deserialize;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use tracing::warn;

use crate::error::AppError;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

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

pub fn parse_post(slug: &str, raw_markdown: &str) -> Result<Post, AppError> {
    let bad = |reason: &str| AppError::BadPost {
        slug: slug.to_string(),
        reason: reason.to_string(),
    };

    let after_opening = raw_markdown
        .strip_prefix("---\n")
        .ok_or_else(|| bad("missing opening --- fence"))?;

    let (yml, body) = after_opening
        .split_once("\n---")
        .ok_or_else(|| bad("missing closing --- fence"))?;

    let fm = serde_yaml_ng::from_str::<Frontmatter>(yml).map_err(|e| AppError::BadPost {
        slug: slug.to_string(),
        reason: format!("yml: {e}"),
    })?;
    let body = body.trim_start_matches('\n');

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
    let lang_attr = lang.unwrap_or("text");
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

        match load_one_post(&path) {
            Ok(post) => posts.push(post),
            Err(e) => warn!(path = %path.display(), error = ?e, "skipping broken post"),
        }
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date));
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
        const MAX_LEN: usize = 64;

        if value.is_empty() || value.len() > MAX_LEN {
            return Err(AppError::NotFound);
        }

        let ok = value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !ok {
            return Err(AppError::NotFound);
        }

        if value.starts_with('-') || value.ends_with('-') || value.contains("--") {
            return Err(AppError::NotFound);
        }

        Ok(Slug(value))
    }
}
