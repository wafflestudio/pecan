use std::sync::Arc;
use std::time::Duration;

use pecan_sandbox::manager::SandboxManager;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::code_execution::{
    AsyncCodeExecutionResult, CodeExecutionRequest, CodeExecutionRequestLazy, CodeExecutionResult,
    execute,
};
use crate::errors::{CoreExecutionError, CoreServiceError};
use crate::utils::queue::Queue;

pub struct ServiceLoop {
    manager_handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

pub struct Service {
    /// for lazy execution (enqueue only — uses try_push to avoid blocking)
    task_queue: Arc<Queue<CodeExecutionRequestLazy>>,
    /// sender half for pushing completed results to the webhook handler
    task_sender: Sender<AsyncCodeExecutionResult>,
    /// sandbox manager for executing code
    sandbox_manager: Arc<SandboxManager>,
    /// sandbox manager loop
    service_loop: Option<ServiceLoop>,
}

pub struct ServiceSpec {
    pub enable_bg_worker_loop: bool,
    pub max_queue_size: u32,
    pub max_concurrent_executions: u32,
    pub webhook_buffer_size: usize,
}

impl Service {
    pub async fn new(
        spec: ServiceSpec,
    ) -> Result<(Self, Receiver<AsyncCodeExecutionResult>), CoreServiceError> {
        let ServiceSpec {
            enable_bg_worker_loop,
            max_queue_size,
            max_concurrent_executions,
            webhook_buffer_size,
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

        let (tx, rx) = mpsc::channel::<AsyncCodeExecutionResult>(webhook_buffer_size);

        Ok((
            Self {
                task_queue,
                task_sender: tx,
                sandbox_manager,
                service_loop,
            },
            rx,
        ))
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

    pub async fn execute_async(
        &self,
        request: CodeExecutionRequestLazy,
    ) -> Result<(), CoreExecutionError> {
        self.task_queue
            .try_push(request)
            .map_err(|_| CoreExecutionError::InternalError("task queue full or closed".to_string()))
    }

    async fn process_one_task(&self) {
        let task = match self.task_queue.try_pop() {
            Ok(task) => task,
            Err(_) => return,
        };

        let result = match self
            .execute(CodeExecutionRequest {
                language: task.req.language,
                code: task.req.code,
                input: task.req.input,
                timeout: task.req.timeout,
                memory_limit: task.req.memory_limit,
            })
            .await
        {
            Ok(res) => Some(res),
            Err(_) => None,
        };

        let _ = self
            .task_sender
            .send(AsyncCodeExecutionResult {
                request_id: task.request_id,
                webhook_url: task.webhook_url,
                send_failed_count: task.send_failed_count,
                desired_stdout: task.desired_stdout,
                result,
            })
            .await;
    }

    pub async fn run_task_loop(&self, cancel: CancellationToken) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    break;
                }
                _ = sleep(Duration::from_millis(500)) => {
                    self.process_one_task().await;
                }
            }
        }
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
