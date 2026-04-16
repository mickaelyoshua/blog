use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use axum_htmx::{HxBoosted, HxRequest};

use crate::{
    blog::{Slug, load_all_posts, parse_post},
    error::AppError,
    templates::{
        BlogListFragmentTemplate, BlogListTemplate, BlogPostFragmentTemplate, BlogPostTemplate,
        HomeFragmentTemplate, HomeTemplate, ResumeFragmentTemplate, ResumeTemplate, STATIC_HASH,
    },
};

pub async fn home(
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    if is_htmx && !is_boosted {
        let html = HomeFragmentTemplate.render()?;
        Ok(Html(html))
    } else {
        let html = HomeTemplate {
            active_nav: "/",
            static_hash: STATIC_HASH,
        }
        .render()?;
        Ok(Html(html))
    }
}

pub async fn resume(
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    if is_htmx && !is_boosted {
        let html = ResumeFragmentTemplate.render()?;
        Ok(Html(html))
    } else {
        let html = ResumeTemplate {
            active_nav: "/cv",
            static_hash: STATIC_HASH,
        }
        .render()?;
        Ok(Html(html))
    }
}

pub async fn blog_list(
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    let posts = load_all_posts("content/posts")?;
    if is_htmx && !is_boosted {
        let html = BlogListFragmentTemplate { posts }.render()?;
        Ok(Html(html))
    } else {
        let html = BlogListTemplate {
            posts,
            active_nav: "/blog",
            static_hash: STATIC_HASH,
        }
        .render()?;
        Ok(Html(html))
    }
}

pub async fn blog_post(
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let slug = Slug::try_from(slug)?;
    let path = format!("content/posts/{}.md", slug.as_str());
    let raw = std::fs::read_to_string(path)?;
    let post = parse_post(slug.as_str(), &raw)?;

    if is_htmx && !is_boosted {
        let html = BlogPostFragmentTemplate { post }.render()?;
        Ok(Html(html))
    } else {
        let html = BlogPostTemplate {
            post,
            active_nav: "/blog",
            static_hash: STATIC_HASH,
        }
        .render()?;
        Ok(Html(html))
    }
}
