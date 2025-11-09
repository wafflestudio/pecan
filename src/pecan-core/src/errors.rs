use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreServiceError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Task queue is full: {0}")]
    TaskQueueFull(String),
}

#[derive(Error, Debug)]
pub enum CoreExecutionError {
    #[error("Not supported language: {0}")]
    NotSupportedLanguage(String),
    #[error("Compile error: {0}")]
    CompileError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("Time limit exceeded: {0}")]
    TimeLimitExceeded(String),
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    #[error("Allocating task error: {0}")]
    AllocatingTaskError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}
