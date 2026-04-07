use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::get;
use pecan_core::code_execution::AsyncCodeExecutionResult;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};

use crate::api::handler::webhook_handler;
use crate::api::routes::{judge_routes, manager_routes};
use crate::application::state::SharedState;

pub async fn start(state: SharedState, webhook_rx: Receiver<AsyncCodeExecutionResult>) {
    let cors_layer = CorsLayer::new().allow_origin(Any);

    let router = Router::new()
        .route("/v1/health", get(health_handler))
        .nest("/v1/judge", judge_routes::routes())
        .nest("/v1/manager", manager_routes::routes())
        .with_state(Arc::clone(&state))
        .layer(cors_layer);

    let addr = SocketAddr::from_str(&format!(
        "{}:{}",
        state.config.server.host, state.config.server.port
    ))
    .unwrap();

    tracing::info!("Listening on {}", addr);

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("\nShutdown signal received, starting graceful shutdown...");
    };

    let cancel_token = CancellationToken::new();

    let task_loop_token = cancel_token.child_token();
    let task_loop_service = Arc::clone(&state.service);

    let webhook_token = cancel_token.child_token();

    tracing::info!("Spawning background services...");
    tokio::spawn(async move {
        webhook_handler::webhook_handler_loop(webhook_rx, webhook_token).await;
    });

    tokio::spawn(async move {
        task_loop_service.run_task_loop(task_loop_token).await;
    });

    let server = axum::serve(listener, router).with_graceful_shutdown(async {
        shutdown_signal.await;
    });

    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }

    cancel_token.cancel();

    tracing::info!("Cleaning up resources...");
    if let Err(e) = state.service.shutdown().await {
        tracing::error!("Error during cleanup: {}", e);
    } else {
        tracing::info!("Cleanup completed successfully");
    }
}

pub async fn health_handler() -> impl IntoResponse {
    "OK"
}
