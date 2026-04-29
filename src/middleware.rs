use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, header},
    middleware::Next,
    response::Response,
};

use crate::env::Env;

// `script-src 'none'` because templates ship zero JavaScript; if HTMX is added
// later (per CLAUDE.md) loosen to 'self' at that point.
const CSP: &str = concat!(
    "default-src 'self'; ",
    "script-src 'none'; ",
    "style-src 'self'; ",
    "img-src 'self' data:; ",
    "font-src 'self'; ",
    "connect-src 'self'; ",
    "base-uri 'self'; ",
    "form-action 'self'; ",
    "frame-ancestors 'none'; ",
    "object-src 'none'",
);

const PERMISSIONS_POLICY: &str = "camera=(), microphone=(), geolocation=(), interest-cohort=()";

// `preload` is intentionally omitted: yoshua.fly.dev is on a domain we don't
// own (Fly's apex), so submission to the HSTS preload list isn't possible.
// Once we cut over to a custom domain we can opt in by adding `preload` and
// submitting at hstspreload.org — but that's a multi-month one-way commitment
// across all subdomains, so don't add it speculatively.
const HSTS: &str = "max-age=31536000; includeSubDomains";

pub async fn security_headers(State(env): State<Env>, req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    let static_headers: [(HeaderName, &'static str); 7] = [
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "strict-origin-when-cross-origin"),
        (header::X_FRAME_OPTIONS, "DENY"),
        (header::CONTENT_SECURITY_POLICY, CSP),
        (
            HeaderName::from_static("permissions-policy"),
            PERMISSIONS_POLICY,
        ),
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ),
        (
            HeaderName::from_static("cross-origin-resource-policy"),
            "same-origin",
        ),
    ];

    for (name, value) in static_headers {
        headers.insert(name, HeaderValue::from_static(value));
    }

    if env == Env::Production {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static(HSTS),
        );
    }

    response
}
