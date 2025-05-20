#!/bin/bash
set -euo pipefail

# Function to extract package name from Cargo.toml
get_package_name() {
  if [ -f "$1/Cargo.toml" ]; then
    grep -E "^name\s*=" "$1/Cargo.toml" | head -1 | cut -d'"' -f2
  fi
}

# Create a temporary file to store seen package names
SEEN_PACKAGES=$(mktemp)

# Function to ensure unique package name
ensure_unique_name() {
  local pkg_path=$1
  local orig_name=$(get_package_name "$pkg_path")
  local new_name=$orig_name
  local counter=1
  
  while grep -q "^$new_name$" "$SEEN_PACKAGES" 2>/dev/null; do
    new_name="${orig_name}-${counter}"
    counter=$((counter + 1))
  done
  
  if [ "$new_name" != "$orig_name" ]; then
    echo "Renaming package in $pkg_path from $orig_name to $new_name"
    sed -i.bak "s/^name = \"$orig_name\"/name = \"$new_name\"/" "$pkg_path/Cargo.toml"
  fi
  
  echo "$new_name" >> "$SEEN_PACKAGES"
}

# Get initial package names for logging
ORBIT_NAME=$(get_package_name "orbit")
ORBITON_NAME=$(get_package_name "orbiton")
ANALYZER_NAME=$(get_package_name "orbit-analyzer")

echo "Initial package names:"
echo "orbit: $ORBIT_NAME"
echo "orbiton: $ORBITON_NAME"
echo "orbit-analyzer: $ANALYZER_NAME"

# Ensure unique names for all packages
ensure_unique_name "orbit"
ensure_unique_name "orbiton"
ensure_unique_name "orbit-analyzer"

echo "Final package names:"
echo "orbit: $(get_package_name "orbit")"
echo "orbiton: $(get_package_name "orbiton")"
echo "orbit-analyzer: $(get_package_name "orbit-analyzer")"

rm -f "$SEEN_PACKAGES"

# Create workspace Cargo.toml
cat > Cargo.toml << EOF
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

# Fix module ambiguity if needed
if [ -f "orbit/src/parser.rs" ] && [ -f "orbit/src/parser/mod.rs" ]; then
  echo "Found both parser.rs and parser/mod.rs. Renaming parser.rs to parser_legacy.rs..."
  mv orbit/src/parser.rs orbit/src/parser_legacy.rs
fi
