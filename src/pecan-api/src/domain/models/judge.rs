use serde::{Deserialize, Serialize};

pub type JudgeStatusCode = u16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JudgeStatus {
    Accepted,
    WrongAnswer,
    CompileError,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    InternalError,
}

impl JudgeStatus {
    pub fn into_status_code(self) -> JudgeStatusCode {
        match self {
            JudgeStatus::Accepted => 0,
            JudgeStatus::WrongAnswer => 1,
            JudgeStatus::CompileError => 2,
            JudgeStatus::RuntimeError => 3,
            JudgeStatus::TimeLimitExceeded => 4,
            JudgeStatus::MemoryLimitExceeded => 5,
            JudgeStatus::InternalError => 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeRequest {
    pub code: String,
    pub language: String,
    pub stdin: String,
    pub desired_stdout: String,
    pub time_limit: f64,
    pub memory_limit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeResponse {
    pub code: JudgeStatusCode,
    pub status: JudgeStatus,
    pub stdout: String,
    pub stderr: String,
    pub time: f64,
    pub memory: f64,
}
