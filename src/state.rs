// Two AppState implementations are gated by `cfg(debug_assertions)`:
//
// - Release builds load `content/posts/` once at startup and serve every
//   request from a shared `Arc<BlogStore>`.
// - Debug builds re-read the directory on every request so authors get
//   hot-reload of markdown files without restarting `cargo run`.
//
// The dev path still calls `BlogStore::load` once in `new()` (discarding the
// result) so that a broken posts directory fails the boot, not the first
// request.

use crate::{blog::BlogStore, error::AppError};
use std::sync::Arc;

#[cfg(not(debug_assertions))]
mod imp {

    use super::*;

    #[derive(Clone)]
    pub struct AppState {
        store: Arc<BlogStore>,
    }

    impl AppState {
        pub fn new(content_dir: &str) -> Result<Self, AppError> {
            let blog_store = BlogStore::load(content_dir)?;
            Ok(AppState {
                store: Arc::new(blog_store),
            })
        }

        pub fn posts(&self) -> Result<Arc<BlogStore>, AppError> {
            Ok(Arc::clone(&self.store))
        }
    }
}

#[cfg(debug_assertions)]
mod imp {
    use super::*;

    #[derive(Clone)]
    pub struct AppState {
        content_dir: Arc<String>,
    }

    impl AppState {
        pub fn new(content_dir: &str) -> Result<Self, AppError> {
            let _ = BlogStore::load(content_dir)?;
            Ok(AppState {
                content_dir: Arc::new(content_dir.to_string()),
            })
        }

        pub fn posts(&self) -> Result<Arc<BlogStore>, AppError> {
            let blog_store = BlogStore::load(&self.content_dir)?;
            Ok(Arc::new(blog_store))
        }
    }
}

pub use imp::AppState;
