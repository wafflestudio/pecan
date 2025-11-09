//! Go language toolchain
//! check toolchains/go/manifest.yaml for more details

use std::path::PathBuf;

use crate::define_language_toolchain_dir_default;
use crate::toolchains::{CompileOption, LanguageToolchain, RuntimeOption};

pub const GO_LANGUAGE: &str = "go";
pub const GO_VERSION: &str = "1.23.3";
pub const GO_BIN: &str = "bin/go";
define_language_toolchain_dir_default!(GO_DIR, "go");

pub const GO_SOURCE_FILE_NAME: &str = "main.go";
pub const GO_BINARY_FILE_NAME: &str = "main";

pub fn language_toolchain_go() -> LanguageToolchain {
    LanguageToolchain {
        name: GO_LANGUAGE.to_string(),
        identifier: GO_LANGUAGE.to_string(),
        version: GO_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(GO_DIR).join(GO_BIN),
            env: None,
            args: vec![
                "build".to_string(),
                "-o".to_string(),
                GO_BINARY_FILE_NAME.to_string(),
                GO_SOURCE_FILE_NAME.to_string(),
            ],
        }),
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(GO_BINARY_FILE_NAME),
            dir_mount_options: None,
            env: None,
            args: vec![],
        },
    }
}
