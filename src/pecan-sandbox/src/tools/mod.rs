pub mod common;
pub mod errors;

#[cfg(feature = "isolate")]
pub mod isolate;

#[cfg(feature = "nsjail")]
pub mod nsjail;

#[cfg(feature = "isolate")]
pub type SandboxTool = isolate::SandboxToolIsolate;
#[cfg(feature = "isolate")]
pub type SandboxInner = isolate::IsolateInner;

#[cfg(feature = "nsjail")]
pub type SandboxTool = nsjail::SandboxToolNsjail;
#[cfg(feature = "nsjail")]
pub type SandboxInner = nsjail::NsjailInner;

#[cfg(not(any(feature = "isolate", feature = "nsjail")))]
compile_error!("Exactly one of 'isolate' or 'nsjail' features must be enabled");

#[cfg(feature = "isolate")]
pub fn build_tool() -> Result<SandboxTool, Box<dyn std::error::Error>> {
    Ok(isolate::SandboxToolIsolate::new())
}

#[cfg(feature = "nsjail")]
pub fn build_tool() -> Result<SandboxTool, Box<dyn std::error::Error>> {
    Ok(nsjail::SandboxToolNsjail::new())
}
