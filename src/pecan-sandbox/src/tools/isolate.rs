//! This module contains the implementation of the Isolate tool.
//!
//! check [Isolate](https://github.com/ioi/isolate) for more details.

use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};

use tokio::fs::{read, remove_file, write};
use tokio::process::Command;

use crate::sandbox::{SandboxExecutionOptions, SandboxExecutionResult, SandboxExecutionStatus};
use crate::tools::common::ISandboxTool;
use crate::tools::errors::SandboxToolError;

const PROGRAM_NAME: &'static str = "isolate";

pub struct SandboxToolIsolate {
    box_id_counter: AtomicI32,
    box_id_pool: Mutex<Vec<i32>>,
}

impl SandboxToolIsolate {
    pub fn new() -> Self {
        Self {
            box_id_counter: AtomicI32::new(0),
            box_id_pool: Mutex::new(Vec::new()),
        }
    }

    pub fn claim_box_id(&self) -> Result<i32, SandboxToolError> {
        let mut box_id_pool = self.box_id_pool.lock().map_err(|e| {
            SandboxToolError::UnknownError(format!("Failed to lock box_id_pool: {}", e))
        })?;

        match box_id_pool.pop() {
            Some(box_id) => Ok(box_id),
            None => {
                let box_id = self.box_id_counter.fetch_add(1, Ordering::Acquire);
                Ok(box_id)
            }
        }
    }

    pub fn release_box_id(&self, box_id: i32) {
        if let Ok(mut box_id_pool) = self.box_id_pool.lock() {
            box_id_pool.push(box_id);
        }
    }

    pub fn get_box_id_pool_len(&self) -> usize {
        self.box_id_pool.lock().map(|pool| pool.len()).unwrap_or(0)
    }

    pub async fn create_isolate_box(&self, box_id: i32) -> Result<IsolateInner, SandboxToolError> {
        let mut base_cmd = Command::new(PROGRAM_NAME);
        if cfg!(feature = "isolate-cg") {
            base_cmd.arg("--cg");
        }

        let res = match base_cmd
            .arg(format!("--box-id={}", box_id))
            .arg("--init")
            .output()
            .await
        {
            Ok(output) => output,
            Err(e) => return Err(SandboxToolError::UnknownError(e.to_string())),
        };

        let output_str = std::str::from_utf8(&res.stdout).map_err(|e| {
            SandboxToolError::UnknownError(format!("Invalid UTF-8 in isolate output: {}", e))
        })?;
        let base_path = format!("{}/box", output_str.trim());

        Ok(IsolateInner::new(box_id, PathBuf::from(base_path)))
    }

    pub async fn destroy_isolate_box(&self, box_id: i32) -> Result<(), SandboxToolError> {
        let mut base_cmd = Command::new(PROGRAM_NAME);
        if cfg!(feature = "isolate-cg") {
            base_cmd.arg("--cg");
        }

        match base_cmd
            .arg(format!("--box-id={}", box_id))
            .arg("--cleanup")
            .output()
            .await
        {
            Ok(output) => output,
            Err(e) => return Err(SandboxToolError::UnknownError(e.to_string())),
        };

        Ok(())
    }
}

impl ISandboxTool for SandboxToolIsolate {
    async fn build_inner(&self) -> Result<IsolateInner, SandboxToolError> {
        let box_id = self
            .claim_box_id()
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;
        let inner = self
            .create_isolate_box(box_id)
            .await
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;

        Ok(inner)
    }

    async fn destroy_inner(&self, inner: &IsolateInner) -> Result<(), SandboxToolError> {
        self.destroy_isolate_box(inner.get_box_id())
            .await
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;
        self.release_box_id(inner.get_box_id());
        Ok(())
    }

