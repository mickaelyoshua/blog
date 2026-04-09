use std::sync::LazyLock;

use chrono::NaiveDate;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use serde::Deserialize;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

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
    let parts: Vec<&str> = raw_markdown.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(AppError::Internal(
            "Wrong format for markdown parsing".to_string(),
        ));
    }

    let fm = serde_yml::from_str::<Frontmatter>(parts[1])?;

    let parser = pulldown_cmark::Parser::new(parts[2]);
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

    for entry in std::fs::read_dir(content_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Filter: only .md files
        if path.extension().is_some_and(|ext| ext == "md") {
            let Some(slug) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            let raw = std::fs::read_to_string(&path)?;
            let post = parse_post(slug, &raw)?;
            posts.push(post);
        }
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(posts)
}
