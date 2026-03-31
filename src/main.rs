use axum::{Router, routing::get};
use blog::routes::{blog_list, blog_post, home, resume};
use tower_http::services::ServeDir;
use tracing::{error, info};

const PORT: &str = "3000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(home))
        .route("/blog", get(blog_list))
        .route("/blog/{slug}", get(blog_post))
        .route("/cv", get(resume))
        .nest_service("/static", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}"))
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to bind to port {PORT}");
            std::process::exit(1);
        });

    info!("Server listening at http://localhost:{PORT}");

    axum::serve(listener, router).await.unwrap()
}
