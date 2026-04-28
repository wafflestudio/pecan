use pecan_core::code_execution::{
    CodeExecutionRequest, CodeExecutionRequestLazy, CodeExecutionStatus,
};
use pecan_core::errors::CoreExecutionError;
use uuid::Uuid;

use crate::api::error::APIError;
use crate::application::state::SharedState;
use crate::domain::models::judge::{
    JudgeAsyncRequest, JudgeAsyncResponse, JudgeRequest, JudgeResponse, JudgeStatus,
};

/// Process single judge request and returns judge response
pub async fn judge(request: JudgeRequest, state: &SharedState) -> Result<JudgeResponse, APIError> {
    let service = &state.service;
    let result = service
        .execute(CodeExecutionRequest {
            language: request.language.as_str().into(),
            code: request.code,
            input: request.stdin,
            timeout: request.time_limit,
            memory_limit: request.memory_limit,
        })
        .await
        .map_err(|e| match e {
            CoreExecutionError::ServiceBusy(msg) => APIError::ServiceBusy(msg),
            e => APIError::InternalError(e.to_string()),
        })?;

    let status = match result.status {
        CodeExecutionStatus::Success => {
            if result.stdout == request.desired_stdout {
                JudgeStatus::Accepted
            } else {
                JudgeStatus::WrongAnswer
            }
        }
        CodeExecutionStatus::CompileError => JudgeStatus::CompileError,
        CodeExecutionStatus::RuntimeError => JudgeStatus::RuntimeError,
        CodeExecutionStatus::TimeLimitExceeded => JudgeStatus::TimeLimitExceeded,
        CodeExecutionStatus::MemoryLimitExceeded => JudgeStatus::MemoryLimitExceeded,
        CodeExecutionStatus::InternalError => JudgeStatus::InternalError,
    };

    Ok(JudgeResponse {
        code: status.clone().into_status_code(),
        status,
        stdout: result.stdout,
        stderr: result.stderr,
        time: result.time,
        memory: result.memory,
    })
}

/// Enqueue new judge request for lazy execution (later returned by webhook)
pub async fn judge_async(
    request: JudgeAsyncRequest,
    state: &SharedState,
) -> Result<JudgeAsyncResponse, APIError> {
    let service = &state.service;
    let request_id = Uuid::new_v4();

    service
        .execute_async(CodeExecutionRequestLazy {
            request_id,
            webhook_url: request.webhook_url,
            send_failed_count: 0,
            desired_stdout: request.desired_stdout,
            req: CodeExecutionRequest {
                language: request.language.as_str().into(),
                code: request.code,
                input: request.stdin,
                timeout: request.time_limit,
                memory_limit: request.memory_limit,
            },
        })
        .await
        .map_err(|e| APIError::InternalError(e.to_string()))?;

    Ok(JudgeAsyncResponse { request_id })
}
