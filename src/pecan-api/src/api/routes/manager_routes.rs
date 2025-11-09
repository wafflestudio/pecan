use axum::Router;
use axum::routing::get;

use crate::api::handler::manager_handler;
use crate::application::state::SharedState;

pub fn routes() -> Router<SharedState> {
    Router::new().route("/sandbox-status", get(manager_handler::get_sandbox_status))
}
