//! This module contains the implementation of the Nsjail tool.
//!
//! check [Nsjail](https://github.com/google/nsjail) for more details.

use std::collections::HashMap;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Mutex;

use tokio::fs::{read, remove_file, write};
use tokio::process::Command;
use uuid::Uuid;

use crate::sandbox::{SandboxExecutionOptions, SandboxExecutionResult, SandboxExecutionStatus};
use crate::tools::common::ISandboxTool;
use crate::tools::errors::SandboxToolError;

const PROGRAM_NAME: &str = "nsjail";
const NSJAIL_BASE_DIR: &str = "/var/pecan/box";

pub struct SandboxToolNsjail {
    box_id_pool: Mutex<HashMap<Uuid, NsjailInner>>,
}

impl SandboxToolNsjail {
    pub fn new() -> Self {
        Self {
            box_id_pool: Mutex::new(HashMap::new()),
        }
    }

    pub async fn create_nsjail_box(&self) -> Result<NsjailInner, SandboxToolError> {
        let new_id = Uuid::new_v4();
        let base_path = format!("{}/{}", NSJAIL_BASE_DIR, String::from(new_id));

        let mut box_id_pool = self.box_id_pool.lock().map_err(|e| {
            SandboxToolError::UnknownError(format!("Failed to lock box_id_pool: {}", e))
        })?;

        create_dir_all(&base_path)
            .map_err(|e| SandboxToolError::FileOperationFailed(e.to_string()))?;

        let inner = NsjailInner {
            box_id: new_id,
            path: PathBuf::from(base_path),
        };

        box_id_pool.insert(new_id, inner.clone());

        Ok(inner)
    }

    pub async fn destroy_nsjail_box(&self, box_id: Uuid) -> Result<(), SandboxToolError> {
        let mut box_id_pool = self.box_id_pool.lock().map_err(|e| {
            SandboxToolError::UnknownError(format!("Failed to lock box_id_pool: {}", e))
        })?;

        match box_id_pool.remove(&box_id) {
            Some(inner) => remove_dir_all(inner.path)
                .map_err(|e| SandboxToolError::FileOperationFailed(e.to_string())),
            None => Err(SandboxToolError::UnknownError(
                "Box id not found".to_string(),
            )),
        }
    }
}

impl ISandboxTool for SandboxToolNsjail {
    async fn build_inner(&self) -> Result<NsjailInner, SandboxToolError> {
        self.create_nsjail_box().await
    }

    async fn destroy_inner(&self, inner: &NsjailInner) -> Result<(), SandboxToolError> {
        self.destroy_nsjail_box(inner.get_box_id()).await
    }

    async fn execute(
        &self,
        inner: &NsjailInner,
        options: &SandboxExecutionOptions,
    ) -> Result<SandboxExecutionResult, SandboxToolError> {
        let stdin_file_name = "stdin.txt";

        self.add_file_wd(inner, stdin_file_name, options.stdin.as_str())
            .await?;

        let mut base_cmd = Command::new(PROGRAM_NAME);

        if let Some(additional_directory_options) = &options.additional_directory_options {
            for additional_directory_option in additional_directory_options {
                base_cmd.args([
                    "--bindmount",
                    &format!(
                        "{}:{}",
                        additional_directory_option.directory_path.to_string_lossy(),
                        additional_directory_option.mount_point.to_string_lossy()
                    ),
                ]);
            }
        }

        base_cmd
            .arg("--use_cgroupv2")
            .args(["--cgroup_mem_max", &options.memory_limit.to_string()])
            .args(["--cgroup_pids_max", &128.to_string()])
            .args(["--time_limit", &options.time_limit.to_string()])
            .args(["--chroot", "/"])
            .args(["--cwd", &inner.get_path().to_string_lossy()])
            .arg("--")
            .arg(options.binary_path.to_str().ok_or_else(|| {
                SandboxToolError::UnknownError("Invalid binary path encoding".to_string())
            })?)
            .args(&options.args);

        let base_cmd_child = base_cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;

        let res = match base_cmd_child.wait_with_output().await {
            Ok(output) => output,
            Err(e) => {
                return Err(SandboxToolError::UnknownError(e.to_string()));
            }
        };

        let stdout = String::from_utf8_lossy(&res.stdout).to_string();
        let stderr = String::from_utf8_lossy(&res.stderr).to_string();

        // parse stderr, stdout here and retrieve status
        let status = parse_nsjail_output(&stdout, &stderr);

        Ok(SandboxExecutionResult {
            status,
            stdout,
            stderr,
            // just return default value because nsjail does not output consumed time and memory
            time: 0.into(),
            memory: 0.into(),
        })
    }

    async fn add_file_wd(
        &self,
        inner: &NsjailInner,
        file_name: &str,
        file_content: &str,
    ) -> Result<(), SandboxToolError> {
        let path = inner.path.join(file_name);
        write(path, file_content)
            .await
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;
        Ok(())
    }

    async fn read_file_wd(
        &self,
        inner: &super::SandboxInner,
        file_name: &str,
    ) -> Result<String, SandboxToolError> {
        let path = inner.path.join(file_name);
        let content = read(path)
            .await
            .map_err(|e| SandboxToolError::FileOperationFailed(e.to_string()))?;

        Ok(String::from_utf8_lossy(&content).to_string())
    }

    async fn remove_file_wd(
        &self,
        inner: &NsjailInner,
        file_name: &str,
    ) -> Result<(), SandboxToolError> {
        let path = inner.path.join(file_name);
        remove_file(path)
            .await
            .map_err(|e| SandboxToolError::FileOperationFailed(e.to_string()))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct NsjailInner {
    box_id: Uuid,
    path: PathBuf,
}

impl NsjailInner {
    pub fn new(box_id: Uuid, path: PathBuf) -> Self {
        Self { box_id, path }
    }

    pub fn get_box_id(&self) -> Uuid {
        self.box_id
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }
}

fn parse_nsjail_output(stdout: &str, stderr: &str) -> SandboxExecutionStatus {
    if stdout != "" && stderr.contains("exited with status: 0") {
        return SandboxExecutionStatus::Success;
    }

    if stderr.contains("run time >= time limit") {
        return SandboxExecutionStatus::TimeLimitExceeded;
    }

    // reduce all the other errors into MLE error
    SandboxExecutionStatus::MemoryLimitExceeded
}
