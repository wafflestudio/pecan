use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum APIError {
    #[error("Not supported language: {0}")]
    NotSupportedLanguage(String),
    #[error("Compile error: {0}")]
    CompileError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("Time limit exceeded: {0}")]
    TimeLimitExceeded(String),
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    #[error("Allocating task error: {0}")]
    AllocatingTaskError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}
