use crate::api::error::APIError;
use crate::application::state::SharedState;
use crate::domain::models::manager::SandboxStatusResponse;

/// get detailed sandbox service status
pub async fn get_sandbox_status(state: &SharedState) -> Result<SandboxStatusResponse, APIError> {
    let service = &state.service;
    let available_sandboxes_count = service.get_available_sandboxes_count().await;
    let idle_sandboxes_count = service.get_idle_sandboxes_count().await;
    let running_sandboxes_count = service.get_running_sandboxes_count().await;
    let error_sandboxes_count = service.get_error_sandboxes_count().await;

    Ok(SandboxStatusResponse {
        available_sandboxes: available_sandboxes_count,
        idle_sandboxes: idle_sandboxes_count,
        running_sandboxes: running_sandboxes_count,
        error_sandboxes: error_sandboxes_count,
    })
}
