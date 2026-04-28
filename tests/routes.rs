// HTTP-level integration tests. These drive the actual Axum router via
// `tower::ServiceExt::oneshot`, so they exercise extractors, error
// IntoResponse impls, and template rendering end-to-end.
//
// Each test builds a small router with a temp content dir so the suite is
// independent of `content/posts/` in the repo.

use std::fs;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use blog::{
    routes::{blog_list, blog_post, healthz, home, resume},
    state::AppState,
};
use http_body_util::BodyExt;
use tempfile::TempDir;
use tower::ServiceExt;

fn write_post(dir: &TempDir, slug: &str, date: &str, title: &str) {
    let body =
        format!("---\ntitle: \"{title}\"\ndate: {date}\nsummary: \"resumo\"\n---\n# {title}\n");
    fs::write(dir.path().join(format!("{slug}.md")), body).unwrap();
}

fn router_with_posts(dir: &TempDir) -> Router {
    let state = AppState::new(dir.path().to_str().unwrap()).unwrap();
    Router::new()
        .route("/", get(home))
        .route("/blog", get(blog_list))
        .route("/blog/{slug}", get(blog_post))
        .route("/cv", get(resume))
        .route("/healthz", get(healthz))
        .with_state(state)
}

async fn get_status_and_body(app: Router, uri: &str) -> (StatusCode, String) {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    (status, body)
}

#[tokio::test]
async fn home_returns_200_html() {
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);
    let (status, body) = get_status_and_body(app, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.starts_with("<!doctype html>"));
    assert!(body.contains("Yoshua"));
}

#[tokio::test]
async fn healthz_returns_200_with_ok_body() {
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);
    let (status, body) = get_status_and_body(app, "/healthz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn cv_returns_200_html() {
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);
    let (status, body) = get_status_and_body(app, "/cv").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("<!doctype html>"));
}

#[tokio::test]
async fn blog_list_renders_post_titles() {
    let dir = TempDir::new().unwrap();
    write_post(&dir, "alpha", "2026-04-01", "Alpha");
    write_post(&dir, "beta", "2026-04-02", "Beta");
    let app = router_with_posts(&dir);

    let (status, body) = get_status_and_body(app, "/blog").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Alpha"));
    assert!(body.contains("Beta"));
    // Newest first (date desc) — Beta (Apr 2) before Alpha (Apr 1).
    let pos_beta = body.find("Beta").unwrap();
    let pos_alpha = body.find("Alpha").unwrap();
    assert!(
        pos_beta < pos_alpha,
        "expected newer post (Beta) to appear before older (Alpha)"
    );
}

#[tokio::test]
async fn blog_list_with_no_posts_shows_empty_state_message() {
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);
    let (status, body) = get_status_and_body(app, "/blog").await;
    assert_eq!(status, StatusCode::OK);
    // pt-BR empty-state copy from templates/blog/list.html:
    assert!(body.contains("Nenhum post publicado ainda"));
}

#[tokio::test]
async fn blog_post_returns_200_for_valid_slug() {
    let dir = TempDir::new().unwrap();
    write_post(&dir, "hello", "2026-04-01", "Olá");
    let app = router_with_posts(&dir);

    let (status, body) = get_status_and_body(app, "/blog/hello").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Olá"));
}

#[tokio::test]
async fn blog_post_returns_404_for_missing_slug() {
    let dir = TempDir::new().unwrap();
    write_post(&dir, "real", "2026-04-01", "Real");
    let app = router_with_posts(&dir);

    let (status, body) = get_status_and_body(app, "/blog/does-not-exist").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    // Rendered error page, not raw text.
    assert!(body.contains("404"));
    assert!(body.contains("Página não encontrada"));
}

#[tokio::test]
async fn blog_post_returns_404_for_uppercase_slug_not_400() {
    // Anti-oracle: malformed slug must NOT return a distinct error code.
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);

    let (status, _) = get_status_and_body(app, "/blog/UPPERCASE").await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "uppercase slug must return 404, not 400 (anti-oracle vs missing slug)"
    );
}

#[tokio::test]
async fn blog_post_returns_404_for_double_hyphen_slug() {
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);

    let (status, _) = get_status_and_body(app, "/blog/foo--bar").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn blog_post_rejects_url_encoded_slash_in_slug() {
    // %2F is `/`. If axum decoded it into the path segment, our slug
    // validator would still reject it because '/' isn't in the allowlist —
    // and either way, a path-traversal payload must come back as 404.
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);

    let (status, _) = get_status_and_body(app, "/blog/foo%2Fbar").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn blog_post_rejects_dotdot_slug() {
    let dir = TempDir::new().unwrap();
    let app = router_with_posts(&dir);

    let (status, _) = get_status_and_body(app, "/blog/..").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn blog_post_renders_sanitized_html_into_page() {
    // End-to-end: a post containing <script> in markdown must be served
    // without that <script> in the response body.
    let dir = TempDir::new().unwrap();
    let body =
        "---\ntitle: \"X\"\ndate: 2026-04-01\nsummary: \"s\"\n---\n<script>alert(1)</script>\n";
    fs::write(dir.path().join("xss.md"), body).unwrap();
    let app = router_with_posts(&dir);

    let (status, page) = get_status_and_body(app, "/blog/xss").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        !page.contains("<script>alert(1)</script>"),
        "raw <script> reached the response"
    );
    assert!(!page.contains("alert(1)"));
}
