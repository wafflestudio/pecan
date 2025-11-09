use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxStatusResponse {
    pub available_sandboxes: usize,
    pub idle_sandboxes: usize,
    pub running_sandboxes: usize,
    pub error_sandboxes: usize,
}
