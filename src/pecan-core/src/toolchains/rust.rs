//! Rust language toolchain
//! check toolchains/rust/manifest.yaml for more details

use std::path::PathBuf;

use crate::define_language_toolchain_dir_default;
use crate::toolchains::{CompileOption, LanguageToolchain, RuntimeOption};

pub const RUST_LANGUAGE: &str = "rust";
pub const RUST_VERSION: &str = "1.81.0";
pub const RUSTC_BIN: &str = "bin/rustc";
define_language_toolchain_dir_default!(RUST_DIR, "rust");

pub const RUST_SOURCE_FILE_NAME: &str = "main.rs";
pub const RUST_BINARY_FILE_NAME: &str = "main";

pub fn language_toolchain_rust() -> LanguageToolchain {
    LanguageToolchain {
        name: RUST_LANGUAGE.to_string(),
        identifier: RUST_LANGUAGE.to_string(),
        version: RUST_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(RUST_DIR).join(RUSTC_BIN),
            env: None,
            args: vec![
                "-o".to_string(),
                RUST_BINARY_FILE_NAME.to_string(),
                RUST_SOURCE_FILE_NAME.to_string(),
            ],
        }),
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(RUST_BINARY_FILE_NAME),
            dir_mount_options: None,
            env: None,
            args: vec![],
        },
    }
}
