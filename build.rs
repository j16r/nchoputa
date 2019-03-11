use std::process::Command;

fn main() {
    let out_dir = "static";

    Command::new("wasm-bindgen")
        .args(&["./viewer/target/wasm32-unknown-unknown/debug/viewer.wasm",
              "--out-dir", out_dir,
              "--no-typescript",
              "--browser",
              "--no-modules"])
        .args(&["--keep-debug", "--debug", "--no-demangle"])
        .status().unwrap();

    Command::new("wasm-gc")
        .args(&["./static/viewer_bg.wasm"])
        .status().unwrap();
}
