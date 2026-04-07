use axum::Router;
use axum::routing::post;

use crate::api::handler::judge_handler;
use crate::application::state::SharedState;

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/judge-single", post(judge_handler::judge_single))
        .route(
            "/judge-single-async",
            post(judge_handler::judge_single_async),
        )
}
