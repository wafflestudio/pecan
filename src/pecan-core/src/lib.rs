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
) -> Result<Service, CoreServiceError> {
    let service = Service::new(ServiceSpec {
        enable_bg_worker_loop: true,
        max_queue_size,
        max_concurrent_executions,
    })
    .await?;

    Ok(service)
}
