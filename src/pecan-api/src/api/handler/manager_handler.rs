use axum::Json;
use axum::extract::State;

use crate::api::error::APIError;
use crate::application::service::manager_service;
use crate::application::state::SharedState;
use crate::domain::models::manager::SandboxStatusResponse;

pub async fn get_sandbox_status(
    State(state): State<SharedState>,
) -> Result<Json<SandboxStatusResponse>, APIError> {
    let response = manager_service::get_sandbox_status(&state).await?;
    Ok(Json(response))
}
