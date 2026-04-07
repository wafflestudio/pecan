use pecan_core::code_execution::{AsyncCodeExecutionResult, CodeExecutionStatus};
use reqwest::Client;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::api::error::APIError;
use crate::domain::models::judge::{JudgeAsyncWebhookResponse, JudgeResponse, JudgeStatus};

async fn send_webhook_request(res: AsyncCodeExecutionResult) -> Result<(), APIError> {
    let client = Client::new();

    match res.result {
        Some(r) => {
            let status = match r.status {
                CodeExecutionStatus::Success => {
                    if r.stdout == res.desired_stdout {
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

            let body = JudgeAsyncWebhookResponse {
                request_id: res.request_id,
                res: JudgeResponse {
                    code: status.clone().into_status_code(),
                    status,
                    stdout: r.stdout,
                    stderr: r.stderr,
                    time: r.time,
                    memory: r.memory,
                },
            };

            let _res = client
                .post(res.webhook_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| APIError::InternalError(e.to_string()));

            Ok(())
        }

        None => Err(APIError::InternalError("".to_string())),
    }
}

pub async fn webhook_handler_loop(
    mut rx: Receiver<AsyncCodeExecutionResult>,
    cancel: CancellationToken,
) {
    tracing::info!("Webhook handler loop started");

    loop {
        select! {
            _ = cancel.cancelled() => {
                tracing::debug!("Webhook handler loop cancelled");
                break;
            }
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        let request_id = msg.request_id;
                        tracing::debug!(
                            request_id = %request_id,
                            "sending webhook request"
                        );
                        match send_webhook_request(msg).await {
                            Ok(_) => tracing::debug!(request_id = %request_id, "webhook delivered"),
                            Err(e) => tracing::error!(request_id = %request_id, error = %e, "webhook delivery failed"),
                        }
                    }
                    None => {
                        tracing::debug!("Webhook channel closed, exiting loop");
                        break;
                    }
                }
            }
        }
    }
}
