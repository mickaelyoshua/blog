use std::sync::Arc;

use crate::blog::Post;
use askama::Template;

const STYLE_HASH: &str = env!("STYLE_HASH");

pub struct LayoutContext {
    pub active_nav: &'static str,
    pub style_hash: &'static str,
}

impl LayoutContext {
    pub fn new(active_nav: &'static str) -> Self {
        Self {
            active_nav,
            style_hash: STYLE_HASH,
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub layout: LayoutContext,
}

#[derive(Template)]
#[template(path = "resume.html")]
pub struct ResumeTemplate {
    pub layout: LayoutContext,
}

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
#[template(path = "blog/post.html")]
pub struct BlogPostTemplate {
    pub post: Arc<Post>,
    pub layout: LayoutContext,
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

    pub(crate) fn render_date_br(date: &NaiveDate) -> String {
        let month_name = MONTH_PT[date.month() as usize - 1];
        format!("{} de {} de {}", date.day(), month_name, date.year())
    }

    #[askama::filter_fn]
    pub fn format_date_br(date: &NaiveDate, _env: &dyn askama::Values) -> askama::Result<String> {
        Ok(render_date_br(date))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn fmt(y: i32, m: u32, d: u32) -> String {
            render_date_br(&NaiveDate::from_ymd_opt(y, m, d).unwrap())
        }

        #[test]
        fn formats_canonical_example_from_claude_md() {
            // CLAUDE.md commits the project to this exact format:
            // "20 de março de 2026". This test pins the accents and word
            // separators against a future regression.
            assert_eq!(fmt(2026, 3, 20), "20 de março de 2026");
        }

        #[test]
        fn formats_january_with_pt_br_name() {
            assert_eq!(fmt(2026, 1, 1), "1 de janeiro de 2026");
        }

        #[test]
        fn formats_december_boundary() {
            assert_eq!(fmt(2026, 12, 31), "31 de dezembro de 2026");
        }

        #[test]
        fn does_not_zero_pad_day() {
            // "1" not "01" — matches Brazilian written convention.
            assert_eq!(fmt(2026, 5, 1), "1 de maio de 2026");
            assert_eq!(fmt(2026, 5, 9), "9 de maio de 2026");
        }

        #[test]
        fn includes_full_month_names_with_accents() {
            assert_eq!(fmt(2026, 2, 15), "15 de fevereiro de 2026");
            assert_eq!(fmt(2026, 3, 15), "15 de março de 2026");
            assert_eq!(fmt(2026, 4, 15), "15 de abril de 2026");
            assert_eq!(fmt(2026, 6, 15), "15 de junho de 2026");
            assert_eq!(fmt(2026, 9, 15), "15 de setembro de 2026");
            assert_eq!(fmt(2026, 11, 15), "15 de novembro de 2026");
        }
    }
}
