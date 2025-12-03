//! Sandbox manager holds all initialized sandboxes, tracks their status,
//! and stores actual tool information based on build configuration

use std::process::Stdio;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use dashmap::DashMap;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::errors::SandboxManagerError;
use crate::sandbox::{
    Sandbox, SandboxExecutionOptions, SandboxExecutionResult, SandboxExecutionStatus, SandboxStatus,
};
use crate::tools::common::ISandboxTool;
use crate::tools::{SandboxTool, build_tool};

pub static MAX_PREWARMED_SANDBOXES: OnceLock<usize> = OnceLock::new();

/// initialize manager config based on deployed environment
fn init_manager_config() -> Result<(), SandboxManagerError> {
    MAX_PREWARMED_SANDBOXES
        .set(
            std::env::var("MAX_PREWARMED_SANDBOXES")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap(),
        )
        .map_err(|e| SandboxManagerError::InternalError(e.to_string()))?;

    Ok(())
}

pub struct SandboxManager {
    pub tool: SandboxTool,
    sandboxes: DashMap<Uuid, Arc<Sandbox>>,
    idle_tx: mpsc::UnboundedSender<Uuid>,
    idle_rx: Mutex<mpsc::UnboundedReceiver<Uuid>>,
    permits: Arc<Semaphore>,
}

pub async fn create_sandbox(tool: &SandboxTool) -> Result<Arc<Sandbox>, SandboxManagerError> {
    let inner = tool
        .build_inner()
        .await
        .map_err(|e| SandboxManagerError::SandboxCreationFailed(e.to_string()))?;

    let sb = Arc::new(Sandbox::new(inner));

    Ok(sb)
}

impl SandboxManager {
    pub async fn new(prewarm: usize) -> Result<Arc<Self>, SandboxManagerError> {
        let _ = init_manager_config(); // ignore error

        let (tx, rx) = mpsc::unbounded_channel::<Uuid>();
        let map = DashMap::new();
        let tool = build_tool().map_err(|e| {
            SandboxManagerError::ToolInitializationFailed(format!("Failed to build tool: {}", e))
        })?;

        for _ in 0..prewarm {
            let sb = create_sandbox(&tool)
                .await
                .map_err(|e| SandboxManagerError::SandboxCreationFailed(e.to_string()))?;
            map.insert(sb.id, Arc::clone(&sb));
            tx.send(sb.id)
                .map_err(|e| SandboxManagerError::QueueFull(e.to_string()))?;
        }

        let mgr = Arc::new(Self {
            tool,
            sandboxes: map,
            idle_tx: tx,
            idle_rx: Mutex::new(rx),
            permits: Arc::new(Semaphore::new(prewarm)),
        });

        Ok(mgr)
    }

    pub fn list_ids(&self) -> Vec<Uuid> {
        self.sandboxes.iter().map(|e| *e.key()).collect()
    }

    /// 1. claim an available sandbox from idle channel, mark it as running
    /// 2. write files into sandbox working directory
    /// 3. compile code if necessary, outside sandboxed environment
    /// 4. execute and retrieve results
    /// 5. mark sandbox as idle, return to idle queue
    pub async fn execute_via_manager(
        &self,
        options: &SandboxExecutionOptions,
    ) -> Result<SandboxExecutionResult, SandboxManagerError> {
        let _permit = self.permits.acquire().await.map_err(|e| {
            SandboxManagerError::SemaphoreClosed(format!("Semaphore acquisition failed: {}", e))
        })?;

        let sb = loop {
            let sb_id = {
                let mut rx = self.idle_rx.lock().await;
                rx.recv()
                    .await
                    .ok_or(SandboxManagerError::NoSandboxAvailable)?
            };

            match self.sandboxes.get(&sb_id) {
                Some(sb) => {
                    if sb.status() == SandboxStatus::Idle {
                        break Arc::clone(&sb);
                    } else {
                        continue;
                    }
                }
                None => continue,
            };
        };

        sb.set_running();

        if let Some(additional_file_options) = &options.additional_file_options {
            for additional_file_option in additional_file_options {
                if let Err(e) = self
                    .tool
                    .add_file_wd(
                        &sb.inner,
                        &additional_file_option.file_name,
                        &additional_file_option.file_content,
                    )
                    .await
                {
                    sb.set_error();
                    return Err(SandboxManagerError::FileOperationFailed(e.to_string()));
                }
            }
        }

        if let Some(compile_options) = &options.compile_options {
            let compile_cmd = Command::new(&compile_options.compiler_path)
                .args(&compile_options.args)
                .envs(compile_options.env.clone().unwrap_or_default())
                .current_dir(sb.inner.get_path())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| SandboxManagerError::CommandExecutionFailed(e.to_string()))?;

            let compile_result = compile_cmd
                .wait_with_output()
                .await
                .map_err(|e| SandboxManagerError::CommandExecutionFailed(e.to_string()))?;

            if !compile_result.status.success() {
                sb.set_idle();
                if let Err(e) = self.idle_tx.send(sb.id) {
                    eprintln!(
                        "Warning: Failed to return sandbox after compile error: {}",
                        e
                    );
                }

                drop(_permit);

                return Ok(SandboxExecutionResult {
                    status: SandboxExecutionStatus::CompileError,
                    stdout: String::from_utf8_lossy(&compile_result.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&compile_result.stderr).to_string(),
                    time: 0.0,
                    memory: 0.0,
                });
            }
        }

