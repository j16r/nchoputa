use std::process::Command;
use std::env;

fn main() {
    let out_dir = "static";
    let profile = env::var("PROFILE").unwrap();

    let source_wasm = format!("target/wasm32-unknown-unknown/{}/viewer.wasm", profile).to_string();

    println!("cargo:rerun-if-changed={}", source_wasm);
    println!("cargo:rerun-if-changed=viewer/src/*.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let core_bindgen_args = [
        &source_wasm,
        "--out-dir", out_dir,
        "--no-typescript",
        "--browser",
        "--no-modules"];
    if profile == "debug" {
        Command::new("wasm-bindgen")
            .args(&core_bindgen_args)
            .args(&["--keep-debug", "--debug", "--no-demangle"])
            .status()
            .unwrap();
    } else {
        Command::new("wasm-bindgen")
            .args(&core_bindgen_args)
            .status()
            .unwrap();
    }

    Command::new("wasm-gc")
        .args(&["static/viewer_bg.wasm"])
        .status()
        .unwrap();
}
