pub mod common;
pub mod errors;

#[cfg(all(feature = "isolate", feature = "nsjail"))]
compile_error!("Enable only one of 'isolate' or 'nsjail', not both");

#[cfg(not(any(feature = "isolate", feature = "nsjail")))]
compile_error!("Exactly one of 'isolate' or 'nsjail' features must be enabled");

#[cfg(sandbox_isolate)]
pub mod isolate;
#[cfg(sandbox_isolate)]
pub const SANDBOX_SOLUTION: &str = "isolate";

#[cfg(sandbox_nsjail)]
pub mod nsjail;
#[cfg(sandbox_nsjail)]
pub const SANDBOX_SOLUTION: &str = "nsjail";

#[cfg(sandbox_isolate)]
pub type SandboxTool = isolate::SandboxToolIsolate;
#[cfg(sandbox_isolate)]
pub type SandboxInner = isolate::IsolateInner;

#[cfg(sandbox_nsjail)]
pub type SandboxTool = nsjail::SandboxToolNsjail;
#[cfg(sandbox_nsjail)]
pub type SandboxInner = nsjail::NsjailInner;

#[cfg(sandbox_isolate)]
pub fn build_tool() -> Result<SandboxTool, Box<dyn std::error::Error>> {
    Ok(isolate::SandboxToolIsolate::new())
}

#[cfg(sandbox_nsjail)]
pub fn build_tool() -> Result<SandboxTool, Box<dyn std::error::Error>> {
    Ok(nsjail::SandboxToolNsjail::new())
}
