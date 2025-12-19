#!/bin/bash

set -e
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Build
echo Building...
export EMCC_CFLAGS="-O2 -sALLOW_MEMORY_GROWTH -sASYNCIFY -sEXPORTED_RUNTIME_METHODS='[\"HEAPF32\"]'"
cargo build --target wasm32-unknown-emscripten

# Deploy
echo Deploying...
mkdir -p web
cp $SCRIPT_DIR/../target/wasm32-unknown-emscripten/debug/langjam-gamejam-engine.js $SCRIPT_DIR/web/
cp $SCRIPT_DIR/../target/wasm32-unknown-emscripten/debug/langjam_gamejam_engine.wasm $SCRIPT_DIR/web/
cp $SCRIPT_DIR/index.html $SCRIPT_DIR/web/

echo Done!
