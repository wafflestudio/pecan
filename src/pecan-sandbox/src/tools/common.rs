use crate::sandbox::{SandboxExecutionOptions, SandboxExecutionResult};
use crate::tools::SandboxInner;
use crate::tools::errors::SandboxToolError;

#[allow(async_fn_in_trait)]
pub trait ISandboxTool: Send + Sync {
    async fn build_inner(&self) -> Result<SandboxInner, SandboxToolError>;

    async fn destroy_inner(&self, inner: &SandboxInner) -> Result<(), SandboxToolError>;

    async fn execute(
        &self,
        inner: &SandboxInner,
        options: &SandboxExecutionOptions,
    ) -> Result<SandboxExecutionResult, SandboxToolError>;

    async fn add_file_wd(
        &self,
        inner: &SandboxInner,
        file_name: &str,
        file_content: &str,
    ) -> Result<(), SandboxToolError>;

    async fn read_file_wd(
        &self,
        inner: &SandboxInner,
        file_name: &str,
    ) -> Result<String, SandboxToolError>;

    async fn remove_file_wd(
        &self,
        inner: &SandboxInner,
        file_name: &str,
    ) -> Result<(), SandboxToolError>;
}
