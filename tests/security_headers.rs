// Integration tests for src/middleware.rs::security_headers.
//
// Two pieces of behavior are critical:
//   1. Five security headers (CSP, X-Frame-Options, X-Content-Type-Options,
//      Referrer-Policy, Permissions-Policy) are always set.
//   2. HSTS is added ONLY when Env::Production. A regression here is hard to
//      detect by manual smoke-test on a dev machine but causes browser-side
//      lockout in production.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    middleware::from_fn_with_state,
    routing::get,
};
use blog::{env::Env, middleware::security_headers};
use tower::ServiceExt;

async fn ok() -> &'static str {
    "ok"
}

fn router_with(env: Env) -> Router {
    Router::new()
        .route("/", get(ok))
        .layer(from_fn_with_state(env, security_headers))
}

async fn fetch_headers(env: Env) -> axum::http::HeaderMap {
    let resp = router_with(env)
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    resp.headers().clone()
}

#[tokio::test]
async fn always_sets_x_content_type_options_nosniff() {
    for env in [Env::Development, Env::Production] {
        let h = fetch_headers(env).await;
        assert_eq!(
            h.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff",
            "missing in {env:?}"
        );
    }
}

#[tokio::test]
async fn always_sets_x_frame_options_deny() {
    for env in [Env::Development, Env::Production] {
        let h = fetch_headers(env).await;
        assert_eq!(h.get(header::X_FRAME_OPTIONS).unwrap(), "DENY");
    }
}

#[tokio::test]
async fn always_sets_referrer_policy_strict_origin_when_cross_origin() {
    for env in [Env::Development, Env::Production] {
        let h = fetch_headers(env).await;
        assert_eq!(
            h.get(header::REFERRER_POLICY).unwrap(),
            "strict-origin-when-cross-origin"
        );
    }
}

#[tokio::test]
async fn always_sets_strict_csp() {
    let h = fetch_headers(Env::Production).await;
    let csp = h
        .get(header::CONTENT_SECURITY_POLICY)
        .unwrap()
        .to_str()
        .unwrap();
    // Pin the strict knobs that protect us specifically.
    // 'unsafe-inline' MUST NOT appear (templates ship no inline JS / CSS).
    assert!(
        !csp.contains("unsafe-inline"),
        "CSP allows unsafe-inline: {csp}"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "CSP allows unsafe-eval: {csp}"
    );
    assert!(csp.contains("default-src 'self'"));
    // No JavaScript ships with the app — script-src must be 'none'. Loosening
    // this back to 'self' is a deliberate decision (e.g. HTMX) and should
    // require updating this test alongside.
    assert!(
        csp.contains("script-src 'none'"),
        "script-src is not locked down: {csp}"
    );
    assert!(csp.contains("frame-ancestors 'none'"));
    assert!(csp.contains("object-src 'none'"));
    assert!(csp.contains("base-uri 'self'"));
}

#[tokio::test]
async fn always_sets_cross_origin_isolation_headers() {
    // Defense-in-depth on top of frame-ancestors / X-Frame-Options: COOP
    // breaks window.opener references from cross-origin pages, CORP blocks
    // cross-origin <embed>/<object>/<img> usage of our resources.
    for env in [Env::Development, Env::Production] {
        let h = fetch_headers(env).await;
        assert_eq!(
            h.get("cross-origin-opener-policy")
                .and_then(|v| v.to_str().ok()),
            Some("same-origin"),
            "COOP missing in {env:?}"
        );
        assert_eq!(
            h.get("cross-origin-resource-policy")
                .and_then(|v| v.to_str().ok()),
            Some("same-origin"),
            "CORP missing in {env:?}"
        );
    }
}

#[tokio::test]
async fn always_sets_permissions_policy() {
    let h = fetch_headers(Env::Development).await;
    let pp = h.get("permissions-policy").unwrap().to_str().unwrap();
    assert!(pp.contains("camera=()"));
    assert!(pp.contains("microphone=()"));
    assert!(pp.contains("geolocation=()"));
    // FLoC opt-out — once shipped, removing it is an unintended ad-targeting
    // re-enable.
    assert!(pp.contains("interest-cohort=()"));
}

#[tokio::test]
async fn hsts_only_set_in_production() {
    // The production-gating is the single most important property of this
    // middleware. HSTS in dev would attack localhost; HSTS in prod with
    // broken TLS is a 1-year browser lockout (see plan.md).
    let dev = fetch_headers(Env::Development).await;
    assert!(
        dev.get(header::STRICT_TRANSPORT_SECURITY).is_none(),
        "HSTS leaked into dev response"
    );

    let prod = fetch_headers(Env::Production).await;
    let hsts = prod
        .get(header::STRICT_TRANSPORT_SECURITY)
        .expect("HSTS missing in prod")
        .to_str()
        .unwrap();
    assert!(
        hsts.contains("max-age=31536000"),
        "HSTS max-age changed: {hsts}"
    );
    assert!(
        hsts.contains("includeSubDomains"),
        "HSTS missing includeSubDomains: {hsts}"
    );
    // `preload` is required so the domain qualifies for the Chrome HSTS
    // preload list. Removing it is a deliberate decision (withdrawing from
    // the preload list) — update this test if you do that.
    assert!(hsts.contains("preload"), "HSTS missing preload: {hsts}");
}
