use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};

use crate::{
    blog::Slug,
    error::AppError,
    state::AppState,
    templates::{
        BlogListTemplate, BlogPostTemplate, HomeTemplate, LayoutContext, ResumeTemplate,
    },
};

pub async fn home() -> Result<impl IntoResponse, AppError> {
    let html = HomeTemplate {
        layout: LayoutContext::new("/"),
    }
    .render()?;
    Ok(Html(html))
}

pub async fn resume() -> Result<impl IntoResponse, AppError> {
    let html = ResumeTemplate {
        layout: LayoutContext::new("/cv"),
    }
    .render()?;
    Ok(Html(html))
}

pub async fn blog_list(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let store = state.posts()?;
    let html = BlogListTemplate {
        posts: store.all.clone(),
        layout: LayoutContext::new("/blog"),
    }
    .render()?;
    Ok(Html(html))
}

pub async fn blog_post(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let slug = Slug::try_from(slug)?;
    let store = state.posts()?;
    let post = Arc::clone(store.by_slug.get(slug.as_str()).ok_or(AppError::NotFound)?);

    let html = BlogPostTemplate {
        post,
        layout: LayoutContext::new("/blog"),
    }
    .render()?;
    Ok(Html(html))
}
