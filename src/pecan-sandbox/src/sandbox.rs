//! This module contains the core types and functions for the sandbox.
//!
//! The sandbox is a abstraction layer on top of the tool specified in the features.
//! It is used to execute programs in a safe and controlled environment.
//!
//! The sandbox is implemented using the tool specified in the features.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};

use uuid::Uuid;

use crate::tools::SandboxInner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxStatus {
    Idle,
    Running,
    Error,
}

const STATUS_IDLE: u8 = 0;
const STATUS_RUNNING: u8 = 1;
const STATUS_ERROR: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SandboxExecutionStatus {
    Success,
    CompileError,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
}

/// The sandbox is identified by a single UUID.
pub struct Sandbox {
    pub id: Uuid,
    status: AtomicU8,
    pub inner: SandboxInner,
}

impl Sandbox {
    pub fn new(inner: SandboxInner) -> Self {
        Self {
            id: Uuid::new_v4(),
            status: AtomicU8::new(STATUS_IDLE),
            inner,
        }
    }

    pub fn status(&self) -> SandboxStatus {
        match self.status.load(Ordering::Acquire) {
            STATUS_IDLE => SandboxStatus::Idle,
            STATUS_RUNNING => SandboxStatus::Running,
            STATUS_ERROR => SandboxStatus::Error,
            _ => SandboxStatus::Error,
        }
    }

    pub fn set_idle(&self) {
        self.status.store(STATUS_IDLE, Ordering::Release);
    }

    pub fn set_running(&self) {
        self.status.store(STATUS_RUNNING, Ordering::Release);
    }

    pub fn set_error(&self) {
        self.status.store(STATUS_ERROR, Ordering::Release);
    }
}

/// Files to be added to the sandbox working directory before the execution.
#[derive(Debug, Clone)]
pub struct SandboxAdditionalFileOptions {
    pub file_name: String,
    pub file_content: String,
}

/// Options for compiling the program before the execution.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// path to the compiler executable
    pub compiler_path: PathBuf,
    /// environment variables to set before compiling
    pub env: Option<HashMap<String, String>>,
    /// arguments to pass to the compiler
    pub args: Vec<String>,
}

/// Options for mounting additional directories
#[derive(Debug, Clone)]
pub struct SandboxAdditionalDirectoryOptions {
    pub directory_path: PathBuf,
    pub mount_point: PathBuf,
}

/// Basic execution options for the sandbox.
#[derive(Debug, Clone)]
pub struct SandboxExecutionOptions {
    pub additional_file_options: Option<Vec<SandboxAdditionalFileOptions>>,
    pub compile_options: Option<CompileOptions>,
    pub additional_directory_options: Option<Vec<SandboxAdditionalDirectoryOptions>>,
    /// path to the binary executable
    pub binary_path: PathBuf,
    /// arguments to pass to the binary
    pub args: Vec<String>,
    /// standard input to the binary
    pub stdin: String,
    /// time limit in seconds
    pub time_limit: f64,
    /// memory limit in kilobytes
    pub memory_limit: f64,
}

/// Result of the sandbox execution.
#[derive(Debug, Clone)]
pub struct SandboxExecutionResult {
    pub status: SandboxExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub time: f64,
    pub memory: f64,
}
