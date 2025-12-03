use std::sync::Arc;

use pecan_sandbox::manager::SandboxManager;
use pecan_sandbox::sandbox::SandboxExecutionStatus;

use crate::errors::CoreExecutionError;
use crate::toolchains::Language;
use crate::toolchains::sandbox_options::build_sandbox_execution_option;

pub struct CodeExecutionRequest {
    pub language: Language,
    pub code: String,
    pub input: String,
    pub timeout: f64,
    pub memory_limit: f64,
}

pub enum CodeExecutionStatus {
    Success,
    CompileError,
    RuntimeError,
    InternalError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
}

pub struct CodeExecutionResult {
    pub status: CodeExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub time: f64,
    pub memory: f64,
}

/// simply execute function provided by sandbox manager
pub async fn execute(
    sandbox_manager: &Arc<SandboxManager>,
    request: CodeExecutionRequest,
) -> Result<CodeExecutionResult, CoreExecutionError> {
    let sandbox_execution_options = build_sandbox_execution_option(
        request.language,
        request.code,
        request.input,
        request.timeout,
        request.memory_limit,
    )?;

    let result = match sandbox_manager
        .execute_via_manager(&sandbox_execution_options)
        .await
    {
        Ok(result) => result,
        Err(e) => return Err(CoreExecutionError::InternalError(e.to_string())),
    };

    return Ok(CodeExecutionResult {
        status: match result.status {
            SandboxExecutionStatus::Success => CodeExecutionStatus::Success,
            SandboxExecutionStatus::CompileError => CodeExecutionStatus::CompileError,
            SandboxExecutionStatus::RuntimeError => CodeExecutionStatus::RuntimeError,
            SandboxExecutionStatus::TimeLimitExceeded => CodeExecutionStatus::TimeLimitExceeded,
            SandboxExecutionStatus::MemoryLimitExceeded => CodeExecutionStatus::MemoryLimitExceeded,
        },
        stdout: result.stdout,
        stderr: result.stderr,
        time: result.time,
        memory: result.memory,
    });
}
