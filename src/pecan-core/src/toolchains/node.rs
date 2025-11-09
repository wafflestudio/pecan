//! Node.js language toolchain
//! check toolchains/node/manifest.yaml for more details

use std::path::PathBuf;

use crate::toolchains::{DirMountOption, LanguageToolchain, RuntimeOption};
use crate::{define_language_toolchain_dir_default, define_mount_point_default};

pub const NODE_LANGUAGE: &str = "node";
pub const NODE_VERSION: &str = "20.18.0";
pub const NODE_BIN: &str = "bin/node";
define_language_toolchain_dir_default!(NODE_DIR, "node");
define_mount_point_default!(NODE_MOUNT_POINT, "node");

pub const NODE_SOURCE_FILE_NAME: &str = "main.js";

pub fn language_toolchain_node() -> LanguageToolchain {
    LanguageToolchain {
        name: NODE_LANGUAGE.to_string(),
        identifier: NODE_LANGUAGE.to_string(),
        version: NODE_VERSION.to_string(),
        compile_option: None,
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(NODE_MOUNT_POINT).join(NODE_BIN),
            dir_mount_options: Some(vec![DirMountOption {
                source_path: PathBuf::from(NODE_DIR),
                target_path: PathBuf::from(NODE_MOUNT_POINT),
            }]),
            env: None,
            args: vec![NODE_SOURCE_FILE_NAME.to_string()],
        },
    }
}
