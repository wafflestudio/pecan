use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxToolError {
    #[error("File operation failed: {0}")]
    FileOperationFailed(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}
