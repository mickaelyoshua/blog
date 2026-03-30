use axum::{Router, extract::Path, routing::get};
use tracing::{error, info};

const PORT: &str = "3000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(home))
        .route("/blog", get(blog))
        .route("/blog/{slug}", get(blog_post))
        .route("/cv", get(cv));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}"))
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to bind to port {PORT}");
            std::process::exit(1);
        });

    info!("Server listening at http://localhost:{PORT}");

    axum::serve(listener, router).await.unwrap()
}

async fn home() -> &'static str {
    "home"
}
async fn blog() -> &'static str {
    "home"
}
async fn blog_post(Path(slug): Path<String>) -> String {
    format!("post: {slug}")
}
async fn cv() -> &'static str {
    "home"
}
