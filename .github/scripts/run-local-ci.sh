#!/bin/bash

# Run all CI checks locally for orbitrs
# Usage: ./run-local-ci.sh

set -e

echo "Running local CI checks for orbitrs..."

# Format code
echo "Checking code formatting..."
./verify-formatting.sh

# Setup WASM target
echo "Setting up WASM target..."
rustup target add wasm32-unknown-unknown

# Run clippy
echo "Running clippy..."
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "Running tests..."
RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo test --all-features

# Build in release mode
echo "Building in release mode..."
cargo build --release

# Build WASM target
echo "Building WASM target..."
cargo build --target wasm32-unknown-unknown --release

echo "Local CI checks completed successfully!"
