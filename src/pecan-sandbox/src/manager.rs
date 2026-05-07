//! Sandbox manager holds all initialized sandboxes, tracks their status,
//! and stores actual tool information based on build configuration

use std::process::Stdio;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::errors::SandboxManagerError;
use crate::sandbox::{
    Sandbox, SandboxExecutionOptions, SandboxExecutionResult, SandboxExecutionStatus, SandboxStatus,
};
use crate::tools::common::ISandboxTool;
use crate::tools::{SandboxTool, build_tool};

pub static MAX_PREWARMED_SANDBOXES: OnceLock<usize> = OnceLock::new();
/// Maximum seconds a sandbox may stay in `Running` before the recovery loop reaps it.
/// Acts as a safety net behind `SandboxGuard`'s RAII cleanup.
pub static MAX_RUNNING_SANDBOX_SECS: OnceLock<u64> = OnceLock::new();

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

    MAX_RUNNING_SANDBOX_SECS
        .set(
            std::env::var("MAX_RUNNING_SANDBOX_SECS")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap(),
        )
        .map_err(|e| SandboxManagerError::InternalError(e.to_string()))?;

    Ok(())
}

/// RAII guard that owns a `Running` sandbox for the duration of an execution.
///
/// Created via `arm`. Use `complete_idle` on the success path to return the
/// sandbox to the idle queue, or `complete_error` to mark it failed. If the
/// guard is dropped without either being called (e.g. the calling future is
/// cancelled, or an early-return path forgot to call them), `Drop` flips the
/// sandbox to `Error` so the recovery loop can recycle it.
struct SandboxGuard<'a> {
    sb: Arc<Sandbox>,
    idle_tx: &'a mpsc::UnboundedSender<Uuid>,
    armed: bool,
}

impl<'a> SandboxGuard<'a> {
    fn arm(sb: Arc<Sandbox>, idle_tx: &'a mpsc::UnboundedSender<Uuid>) -> Self {
        sb.set_running();
        Self {
            sb,
            idle_tx,
            armed: true,
        }
    }

    fn sandbox(&self) -> &Arc<Sandbox> {
        &self.sb
    }

    fn complete_idle(mut self) -> Result<(), SandboxManagerError> {
        self.sb.set_idle();
        self.armed = false;
        self.idle_tx
            .send(self.sb.id)
            .map_err(|e| SandboxManagerError::QueueFull(e.to_string()))
    }

    fn complete_error(mut self) {
        self.sb.set_error();
        self.armed = false;
    }
}

