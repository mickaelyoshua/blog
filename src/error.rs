use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tracing::{error, warn};

use crate::templates::ErrorTemplate;

pub enum AppError {
    NotFound,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => {
                warn!("Not Found");
                (StatusCode::NOT_FOUND, "Página não encontrada")
            }
            AppError::Internal(e) => {
                error!(error = %e, "Internal Error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erro interno do servidor",
                )
            }
        };

        let html = ErrorTemplate {
            status: status.as_u16(),
            message: message.to_string(),
            active_nav: "",
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

impl From<serde_yml::Error> for AppError {
    fn from(e: serde_yml::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
