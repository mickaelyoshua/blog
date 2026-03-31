use chrono::NaiveDate;
use serde::Deserialize;

use crate::error::AppError;

pub struct Post {
    pub slug: String,
    pub title: String,
    pub created_at: NaiveDate,
    pub updated_at: Option<NaiveDate>,
    pub summary: String,
    pub content_html: String,
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    created_at: NaiveDate,
    updated_at: Option<NaiveDate>,
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
    pulldown_cmark::html::push_html(&mut html, parser);

    Ok(Post {
        slug: slug.to_string(),
        title: fm.title,
        created_at: fm.created_at,
        updated_at: fm.updated_at,
        summary: fm.summary,
        content_html: html,
    })
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

    posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(posts)
}
