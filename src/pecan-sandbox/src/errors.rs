use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Duplicate sandbox ID: {0}")]
    DuplicateSandboxId(Uuid),
}

#[derive(Error, Debug)]
pub enum SandboxManagerError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Sandbox destruction failed: {0}")]
    SandboxDestructionFailed(String),
    #[error("Sandbox creation failed: {0}")]
    SandboxCreationFailed(String),
    #[error("Tool initialization failed: {0}")]
    ToolInitializationFailed(String),
    #[error("Semaphore closed: {0}")]
    SemaphoreClosed(String),
    #[error("No sandbox available from idle queue")]
    NoSandboxAvailable,
    #[error("File operation failed: {0}")]
    FileOperationFailed(String),
    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),
    #[error("Failed to return sandbox to idle queue: {0}")]
    QueueFull(String),
    #[error("Sandbox execution failed: {0}")]
    ExecutionFailed(String),
}
