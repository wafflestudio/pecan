use std::collections::HashMap;
use std::path::PathBuf;

// Language toolchains
mod c;
mod cpp;
mod go;
mod java;
mod kotlin;
mod node;
mod python;
mod rust;
mod typescript;

pub mod sandbox_options;

#[derive(Debug, Clone)]
pub enum Language {
    C,
    Cpp,
    Go,
    Java,
    Kotlin,
    Node,
    Python,
    Rust,
    Typescript,
    Unknown,
}

impl From<&str> for Language {
    fn from(value: &str) -> Self {
        match value {
            "c" => Language::C,
            "cpp" => Language::Cpp,
            "go" => Language::Go,
            "java" => Language::Java,
            "kotlin" => Language::Kotlin,
            "node" => Language::Node,
            "python" => Language::Python,
            "rust" => Language::Rust,
            "typescript" => Language::Typescript,
            _ => Language::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirMountOption {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CompileOption {
    pub compiler_path: PathBuf,
    pub env: Option<HashMap<String, String>>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeOption {
    pub binary_path: PathBuf,
    pub dir_mount_options: Option<Vec<DirMountOption>>,
    pub env: Option<HashMap<String, String>>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LanguageToolchain {
    pub name: String,
    pub identifier: String,
    pub version: String,
    pub compile_option: Option<CompileOption>,
    pub runtime_option: RuntimeOption,
}

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn language_from_str_maps_known_values() {
        assert!(matches!(Language::from("c"), Language::C));
        assert!(matches!(Language::from("cpp"), Language::Cpp));
        assert!(matches!(Language::from("go"), Language::Go));
        assert!(matches!(Language::from("java"), Language::Java));
        assert!(matches!(Language::from("kotlin"), Language::Kotlin));
        assert!(matches!(Language::from("node"), Language::Node));
        assert!(matches!(Language::from("python"), Language::Python));
        assert!(matches!(Language::from("rust"), Language::Rust));
        assert!(matches!(Language::from("typescript"), Language::Typescript));
    }

    #[test]
    fn language_from_str_defaults_to_unknown() {
        assert!(matches!(Language::from("elixir"), Language::Unknown));
        assert!(matches!(Language::from(""), Language::Unknown));
    }
}

#[macro_export]
macro_rules! define_language_toolchain_dir_default {
    ($name:ident, $language:expr) => {
        pub const $name: &str = concat!("/opt/toolchains/", $language, "/current");
    };
}

#[macro_export]
macro_rules! define_mount_point_default {
    ($name:ident, $language:expr) => {
        pub const $name: &str = concat!("/opt/", $language);
    };
}