        let result = self.tool.execute(&sb.inner, options).await;

        if let Some(additional_file_options) = &options.additional_file_options {
            for additional_file_option in additional_file_options {
                if let Err(e) = self
                    .tool
                    .remove_file_wd(&sb.inner, &additional_file_option.file_name)
                    .await
                {
                    sb.set_error();
                    return Err(SandboxManagerError::FileOperationFailed(e.to_string()));
                }
            }
        }

        match &result {
            Ok(_) => {
                sb.set_idle();

                if let Err(e) = self.idle_tx.send(sb.id) {
                    sb.set_error();
                    drop(_permit);

                    return Err(SandboxManagerError::QueueFull(format!(
                        "Idle queue is full or closed: {}",
                        e
                    )));
                }
            }
            Err(_) => {
                sb.set_error();
            }
        }

        result.map_err(|e| SandboxManagerError::ExecutionFailed(e.to_string()))
    }

    pub async fn available_sandboxes_count(&self) -> usize {
        self.sandboxes.len()
    }

    pub async fn idle_sandboxes_count(&self) -> usize {
        self.sandboxes
            .iter()
            .filter(|entry| entry.value().status() == SandboxStatus::Idle)
            .count()
    }

    pub async fn running_sandboxes_count(&self) -> usize {
        self.sandboxes
            .iter()
            .filter(|entry| entry.value().status() == SandboxStatus::Running)
            .count()
    }

    pub async fn error_sandboxes_count(&self) -> usize {
        self.sandboxes
            .iter()
            .filter(|entry| entry.value().status() == SandboxStatus::Error)
            .count()
    }

    pub async fn add_new_prewarmed_sandbox(&self, num: usize) -> Result<(), SandboxManagerError> {
        let target_num = num
            .min(MAX_PREWARMED_SANDBOXES.get().unwrap() - self.available_sandboxes_count().await);

        for _ in 0..target_num {
            let sb = create_sandbox(&self.tool)
                .await
                .map_err(|e| SandboxManagerError::SandboxCreationFailed(e.to_string()))?;
            self.sandboxes.insert(sb.id, Arc::clone(&sb));
            self.idle_tx.send(sb.id).expect("idle queue closed");
        }

        self.permits.add_permits(target_num);

        Ok(())
    }

    pub async fn destroy_sandbox(&self, id: Uuid) -> Result<(), SandboxManagerError> {
        if let Some((_, sb)) = self.sandboxes.remove(&id) {
            self.tool
                .destroy_inner(&sb.inner)
                .await
                .map_err(|e| SandboxManagerError::SandboxDestructionFailed(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn remove_idle_sandbox(&self, num: usize) -> Result<(), SandboxManagerError> {
        let target_num = num.min(self.idle_sandboxes_count().await);

        for _ in 0..target_num {
            let sb_id = {
                let mut rx = self.idle_rx.lock().await;
                rx.recv()
                    .await
                    .ok_or(SandboxManagerError::NoSandboxAvailable)?
            };

            if self.sandboxes.contains_key(&sb_id) {
                self.destroy_sandbox(sb_id).await?;
            } else {
                continue;
            }
        }

        self.permits.forget_permits(target_num);

        Ok(())
    }

    pub async fn teardown(&self) -> Result<(), SandboxManagerError> {
        let ids: Vec<Uuid> = self.sandboxes.iter().map(|e| *e.key()).collect();
        for id in ids {
            let _ = self.destroy_sandbox(id).await;
        }
        Ok(())
    }

    async fn _loop(&self) {
        // destroy sandbox with error status
        let error_sandbox_ids: Vec<Uuid> = self
            .sandboxes
            .iter()
            .filter(|entry| entry.value().status() == SandboxStatus::Error)
            .map(|entry| *entry.key())
            .collect();

        for id in error_sandbox_ids.clone() {
            self.destroy_sandbox(id).await.unwrap();
        }

        self.add_new_prewarmed_sandbox(error_sandbox_ids.len())
            .await
            .unwrap();
    }

    pub async fn run_loop(&self, cancel: CancellationToken) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    break;
                }
                _ = sleep(Duration::from_millis(100)) => {
                    self._loop().await;
                }
            }
        }
    }
}
