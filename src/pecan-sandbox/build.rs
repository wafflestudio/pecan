fn main() {
    match std::env::var("SANDBOX_TYPE").as_deref() {
        Ok("isolate-cg") => {
            println!("cargo:rustc-cfg=feature=\"isolate\"");
            println!("cargo:rustc-cfg=feature=\"isolate-cg\"");
        }
        Ok("isolate") => {
            println!("cargo:rustc-cfg=feature=\"isolate\"");
        }
        Ok("nsjail") => {
            // println!("cargo:rustc-cfg=feature=\"nsjail\"");
            // TODO: implement nsjail feature
            panic!("nsjail feature is not implemented");
        }
        _ => {
            // default to isolate
            println!("cargo:rustc-cfg=feature=\"isolate\"");
        }
    }
}
