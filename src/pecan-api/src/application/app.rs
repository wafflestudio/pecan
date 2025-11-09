use std::sync::Arc;

use crate::api::server;
use crate::application::config;
use crate::application::state::AppState;

pub async fn run() {
    let config = config::load_config();

    let service = pecan_core::init(
        config.service.max_queue_size,
        config.service.max_concurrent_executions,
    )
    .await
    .unwrap();

    let shared_state = Arc::new(AppState {
        config,
        service: service,
    });

    server::start(shared_state).await;
}
