use std::time::Duration;

use axum::{Router, http::StatusCode, middleware::from_fn_with_state, routing::get};
use blog::{
    env::Env,
    middleware::security_headers,
    routes::{blog_list, blog_post, healthz, home, resume},
    state::AppState,
};
use tokio::signal::unix::SignalKind;
use tower_http::{
    compression::CompressionLayer, services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    let env = Env::from_env();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,blog=debug,tower_http=debug")),
        )
        .with_ansi(env == Env::Development)
        .init();

    info!(?env, "Detected environment");

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let state = AppState::new("content/posts").expect("failed to load content/posts at startup");

    let app = Router::new()
        .route("/", get(home))
        .route("/blog", get(blog_list))
        .route("/blog/{slug}", get(blog_post))
        .route("/cv", get(resume))
        .route("/healthz", get(healthz))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // /static and the security-headers layer are mounted outside TraceLayer so
    // per-asset requests don't generate spans (would drown out request traces).
    let router = app
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .layer(from_fn_with_state(env, security_headers));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to bind to port {port}");
            std::process::exit(1);
        });

    info!("Server listening at http://localhost:{port}");

    if let Err(e) = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!(error = %e, "server terminated unexpectedly");
        std::process::exit(1);
    }

    info!("Server shut down cleanly");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install SIGINT handler")
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, draining in-flight requests");
}
