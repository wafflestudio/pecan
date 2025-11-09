//! Kotlin language toolchain
//! check toolchains/kotlin/manifest.yaml for more details

use std::collections::HashMap;
use std::path::PathBuf;

use crate::define_language_toolchain_dir_default;
use crate::toolchains::java::{JAVA_BIN, JAVA_DIR, JAVA_MOUNT_POINT};
use crate::toolchains::{CompileOption, DirMountOption, LanguageToolchain, RuntimeOption};

pub const KOTLIN_LANGUAGE: &str = "kotlin";
pub const KOTLIN_VERSION: &str = "2.0.21";
pub const KOTLINC_BIN: &str = "kotlinc/bin/kotlinc";
define_language_toolchain_dir_default!(KOTLIN_DIR, "kotlin");

pub const KOTLIN_SOURCE_FILE_NAME: &str = "Main.kt";
pub const KOTLIN_JAR_FILE_NAME: &str = "Main.jar";

pub fn language_toolchain_kotlin() -> LanguageToolchain {
    LanguageToolchain {
        name: KOTLIN_LANGUAGE.to_string(),
        identifier: KOTLIN_LANGUAGE.to_string(),
        version: KOTLIN_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(KOTLIN_DIR).join(KOTLINC_BIN),
            env: Some(HashMap::from([(
                "JAVA_HOME".to_string(),
                JAVA_DIR.to_string(),
            )])),
            args: vec![
                KOTLIN_SOURCE_FILE_NAME.to_string(),
                "-include-runtime".to_string(),
                "-d".to_string(),
                KOTLIN_JAR_FILE_NAME.to_string(),
            ],
        }),
        runtime_option: RuntimeOption {
            binary_path: PathBuf::from(JAVA_MOUNT_POINT).join(JAVA_BIN),
            dir_mount_options: Some(vec![DirMountOption {
                source_path: PathBuf::from(JAVA_DIR),
                target_path: PathBuf::from(JAVA_MOUNT_POINT),
            }]),
            env: None,
            args: vec![
                "-Xmx128m".to_string(),
                "-Xms16m".to_string(),
                "-Xss512k".to_string(),
                "-XX:MaxMetaspaceSize=128m".to_string(),
                "-XX:ReservedCodeCacheSize=64m".to_string(),
                "-XX:MaxDirectMemorySize=32m".to_string(),
                "-XX:CompressedClassSpaceSize=64m".to_string(),
                "-jar".to_string(),
                KOTLIN_JAR_FILE_NAME.to_string(),
            ],
        },
    }
}
