//! Python language toolchain
//! check toolchains/python/manifest.yaml for more details

use std::path::PathBuf;

use crate::toolchains::{DirMountOption, LanguageToolchain, RuntimeOption};
use crate::{define_language_toolchain_dir_default, define_mount_point_default};

pub const PYTHON_LANGUAGE: &str = "python";
pub const PYTHON_VERSION: &str = "3.12.7";
pub const PYTHON_BIN: &str = "bin/python3";
define_language_toolchain_dir_default!(PYTHON_DIR, "python");
define_mount_point_default!(PYTHON_MOUNT_POINT, "python");

pub const PYTHON_SOURCE_FILE_NAME: &str = "main.py";

pub fn language_toolchain_python() -> LanguageToolchain {
    LanguageToolchain {
        name: PYTHON_LANGUAGE.to_string(),
        identifier: PYTHON_LANGUAGE.to_string(),
        version: PYTHON_VERSION.to_string(),
        compile_option: None,
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(PYTHON_MOUNT_POINT).join(PYTHON_BIN),
            dir_mount_options: Some(vec![DirMountOption {
                source_path: PathBuf::from(PYTHON_DIR),
                target_path: PathBuf::from(PYTHON_MOUNT_POINT),
            }]),
            env: None,
            args: vec![PYTHON_SOURCE_FILE_NAME.to_string()],
        },
    }
}