impl<'a> Drop for SandboxGuard<'a> {
    fn drop(&mut self) {
        if self.armed {
            self.sb.set_error();
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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
        let _permit = timeout(
            Duration::from_secs_f64(options.time_limit),
            self.permits.acquire(),
        )
        .await
        .map_err(|_| SandboxManagerError::SemaphoreAcquireTimeout)?
        .map_err(|e| {
            SandboxManagerError::SemaphoreClosed(format!("Semaphore acquisition failed: {}", e))
        })?;

        let claim_deadline = Duration::from_secs_f64(options.time_limit);
        let sb = timeout(claim_deadline, async {
            loop {
                let sb_id = {
                    let mut rx = self.idle_rx.lock().await;
                    rx.recv()
                        .await
                        .ok_or(SandboxManagerError::NoSandboxAvailable)?
                };

                match self.sandboxes.get(&sb_id) {
                    Some(sb) => {
                        if sb.status() == SandboxStatus::Idle {
                            return Ok::<Arc<Sandbox>, SandboxManagerError>(Arc::clone(&sb));
                        } else {
                            continue;
                        }
                    }
                    None => continue,
                }
            }
        })
        .await
        .map_err(|_| SandboxManagerError::IdleQueueTimeout)??;

        let guard = SandboxGuard::arm(sb, &self.idle_tx);
        let sb = Arc::clone(guard.sandbox());

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
                    guard.complete_error();
                    return Err(SandboxManagerError::FileOperationFailed(e.to_string()));
                }
            }
        }

        if let Some(compile_options) = &options.compile_options {
            let compile_cmd = match Command::new(&compile_options.compiler_path)
                .args(&compile_options.args)
                .envs(compile_options.env.iter().flatten())
                .current_dir(sb.inner.get_path())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    guard.complete_error();
                    return Err(SandboxManagerError::CommandExecutionFailed(e.to_string()));
                }
            };

            let compile_result = match timeout(
                Duration::from_secs_f64(options.compile_timeout),
                compile_cmd.wait_with_output(),
            )
            .await
            {
                Ok(Ok(output)) => output,
                Ok(Err(e)) => {
                    guard.complete_error();
                    return Err(SandboxManagerError::CommandExecutionFailed(e.to_string()));
                }
                Err(_) => {
                    guard.complete_error();
                    return Err(SandboxManagerError::CompileTimeout);
                }
            };

            if !compile_result.status.success() {
                if let Err(e) = guard.complete_idle() {
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
                    guard.complete_error();
                    return Err(SandboxManagerError::FileOperationFailed(e.to_string()));
                }
            }
        }

        match &result {
            Ok(_) => {
                if let Err(e) = guard.complete_idle() {
                    drop(_permit);
                    return Err(SandboxManagerError::QueueFull(format!(
                        "Idle queue is full or closed: {}",
                        e
                    )));
                }
            }
            Err(_) => {
                guard.complete_error();
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
        let target_num = num.min(
            MAX_PREWARMED_SANDBOXES.get_or_init(|| 1000) - self.available_sandboxes_count().await,
        );

        self.replenish_destroyed(target_num).await?;
        self.permits.add_permits(target_num);

        Ok(())
    }

    /// Create `num` fresh sandboxes and enqueue them as idle, **without** touching
    /// the semaphore permit count. Used by the recovery loop when replacing a
    /// sandbox whose permit was already released by the caller's RAII drop.
    async fn replenish_destroyed(&self, num: usize) -> Result<(), SandboxManagerError> {
        for _ in 0..num {
            let sb = create_sandbox(&self.tool)
                .await
                .map_err(|e| SandboxManagerError::SandboxCreationFailed(e.to_string()))?;
            self.sandboxes.insert(sb.id, Arc::clone(&sb));
            self.idle_tx
                .send(sb.id)
                .map_err(|e| SandboxManagerError::QueueFull(e.to_string()))?;
        }
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
        // mark stuck Running sandboxes as Error so they get recycled below.
        // SandboxGuard's Drop normally handles cancelled futures, but this is a
        // safety net for any path where the future genuinely hangs.
        let now = now_secs();
        let threshold = *MAX_RUNNING_SANDBOX_SECS.get_or_init(|| 600);
        for entry in self.sandboxes.iter() {
            let sb = entry.value();
            if sb.status() == SandboxStatus::Running
                && sb
                    .running_for_secs(now)
                    .map(|s| s > threshold)
                    .unwrap_or(false)
            {
                sb.set_error();
            }
        }

        // destroy sandbox with error status
        let error_sandbox_ids: Vec<Uuid> = self
            .sandboxes
            .iter()
            .filter(|entry| entry.value().status() == SandboxStatus::Error)
            .map(|entry| *entry.key())
            .collect();

        for id in &error_sandbox_ids {
            if let Err(e) = self.destroy_sandbox(*id).await {
                eprintln!("Warning: failed to destroy error sandbox {}: {}", id, e);
            }
        }

        // Replenish without touching the semaphore: the permit for each error
        // sandbox was already released by the caller's RAII drop.
        if let Err(e) = self.replenish_destroyed(error_sandbox_ids.len()).await {
            eprintln!("Warning: failed to replenish destroyed sandboxes: {}", e);
        }
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use tokio::sync::mpsc;

    use super::{SandboxGuard, SandboxManager};
    use crate::sandbox::SandboxStatus;

    #[tokio::test]
    async fn manager_starts_empty_with_zero_prewarm() {
        let manager = SandboxManager::new(0).await.expect("manager init");

        assert_eq!(manager.available_sandboxes_count().await, 0);
        assert_eq!(manager.idle_sandboxes_count().await, 0);
        assert_eq!(manager.running_sandboxes_count().await, 0);
        assert_eq!(manager.error_sandboxes_count().await, 0);
        assert!(manager.list_ids().is_empty());
    }

    #[tokio::test]
    async fn manager_prewarm_creates_idle_sandboxes() {
        let manager = SandboxManager::new(2).await.expect("manager init");

        assert_eq!(manager.available_sandboxes_count().await, 2);
        assert_eq!(manager.idle_sandboxes_count().await, 2);
        assert_eq!(manager.running_sandboxes_count().await, 0);
        assert_eq!(manager.error_sandboxes_count().await, 0);
        assert_eq!(manager.list_ids().len(), 2);
    }

    #[tokio::test]
    async fn manager_adds_and_removes_many_sandboxes() {
        let manager = SandboxManager::new(1).await.expect("manager init");

        assert_eq!(manager.available_sandboxes_count().await, 1);
        assert_eq!(manager.idle_sandboxes_count().await, 1);

        manager
            .add_new_prewarmed_sandbox(4)
            .await
            .expect("add prewarmed");
        assert_eq!(manager.available_sandboxes_count().await, 5);
        assert_eq!(manager.idle_sandboxes_count().await, 5);

        manager.remove_idle_sandbox(3).await.expect("remove idle");
        assert_eq!(manager.available_sandboxes_count().await, 2);
        assert_eq!(manager.idle_sandboxes_count().await, 2);
        assert_eq!(manager.list_ids().len(), 2);

        manager
            .add_new_prewarmed_sandbox(2)
            .await
            .expect("add prewarmed");
        assert_eq!(manager.available_sandboxes_count().await, 4);
        assert_eq!(manager.idle_sandboxes_count().await, 4);

        manager.remove_idle_sandbox(4).await.expect("remove idle");
        assert_eq!(manager.available_sandboxes_count().await, 0);
        assert_eq!(manager.idle_sandboxes_count().await, 0);
        assert!(manager.list_ids().is_empty());
    }

    #[tokio::test]
    async fn guard_drop_marks_error_when_unarmed() {
        let manager = SandboxManager::new(1).await.expect("manager init");
        let id = manager.list_ids()[0];
        let sb = Arc::clone(manager.sandboxes.get(&id).unwrap().value());
        let (tx, _rx) = mpsc::unbounded_channel();

        {
            let _guard = SandboxGuard::arm(Arc::clone(&sb), &tx);
            assert_eq!(sb.status(), SandboxStatus::Running);
        } // guard dropped without complete_*

        assert_eq!(sb.status(), SandboxStatus::Error);
    }

    #[tokio::test]
    async fn guard_complete_idle_returns_to_queue() {
        let manager = SandboxManager::new(1).await.expect("manager init");
        let id = manager.list_ids()[0];
        let sb = Arc::clone(manager.sandboxes.get(&id).unwrap().value());
        let (tx, mut rx) = mpsc::unbounded_channel();

        let guard = SandboxGuard::arm(Arc::clone(&sb), &tx);
        assert_eq!(sb.status(), SandboxStatus::Running);
        guard.complete_idle().expect("complete idle");

        assert_eq!(sb.status(), SandboxStatus::Idle);
        assert_eq!(rx.try_recv().ok(), Some(id));
    }

    #[tokio::test]
    async fn guard_complete_error_does_not_enqueue() {
        let manager = SandboxManager::new(1).await.expect("manager init");
        let id = manager.list_ids()[0];
        let sb = Arc::clone(manager.sandboxes.get(&id).unwrap().value());
        let (tx, mut rx) = mpsc::unbounded_channel();

        let guard = SandboxGuard::arm(Arc::clone(&sb), &tx);
        guard.complete_error();

        assert_eq!(sb.status(), SandboxStatus::Error);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn loop_does_not_drift_permits_on_error_recovery() {
        let manager = SandboxManager::new(3).await.expect("manager init");
        let initial_permits = manager.permits.available_permits();
        assert_eq!(initial_permits, 3);

        // simulate two requests that ended with `complete_error`: caller's
        // permit was already released, sandbox sits in Error awaiting recovery.
        let ids = manager.list_ids();
        for id in ids.iter().take(2) {
            manager.sandboxes.get(id).unwrap().value().set_error();
        }

        manager._loop().await;

        assert_eq!(manager.error_sandboxes_count().await, 0);
        assert_eq!(manager.available_sandboxes_count().await, 3);
        assert_eq!(
            manager.permits.available_permits(),
            initial_permits,
            "permits must not drift up when replacing error sandboxes"
        );
    }

    #[tokio::test]
    async fn loop_reaps_stuck_running_after_threshold() {
        let manager = SandboxManager::new(2).await.expect("manager init");
        let id = manager.list_ids()[0];

        {
            let sb = manager.sandboxes.get(&id).unwrap();
            sb.set_running();
            // backdate the running timestamp far past the 600s default threshold
            sb.running_since.store(1, Ordering::Release);
        }

        manager._loop().await;

        // stuck Running sandbox should have been replaced; total count preserved
        assert_eq!(manager.available_sandboxes_count().await, 2);
        assert_eq!(manager.running_sandboxes_count().await, 0);
        assert_eq!(manager.error_sandboxes_count().await, 0);
        assert!(!manager.sandboxes.contains_key(&id));
    }
}
