#!/bin/sh

release_folder="target/wasm32-unknown-unknown/release"

for wasm_file in "$release_folder"/*.wasm; do
    if [[ "$wasm_file" != *"optimized"* ]]; then
        stellar contract optimize --wasm "$wasm_file"
    fi
done
