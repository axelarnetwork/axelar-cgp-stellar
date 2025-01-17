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
    case "$wasm_file" in
        *optimized*)
            # Skip already optimized files
            continue
            ;;
        *)
            stellar contract optimize --wasm "$wasm_file"
            ;;
    esac
done

# Rename files that start with the prefix and contain "optimized"
for wasm_file in "$release_folder"/*.wasm; do
    base_name=$(basename "$wasm_file")
    case "$base_name" in
        "${prefix}"*optimized*)
            # Remove the prefix from the filename
            new_base_name="${base_name#${prefix}}"
            mv "$wasm_file" "$release_folder/$new_base_name"
            ;;
    esac
done
