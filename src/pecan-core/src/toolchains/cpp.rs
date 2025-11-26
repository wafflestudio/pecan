//! C++ language toolchain
//! check toolchains/cpp/manifest.yaml for more details
//!
//! Currently, C, C++ language toolchain is not valid due to complex system library dependency for libc.
//! Just use gcc, g++ from apt install.

use std::path::PathBuf;

use crate::toolchains::{CompileOption, LanguageToolchain, RuntimeOption};

pub const CPP_LANGUAGE: &str = "cpp";
pub const CPP_VERSION: &str = "11";
pub const GXX_BIN: &str = "/usr/bin/g++";

pub const CPP_SOURCE_FILE_NAME: &str = "main.cpp";
pub const CPP_BINARY_FILE_NAME: &str = "main";

pub fn language_toolchain_cpp() -> LanguageToolchain {
    LanguageToolchain {
        name: CPP_LANGUAGE.to_string(),
        identifier: CPP_LANGUAGE.to_string(),
        version: CPP_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(GXX_BIN),
            env: None,
            args: vec![
                "-o".to_string(),
                CPP_BINARY_FILE_NAME.to_string(),
                CPP_SOURCE_FILE_NAME.to_string(),
            ],
        }),
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(CPP_BINARY_FILE_NAME),
            dir_mount_options: None,
            env: None,
            args: vec![],
        },
    }
}
