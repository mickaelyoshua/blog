use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use axum_htmx::{HxBoosted, HxRequest};

use crate::{
    blog::Slug,
    error::AppError,
    state::AppState,
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
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    let store = state.posts()?;
    if is_htmx && !is_boosted {
        let html = BlogListFragmentTemplate {
            posts: store.all.clone(),
        }
        .render()?;
        Ok(Html(html))
    } else {
        let html = BlogListTemplate {
            posts: store.all.clone(),
            active_nav: "/blog",
            static_hash: STATIC_HASH,
        }
        .render()?;
        Ok(Html(html))
    }
}

pub async fn blog_post(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let slug = Slug::try_from(slug)?;
    let store = state.posts()?;
    let post = Arc::clone(store.by_slug.get(slug.as_str()).ok_or(AppError::NotFound)?);

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
