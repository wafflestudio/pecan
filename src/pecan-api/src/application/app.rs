use std::sync::Arc;

use crate::api::server;
use crate::application::config;
use crate::application::state::AppState;

pub async fn run() {
    let config = config::load_config();

    let (service, webhook_rx) = pecan_core::init(
        config.service.max_queue_size,
        config.service.max_concurrent_executions,
        config.service.max_queue_size as usize,
    )
    .await
    .unwrap();

    let shared_state = Arc::new(AppState {
        config,
        service: Arc::new(service),
    });

    server::start(shared_state, webhook_rx).await;
}
