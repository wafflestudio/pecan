use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxToolError {
    #[error("Unknown error: {0}")]
    UnknownError(String),
}
