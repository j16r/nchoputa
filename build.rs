use std::process::Command;

fn main() {
    let out_dir = "static";

    Command::new("wasm-bindgen")
        .args(&["./viewer/target/wasm32-unknown-unknown/debug/viewer.wasm",
              "--out-dir", out_dir,
              "--no-typescript",
              "--browser",
              "--no-modules"])
        .status().unwrap();
}
