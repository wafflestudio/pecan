use tokio::sync::mpsc::Receiver;

use crate::code_execution::AsyncCodeExecutionResult;
use crate::errors::CoreServiceError;
use crate::service::{Service, ServiceSpec};

pub mod code_execution;
pub mod errors;
pub mod service;
pub mod toolchains;
pub mod utils;

pub const SANDBOX_SOLUTION: &str = pecan_sandbox::tools::SANDBOX_SOLUTION;

pub async fn init(
    max_queue_size: u32,
    max_concurrent_executions: u32,
    webhook_buffer_size: usize,
) -> Result<(Service, Receiver<AsyncCodeExecutionResult>), CoreServiceError> {
    let (service, rx) = Service::new(ServiceSpec {
        enable_bg_worker_loop: true,
        max_queue_size,
        max_concurrent_executions,
        webhook_buffer_size,
    })
    .await?;

    Ok((service, rx))
}
