#!/bin/bash

# Verify and format Rust code
# Usage: ./verify-formatting.sh

set -e

# Check if rustfmt is installed
if ! command -v rustfmt &> /dev/null; then
    rustup component add rustfmt
fi

# Format all Rust files
echo "Formatting Rust files..."
cargo fmt --all -- --check

echo "All files are properly formatted!"
