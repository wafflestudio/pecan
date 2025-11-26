//! C language toolchain
//! check toolchains/c/manifest.yaml for more details
//!
//! Currently, C, C++ language toolchain is not valid due to complex system library dependency for libc.
//! Just use gcc, g++ from apt install.

use std::path::PathBuf;

use crate::toolchains::{CompileOption, LanguageToolchain, RuntimeOption};

pub const C_LANGUAGE: &str = "c";
pub const C_VERSION: &str = "11";
pub const GCC_BIN: &str = "/usr/bin/gcc";

pub const C_SOURCE_FILE_NAME: &str = "main.c";
pub const C_BINARY_FILE_NAME: &str = "main";

pub fn language_toolchain_c() -> LanguageToolchain {
    LanguageToolchain {
        name: C_LANGUAGE.to_string(),
        identifier: C_LANGUAGE.to_string(),
        version: C_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(GCC_BIN),
            env: None,
            args: vec![
                "-o".to_string(),
                C_BINARY_FILE_NAME.to_string(),
                C_SOURCE_FILE_NAME.to_string(),
            ],
        }),
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(C_BINARY_FILE_NAME),
            dir_mount_options: None,
            env: None,
            args: vec![],
        },
    }
}
