use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use tokio_util::sync::CancellationToken;

use crate::manager::SandboxManager;
use crate::sandbox::{
    CompileOptions, SandboxAdditionalDirectoryOptions, SandboxAdditionalFileOptions,
    SandboxExecutionOptions,
};

pub mod errors;
pub mod manager;
pub mod sandbox;
pub mod tools;

pub async fn test_sandbox_manager() {
    let mut _manager = match SandboxManager::new(1).await {
        Ok(manager) => manager,
        Err(e) => {
            println!("Error creating manager: {:?}", e);
            return;
        }
    };

    let cancel_token = CancellationToken::new();
    let child_token = cancel_token.child_token();

    let manager_for_loop = Arc::clone(&_manager);
    let _loop_handle = tokio::spawn(async move {
        manager_for_loop.run_loop(child_token).await;
    });

    let ids = _manager.list_ids();
    println!("Sandbox IDs: {:?}", ids);

    let opt = Arc::new(SandboxExecutionOptions {
        compile_options: Some(CompileOptions {
            compiler_path: PathBuf::from("/opt/toolchains/kotlin/current/kotlinc/bin/kotlinc"),
            env: Some(HashMap::from([(
                "JAVA_HOME".to_string(),
                "/opt/toolchains/java/current".to_string(),
            )])),
            args: vec![
                "Main.kt".to_string(),
                "-include-runtime".to_string(),
                "-d".to_string(),
                "Main.jar".to_string(),
            ],
        }),
        additional_file_options: Some(vec![SandboxAdditionalFileOptions {
            file_name: "Main.kt".to_string(),
            file_content: r#"fun main() {
                        val N = readLine()!!.toInt()
                        var sum = 0
                        for (i in 0 until N) {
                            sum += readLine()!!.toInt()
                        }
                        println(sum)
                    }"#
            .to_string(),
        }]),
        additional_directory_options: Some(vec![SandboxAdditionalDirectoryOptions {
            directory_path: PathBuf::from("/opt/toolchains/java/current"),
            mount_point: PathBuf::from("/opt/java"),
        }]),
        binary_path: PathBuf::from("/opt/java/bin/java"),
        args: vec![
            "-Xmx128m".to_string(),
            "-Xms16m".to_string(),
            "-Xss512k".to_string(),
            "-XX:MaxMetaspaceSize=128m".to_string(),
            "-XX:ReservedCodeCacheSize=64m".to_string(),
            "-XX:MaxDirectMemorySize=32m".to_string(),
            "-XX:CompressedClassSpaceSize=64m".to_string(),
            "-jar".to_string(),
            "Main.jar".to_string(),
        ],
        stdin: "5\n1\n2\n3\n4\n1011".to_string(),
        time_limit: 10.0,
        memory_limit: 2048000.0,
    });

    //     let opt = Arc::new(SandboxExecutionOptions {
    //         compile_options: Some(CompileOptions {
    //             compiler_path: PathBuf::from("/opt/toolchains/rust/current/bin/rustc"),
    //             compiler_args: vec!["-o".to_string(), "main".to_string(), "main.rs".to_string()],
    //         }),
    //         additional_file_options: Some(
    //             vec![SandboxAdditionalFileOptions {
    //                 file_name: "main.rs".to_string(),
    //                 file_content: r#"
    //                     fn main() {
    // 	let mut input = String::new();
    // 	std::io::stdin().read_line(&mut input).unwrap();
    // 	let N: i32 = input.trim().parse().unwrap();
    // 	let mut inputs = Vec::new();
    // 	for _ in 0..N {
    // 		let mut input = String::new();
    // 		std::io::stdin().read_line(&mut input).unwrap();
    // 		inputs.push(input.trim().parse().unwrap());
    // 	}
    // 	let sum: i32 = inputs.iter().sum();
    // 	println!("{}", sum);
    // }
    //                 "#.to_string(),
    //             }],
    //         ),
    //         additional_directory_options: Some(
    //             vec![SandboxAdditionalDirectoryOptions {
    //                 directory_path: PathBuf::from("/opt/toolchains/rust/current"),
    //                 mount_point: PathBuf::from("/opt/rust"),
    //             }],
    //         ),
    //         binary_path: PathBuf::from("./main"),
    //         args: vec![],
    //         stdin: "4\n1\n2\n3\n10000".to_string(),
    //         time_limit: 4.0,
    //         memory_limit: 5000000.0,
    //     });

    let mut futs = FuturesUnordered::new();
    for _ in 0..1 {
        let mgr = _manager.clone();
        let opt_cloned = opt.clone();
        futs.push(async move { mgr.execute_via_manager(&opt_cloned).await });
    }

    while let Some(res) = futs.next().await {
        match res {
            Ok(exec_res) => println!("[DONE] {:?}", exec_res),
            Err(e) => eprintln!("[ERROR] {:?}", e),
        }
    }

    // wait until keyboard interrupt
    tokio::signal::ctrl_c().await.unwrap();

    println!("Teardown...");
    _manager.teardown().await.unwrap();
    cancel_token.cancel();
    let _ = _loop_handle.await;
    println!("Teardown done");
}
