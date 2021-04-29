use std::{env, path::Path, process::Command};

fn main() {
    let out_dir = "static";
    let profile = env::var("PROFILE").unwrap();

    let source_wasm = Path::new("viewer/target/wasm32-unknown-unknown/")
        .join(&profile)
        .join("viewer.wasm");

    println!("cargo:rerun-if-changed={}", source_wasm.to_str().unwrap());
    println!("cargo:rerun-if-changed=viewer/src/*.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let mut bindgen_args = vec!(
        source_wasm.to_str().unwrap(),
        "--out-dir", out_dir,
        "--no-typescript",
        "--browser",
        "--no-modules",
        "--out-name", source_wasm.file_name().unwrap().to_str().unwrap());
    if profile == "debug" {
        bindgen_args.extend(&["--keep-debug", "--debug", "--no-demangle"]);
    }
    let status = Command::new("wasm-bindgen")
        .args(&bindgen_args)
        .status()
        .expect("error while running wasm-bindgen");
    assert!(status.success());
}
