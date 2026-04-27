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
        HomeFragmentTemplate, HomeTemplate, LayoutContext, ResumeFragmentTemplate, ResumeTemplate,
    },
};

fn render<F, P, FT, PT>(
    is_htmx: bool,
    is_boosted: bool,
    fragment: F,
    page: P,
) -> Result<Html<String>, AppError>
where
    F: FnOnce() -> FT,
    P: FnOnce() -> PT,
    FT: Template,
    PT: Template,
{
    if is_htmx && !is_boosted {
        Ok(Html(fragment().render()?))
    } else {
        Ok(Html(page().render()?))
    }
}

pub async fn home(
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    render(
        is_htmx,
        is_boosted,
        || HomeFragmentTemplate,
        || HomeTemplate {
            layout: LayoutContext::new("/"),
        },
    )
}

pub async fn resume(
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    render(
        is_htmx,
        is_boosted,
        || ResumeFragmentTemplate,
        || ResumeTemplate {
            layout: LayoutContext::new("/cv"),
        },
    )
}

pub async fn blog_list(
    State(state): State<AppState>,
    HxRequest(is_htmx): HxRequest,
    HxBoosted(is_boosted): HxBoosted,
) -> Result<impl IntoResponse, AppError> {
    let store = state.posts()?;
    render(
        is_htmx,
        is_boosted,
        || BlogListFragmentTemplate {
            posts: store.all.clone(),
        },
        || BlogListTemplate {
            posts: store.all.clone(),
            layout: LayoutContext::new("/blog"),
        },
    )
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
    let post_for_fragment = Arc::clone(&post);

    render(
        is_htmx,
        is_boosted,
        move || BlogPostFragmentTemplate {
            post: post_for_fragment,
        },
        move || BlogPostTemplate {
            post,
            layout: LayoutContext::new("/blog"),
        },
    )
}
