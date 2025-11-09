use pecan_sandbox::sandbox::{
    CompileOptions, SandboxAdditionalDirectoryOptions, SandboxAdditionalFileOptions,
    SandboxExecutionOptions,
};

use crate::errors::CoreExecutionError;
use crate::toolchains::Language;
use crate::toolchains::c::{C_SOURCE_FILE_NAME, language_toolchain_c};
use crate::toolchains::cpp::{CPP_SOURCE_FILE_NAME, language_toolchain_cpp};
use crate::toolchains::go::{GO_SOURCE_FILE_NAME, language_toolchain_go};
use crate::toolchains::java::{JAVA_SOURCE_FILE_NAME, language_toolchain_java};
use crate::toolchains::kotlin::{KOTLIN_SOURCE_FILE_NAME, language_toolchain_kotlin};
use crate::toolchains::node::{NODE_SOURCE_FILE_NAME, language_toolchain_node};
use crate::toolchains::python::{PYTHON_SOURCE_FILE_NAME, language_toolchain_python};
use crate::toolchains::rust::{RUST_SOURCE_FILE_NAME, language_toolchain_rust};
use crate::toolchains::typescript::{TYPESCRIPT_SOURCE_FILE_NAME, language_toolchain_typescript};

pub fn build_sandbox_execution_option(
    language: Language,
    code: String,
    stdin: String,
    timeout: f64,
    memory_limit: f64,
) -> Result<SandboxExecutionOptions, CoreExecutionError> {
    let language_toolchain = match language {
        Language::C => language_toolchain_c(),
        Language::Cpp => language_toolchain_cpp(),
        Language::Go => language_toolchain_go(),
        Language::Java => language_toolchain_java(),
        Language::Kotlin => language_toolchain_kotlin(),
        Language::Node => language_toolchain_node(),
        Language::Python => language_toolchain_python(),
        Language::Rust => language_toolchain_rust(),
        Language::Typescript => language_toolchain_typescript(),
        Language::Unknown => {
            return Err(CoreExecutionError::NotSupportedLanguage(
                "Unknown language".to_string(),
            ));
        }
    };

    let additional_file_options = match language {
        Language::C => Some(vec![SandboxAdditionalFileOptions {
            file_name: C_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Cpp => Some(vec![SandboxAdditionalFileOptions {
            file_name: CPP_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Go => Some(vec![SandboxAdditionalFileOptions {
            file_name: GO_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Java => Some(vec![SandboxAdditionalFileOptions {
            file_name: JAVA_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Kotlin => Some(vec![SandboxAdditionalFileOptions {
            file_name: KOTLIN_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Node => Some(vec![SandboxAdditionalFileOptions {
            file_name: NODE_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Python => Some(vec![SandboxAdditionalFileOptions {
            file_name: PYTHON_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Rust => Some(vec![SandboxAdditionalFileOptions {
            file_name: RUST_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Typescript => Some(vec![SandboxAdditionalFileOptions {
            file_name: TYPESCRIPT_SOURCE_FILE_NAME.to_string(),
            file_content: code,
        }]),
        Language::Unknown => None,
    };

    let compile_options = match language_toolchain.compile_option {
        Some(compile_option) => Some(CompileOptions {
            compiler_path: compile_option.compiler_path,
            env: compile_option.env,
            args: compile_option.args,
        }),
        None => None,
    };

    let additional_directory_options = match language_toolchain.runtime_option.dir_mount_options {
        Some(dir_mount_options) => Some(
            dir_mount_options
                .into_iter()
                .map(|dir_mount_option| SandboxAdditionalDirectoryOptions {
                    directory_path: dir_mount_option.source_path,
                    mount_point: dir_mount_option.target_path,
                })
                .collect::<Vec<SandboxAdditionalDirectoryOptions>>(),
        ),
        None => None,
    };

    return Ok(SandboxExecutionOptions {
        additional_file_options: additional_file_options,
        compile_options: compile_options,
        additional_directory_options: additional_directory_options,
        binary_path: language_toolchain.runtime_option.binary_path,
        args: language_toolchain.runtime_option.args,
        stdin: stdin,
        time_limit: timeout,
        memory_limit: memory_limit,
    });
}
