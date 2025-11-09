use std::sync::Arc;

use pecan_sandbox::manager::SandboxManager;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::code_execution::{CodeExecutionRequest, CodeExecutionResult, execute};
use crate::errors::{CoreExecutionError, CoreServiceError};
use crate::utils::queue::Queue;

pub struct ServiceLoop {
    manager_handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

pub struct Service {
    /// for lazy execution
    task_queue: Arc<Queue<CodeExecutionRequest>>,
    /// sandbox manager for executing code
    sandbox_manager: Arc<SandboxManager>,
    /// sandbox manager loop
    service_loop: Option<ServiceLoop>,
}

pub struct ServiceSpec {
    pub enable_bg_worker_loop: bool,
    pub max_queue_size: u32,
    pub max_concurrent_executions: u32,
}

impl Service {
    pub async fn new(spec: ServiceSpec) -> Result<Arc<Self>, CoreServiceError> {
        let ServiceSpec {
            enable_bg_worker_loop,
            max_queue_size,
            max_concurrent_executions,
        } = spec;

        let task_queue = Arc::new(Queue::bounded(max_queue_size as usize));

        let sandbox_manager = SandboxManager::new(max_concurrent_executions as usize)
            .await
            .map_err(|e| CoreServiceError::InternalError(e.to_string()))?;

        let ct = CancellationToken::new();
        let ct_m_loop = ct.child_token();
        let sandbox_manager_for_loop = Arc::clone(&sandbox_manager);

        let service_loop = if enable_bg_worker_loop {
            Some(ServiceLoop {
                manager_handle: tokio::spawn(async move {
                    sandbox_manager_for_loop.run_loop(ct_m_loop).await
                }),
                cancel_token: ct,
            })
        } else {
            None
        };

        Ok(Arc::new(Self {
            task_queue,
            sandbox_manager,
            service_loop,
        }))
    }

    pub async fn get_available_sandboxes_count(&self) -> usize {
        self.sandbox_manager.available_sandboxes_count().await
    }

    pub async fn get_idle_sandboxes_count(&self) -> usize {
        self.sandbox_manager.idle_sandboxes_count().await
    }

    pub async fn get_running_sandboxes_count(&self) -> usize {
        self.sandbox_manager.running_sandboxes_count().await
    }

    pub async fn get_error_sandboxes_count(&self) -> usize {
        self.sandbox_manager.error_sandboxes_count().await
    }

    pub async fn execute(
        &self,
        request: CodeExecutionRequest,
    ) -> Result<CodeExecutionResult, CoreExecutionError> {
        let result = execute(&self.sandbox_manager, request).await?;

        Ok(result)
    }

    pub async fn shutdown(&self) -> Result<(), CoreServiceError> {
        self.task_queue.close();

        let _ = self
            .sandbox_manager
            .teardown()
            .await
            .map_err(|e| CoreServiceError::InternalError(e.to_string()))?;

        if let Some(service_loop) = &self.service_loop {
            service_loop.cancel_token.cancel();
            service_loop.manager_handle.abort();
        }
        Ok(())
    }
}
