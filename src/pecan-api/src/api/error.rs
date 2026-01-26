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

#[cfg(test)]
mod tests {
    use super::APIError;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;
    use http::StatusCode;

    #[tokio::test]
    async fn into_response_returns_json_error_body() {
        let error = APIError::NotSupportedLanguage("brain".to_string());

        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body read");
        let decoded: APIError = serde_json::from_slice(&body).expect("json decode");

        assert_eq!(decoded, APIError::NotSupportedLanguage("brain".to_string()));
    }
}
