#!/bin/bash

set -euo pipefail

# First, update orbiton's dependencies to use local paths
sed -i.bak 's#orbit = { git = "https://github.com/orbitrs/orbit.git" }#orbit = { path = "../orbit" }#' orbiton/Cargo.toml
sed -i.bak 's#orbit-analyzer = { git = "https://github.com/orbitrs/orbit-analyzer.git" }#orbit-analyzer = { path = "../orbit-analyzer" }#' orbiton/Cargo.toml

# Create the workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "orbit",
    "orbit-analyzer",
    "orbiton"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Orbit Team <orbit@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/orbitrs/orbit"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
log = "0.4"
tokio = { version = "1.34.0", features = ["full"] }
wasm-bindgen = "0.2.89"
anyhow = "1.0.75"
clap = { version = "4.4", features = ["derive"] }
futures = "0.3.28"
async-trait = "0.1.74"
regex = "1.10.2"
chrono = { version = "0.4.31", features = ["serde"] }
url = "2.5.0"
reqwest = { version = "0.11", features = ["json"] }
EOF

# Print debug information
echo "Workspace configuration:"
find . -name "Cargo.toml" -type f -exec echo "{}" \; -exec cat "{}" \;
