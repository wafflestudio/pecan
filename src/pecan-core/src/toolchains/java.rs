//! Java language toolchain
//! check toolchains/java/manifest.yaml for more details

use std::path::PathBuf;

use crate::toolchains::{CompileOption, DirMountOption, LanguageToolchain, RuntimeOption};
use crate::{define_language_toolchain_dir_default, define_mount_point_default};

pub const JAVA_LANGUAGE: &str = "java";
pub const JAVA_VERSION: &str = "17";
pub const JAVA_BIN: &str = "bin/java";
pub const JAVAC_BIN: &str = "bin/javac";
define_language_toolchain_dir_default!(JAVA_DIR, "java");
define_mount_point_default!(JAVA_MOUNT_POINT, "java");

pub const JAVA_SOURCE_FILE_NAME: &str = "Main.java";
pub const JAVA_CLASS_NAME: &str = "Main";

pub fn language_toolchain_java() -> LanguageToolchain {
    LanguageToolchain {
        name: JAVA_LANGUAGE.to_string(),
        identifier: JAVA_LANGUAGE.to_string(),
        version: JAVA_VERSION.to_string(),
        compile_option: Some(CompileOption {
            compiler_path: PathBuf::from(JAVA_DIR).join(JAVAC_BIN),
            env: None,
            args: vec![JAVA_SOURCE_FILE_NAME.to_string()],
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
                JAVA_CLASS_NAME.to_string(),
            ],
        },
    }
}