    async fn execute(
        &self,
        inner: &IsolateInner,
        options: &SandboxExecutionOptions,
    ) -> Result<SandboxExecutionResult, SandboxToolError> {
        let stdin_file_name = "stdin.txt";

        let meta_file_name = "meta.txt";
        let meta_file_path = inner.path.join(meta_file_name);

        self.add_file_wd(inner, stdin_file_name, options.stdin.as_str())
            .await?;

        let mut base_cmd = Command::new(PROGRAM_NAME);
        if cfg!(feature = "isolate-cg") {
            base_cmd
                .arg("--cg")
                .arg(format!("--cg-mem={}", options.memory_limit));
        } else {
            base_cmd.arg(format!("--mem={}", options.memory_limit));
        }

        if let Some(additional_directory_options) = &options.additional_directory_options {
            for additional_directory_option in additional_directory_options {
                base_cmd.arg(format!(
                    "--dir={}={}",
                    additional_directory_option.mount_point.to_string_lossy(),
                    additional_directory_option.directory_path.to_string_lossy()
                ));
            }
        }

        base_cmd
            .arg(format!("--box-id={}", inner.get_box_id()))
            .arg(format!("--processes={}", 128))
            .arg(format!("--time={}", options.time_limit))
            .arg(format!("--wall-time={}", 100))
            .arg(format!("--stdin={}", stdin_file_name))
            .arg(format!("--meta={}", meta_file_path.to_string_lossy()))
            .arg("--run")
            .arg("--")
            .arg(options.binary_path.to_str().ok_or_else(|| {
                SandboxToolError::UnknownError("Invalid binary path encoding".to_string())
            })?)
            .args(
                options
                    .args
                    .iter()
                    .map(|arg| arg.as_str())
                    .collect::<Vec<&str>>(),
            );

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

        let meta_content = self.read_file_wd(inner, meta_file_name).await?;

        let meta_time: f64 = parse_meta_file(&meta_content, "time", 0.0);
        let meta_cg_mem: u32 = parse_meta_file(&meta_content, "cg-mem", 0);
        let meta_mem: u32 = parse_meta_file(&meta_content, "max-rss", 0);
        let meta_cg_oom_killed: u32 = parse_meta_file(&meta_content, "cg-oom-killed", 0);
        let meta_status: String = parse_meta_file(&meta_content, "status", "OK".to_string());

        self.remove_file_wd(inner, meta_file_name).await?;
        self.remove_file_wd(inner, stdin_file_name).await?;

        let mut is_internal_error = false;

        let status = if meta_cg_oom_killed == 1 {
            SandboxExecutionStatus::MemoryLimitExceeded
        } else if meta_status == "RE" || meta_status == "SG" {
            SandboxExecutionStatus::RuntimeError
        } else if meta_status == "TO" {
            SandboxExecutionStatus::TimeLimitExceeded
        } else if meta_status == "XX" {
            is_internal_error = true;
            SandboxExecutionStatus::RuntimeError
        } else if !res.status.success() {
            SandboxExecutionStatus::RuntimeError
        } else {
            SandboxExecutionStatus::Success
        };

        if is_internal_error {
            return Err(SandboxToolError::UnknownError("Internal error".to_string()));
        }

        let stdout = String::from_utf8_lossy(&res.stdout).to_string();
        let stderr = String::from_utf8_lossy(&res.stderr).to_string();

        Ok(SandboxExecutionResult {
            status,
            stdout,
            stderr,
            time: meta_time,
            memory: match cfg!(feature = "isolate-cg") {
                true => meta_cg_mem as f64,
                false => meta_mem as f64,
            },
        })
    }

    async fn add_file_wd(
        &self,
        inner: &IsolateInner,
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
        inner: &IsolateInner,
        file_name: &str,
    ) -> Result<String, SandboxToolError> {
        let path = inner.path.join(file_name);
        let content = read(path)
            .await
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;
        Ok(String::from_utf8_lossy(&content).to_string())
    }

    async fn remove_file_wd(
        &self,
        inner: &IsolateInner,
        file_name: &str,
    ) -> Result<(), SandboxToolError> {
        let path = inner.path.join(file_name);
        remove_file(path)
            .await
            .map_err(|e| SandboxToolError::UnknownError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct IsolateInner {
    box_id: i32,
    path: PathBuf,
}

impl IsolateInner {
    pub fn new(box_id: i32, path: PathBuf) -> Self {
        Self { box_id, path }
    }

    pub fn get_box_id(&self) -> i32 {
        self.box_id
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

fn parse_meta_file<S: FromStr>(content: &str, key: &str, default: S) -> S {
    content
        .lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| line.split(':').nth(1))
        .map(|value| value.trim().parse::<S>().ok())
        .flatten()
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::SandboxToolIsolate;
    use crate::tools::common::ISandboxTool;
    use std::collections::HashSet;
    use tokio::process::Command;

    #[tokio::test]
    async fn isolate_build_and_destroy_releases_box_id() {
        let isolate_ready = Command::new("isolate")
            .arg("--version")
            .output()
            .await
            .is_ok();
        if !isolate_ready {
            eprintln!("Skipping isolate test: isolate binary not available");
            return;
        }

        let tool = SandboxToolIsolate::new();
        let inner = tool.build_inner().await.expect("build inner");
        let initial_pool_len = tool.get_box_id_pool_len();

        tool.destroy_inner(&inner).await.expect("destroy inner");

        assert_eq!(tool.get_box_id_pool_len(), initial_pool_len + 1);
    }

    #[tokio::test]
    async fn isolate_multiple_boxes_create_and_destroy() {
        let isolate_ready = Command::new("isolate")
            .arg("--version")
            .output()
            .await
            .is_ok();
        if !isolate_ready {
            eprintln!("Skipping isolate test: isolate binary not available");
            return;
        }

        let tool = SandboxToolIsolate::new();
        let initial_pool_len = tool.get_box_id_pool_len();

        let mut inners = Vec::new();
        for _ in 0..3 {
            inners.push(tool.build_inner().await.expect("build inner"));
        }

        let ids: HashSet<i32> = inners.iter().map(|inner| inner.get_box_id()).collect();
        assert_eq!(ids.len(), 3, "box ids should be unique");

        for inner in inners {
            tool.destroy_inner(&inner).await.expect("destroy inner");
        }

        assert_eq!(tool.get_box_id_pool_len(), initial_pool_len + 3);

        let reused = tool.build_inner().await.expect("build inner");
        assert!(
            ids.contains(&reused.get_box_id()),
            "expected a reused box id from the pool"
        );
        tool.destroy_inner(&reused).await.expect("destroy inner");
    }
}
