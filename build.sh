#!/bin/bash
set -ex

rm -f viewer/viewer.wasm viewer/viewer.js
cd viewer
cargo build
cd ..
cargo build
