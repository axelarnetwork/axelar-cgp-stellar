#!/bin/sh

release_folder="target/wasm32-unknown-unknown/release"
prefix="stellar_"

if ! command -v stellar >/dev/null 2>&1; then
    echo "Error: 'stellar' command not found" >&2
    exit 1
fi

if [ ! -d "$release_folder" ]; then
    echo "Error: Release folder not found: $release_folder" >&2
    exit 1
fi

# Optimize WASM files
for wasm_file in "$release_folder"/*.wasm; do
    if [[ "$wasm_file" != *"optimized"* ]]; then
        stellar contract optimize --wasm "$wasm_file"
    fi
done

# Check and rename if the file starts with prefix and contains "optimized" in the middle
for wasm_file in "$release_folder"/*.wasm; do
    base_name=$(basename "$wasm_file")
    if [[ "$base_name" == ${prefix}* && "$base_name" == *optimized* ]]; then
        new_base_name="${base_name#${prefix}}"
        mv "$wasm_file" "$release_folder/$new_base_name"
    fi
done
