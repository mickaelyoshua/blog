use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::{error, warn};

pub enum AppError {
    NotFound,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => {
                warn!("Not Found");
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
            AppError::Internal(e) => {
                error!(error = %e, "Internal Error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal").into_response()
            }
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<askama::Error> for AppError {
    fn from(e: askama::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
