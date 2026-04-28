use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tracing::{error, warn};

use crate::templates::{ErrorTemplate, LayoutContext};

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Internal(String),
    BadPost { slug: String, reason: String },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::NotFound => warn!("Not Found"),
            AppError::Internal(e) => error!(error = %e, "Internal Error"),
            AppError::BadPost { slug, reason } => {
                error!(slug = %slug, reason = %reason, "Bad post")
            }
        }

        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Página não encontrada"),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro interno do servidor",
            ),
        };

        // If the error template itself fails to render, fall back to raw HTML.
        // Anything that re-enters AppError-producing code here would loop
        // forever (error → render fail → AppError → render fail → …).
        let html = ErrorTemplate {
            status: status.as_u16(),
            message: message.to_string(),
            layout: LayoutContext::new(""),
        }
        .render()
        .unwrap_or_else(|_| format!("<h1>{}</h1><p>{message}</p>", status.as_u16()));

        (status, Html(html)).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound,
            _ => AppError::Internal(e.to_string()),
        }
    }
}

impl From<askama::Error> for AppError {
    fn from(e: askama::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use std::io;

    #[test]
    fn io_not_found_maps_to_app_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
        match AppError::from(io_err) {
            AppError::NotFound => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn io_permission_denied_maps_to_internal() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "nope");
        match AppError::from(io_err) {
            AppError::Internal(msg) => {
                assert!(msg.contains("nope"), "expected msg propagated, got: {msg}");
            }
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    #[test]
    fn io_other_kinds_map_to_internal() {
        let io_err = io::Error::other("disk on fire");
        match AppError::from(io_err) {
            AppError::Internal(_) => {}
            other => panic!("expected Internal, got {other:?}"),
        }
    }

    async fn body_to_string(resp: Response) -> (StatusCode, String) {
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn not_found_response_is_404_with_pt_br_message() {
        let resp = AppError::NotFound.into_response();
        let (status, body) = body_to_string(resp).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(
            body.contains("Página não encontrada"),
            "user-facing message missing, body was: {body}"
        );
    }

    #[tokio::test]
    async fn internal_response_is_500_and_does_not_leak_reason() {
        let resp = AppError::Internal("connection refused to internal-only-host:5432".into())
            .into_response();
        let (status, body) = body_to_string(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            !body.contains("connection refused"),
            "internal reason leaked into response body: {body}"
        );
        assert!(
            !body.contains("internal-only-host"),
            "internal reason leaked into response body: {body}"
        );
        assert!(body.contains("Erro interno do servidor"));
    }

    #[tokio::test]
    async fn bad_post_response_is_500_and_does_not_leak_reason_or_slug() {
        // BadPost details (slug, reason) are diagnostic — they go to logs
        // but must NOT appear in the user-facing response.
        let resp = AppError::BadPost {
            slug: "leaked-slug".to_string(),
            reason: "yml: very specific parse failure".to_string(),
        }
        .into_response();
        let (status, body) = body_to_string(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            !body.contains("leaked-slug"),
            "BadPost slug leaked into response: {body}"
        );
        assert!(
            !body.contains("very specific parse failure"),
            "BadPost reason leaked into response: {body}"
        );
        assert!(body.contains("Erro interno do servidor"));
    }
}
