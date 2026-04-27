use std::sync::Arc;

use crate::blog::Post;
use askama::Template;

const STYLE_HASH: &str = env!("STYLE_HASH");
const VENDOR_HASH: &str = env!("VENDOR_HASH");

pub struct LayoutContext {
    pub active_nav: &'static str,
    pub style_hash: &'static str,
    pub vendor_hash: &'static str,
}

impl LayoutContext {
    pub fn new(active_nav: &'static str) -> Self {
        Self {
            active_nav,
            style_hash: STYLE_HASH,
            vendor_hash: VENDOR_HASH,
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub layout: LayoutContext,
}

#[derive(Template)]
#[template(path = "home_fragment.html")]
pub struct HomeFragmentTemplate;

#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    pub layout: LayoutContext,
}

#[derive(Template)]
#[template(path = "resume_fragment.html")]
pub struct ResumeFragmentTemplate;

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub status: u16,
    pub message: String,
    pub layout: LayoutContext,
}

#[derive(Template)]
#[template(path = "blog/list.html")]
pub struct BlogListTemplate {
    pub posts: Vec<Arc<Post>>,
    pub layout: LayoutContext,
}

#[derive(Template)]
#[template(path = "blog/list_fragment.html")]
pub struct BlogListFragmentTemplate {
    pub posts: Vec<Arc<Post>>,
}

#[derive(Template)]
#[template(path = "blog/post.html")]
pub struct BlogPostTemplate {
    pub post: Arc<Post>,
    pub layout: LayoutContext,
}

#[derive(Template)]
#[template(path = "blog/post_fragment.html")]
pub struct BlogPostFragmentTemplate {
    pub post: Arc<Post>,
}

pub mod filters {
    use chrono::{Datelike, NaiveDate};

    const MONTH_PT: [&str; 12] = [
        "janeiro",
        "fevereiro",
        "março",
        "abril",
        "maio",
        "junho",
        "julho",
        "agosto",
        "setembro",
        "outubro",
        "novembro",
        "dezembro",
    ];

    #[askama::filter_fn]
    pub fn format_date_br(date: &NaiveDate, _env: &dyn askama::Values) -> askama::Result<String> {
        let month_name = MONTH_PT[date.month() as usize - 1];
        Ok(format!(
            "{} de {} de {}",
            date.day(),
            month_name,
            date.year()
        ))
    }
}
