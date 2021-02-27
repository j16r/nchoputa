#!/bin/bash
set -ex

rm -f viewer/viewer_bg.wasm viewer/viewer.js
cd viewer
cargo build
cd ..
cargo build
