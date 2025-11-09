use axum::Json;
use axum::extract::State;

use crate::api::error::APIError;
use crate::application::service::judge_service;
use crate::application::state::SharedState;
use crate::domain::models::judge::{JudgeRequest, JudgeResponse};

pub async fn judge_single(
    State(state): State<SharedState>,
    Json(request): Json<JudgeRequest>,
) -> Result<Json<JudgeResponse>, APIError> {
    let response = judge_service::judge(request, &state).await?;
    Ok(Json(response))
}
