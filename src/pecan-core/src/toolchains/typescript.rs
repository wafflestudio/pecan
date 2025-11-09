//! TypeScript language toolchain
//! check toolchains/typescript/manifest.yaml for more details

use std::path::PathBuf;

use crate::define_language_toolchain_dir_default;
use crate::toolchains::node::{NODE_BIN, NODE_DIR, NODE_MOUNT_POINT};
use crate::toolchains::{CompileOption, DirMountOption, LanguageToolchain, RuntimeOption};

pub const TYPESCRIPT_LANGUAGE: &str = "typescript";
pub const TYPESCRIPT_VERSION: &str = "5.7.3";
pub const TYPESCRIPT_BIN: &str = "bin/tsc";
define_language_toolchain_dir_default!(TYPESCRIPT_DIR, "typescript");

pub const TYPESCRIPT_SOURCE_FILE_NAME: &str = "main.ts";
pub const TYPESCRIPT_JS_FILE_NAME: &str = "main.js";

pub fn language_toolchain_typescript() -> LanguageToolchain {
    LanguageToolchain {
        name: TYPESCRIPT_LANGUAGE.to_string(),
        identifier: TYPESCRIPT_LANGUAGE.to_string(),
        version: TYPESCRIPT_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(TYPESCRIPT_DIR).join(TYPESCRIPT_BIN),
            env: None,
            args: vec![TYPESCRIPT_SOURCE_FILE_NAME.to_string()],
        }),
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(NODE_MOUNT_POINT).join(NODE_BIN),
            dir_mount_options: Some(vec![DirMountOption {
                source_path: PathBuf::from(NODE_DIR),
                target_path: PathBuf::from(NODE_MOUNT_POINT),
            }]),
            env: None,
            args: vec![TYPESCRIPT_JS_FILE_NAME.to_string()],
        },
    }
}
