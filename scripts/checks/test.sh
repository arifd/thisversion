#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Print each command before executing it.
set -x

# Run tests (including doctests):
cargo test --all-features

# Build the artifacts needed for mdbook code examples to compile, and keep these
# artifacts separate from cargo test's target/debug/deps, where rustdoc cannot
# choose between the multiple feature variants of each crate:
mdbook_target_dir="target/mdbook-test"
rm -rf "$mdbook_target_dir"
CARGO_TARGET_DIR="$mdbook_target_dir" cargo build -p thisversion --example serde --all-features

# Run mdBook code examples:
CARGO_MANIFEST_DIR="$PWD/crates/thisversion" \
    RUSTUP_TOOLCHAIN=stable \
    mdbook test docs --library-path "$mdbook_target_dir/debug/deps"

# Check our no-std guarantee:
cargo check -p thisversion --lib --target thumbv7em-none-eabi

# Run all examples:
find crates -type f -path '*/examples/*.rs' -print0 |
    while IFS= read -r -d '' example; do
        cargo run --all-features --example "$(basename "$example" .rs)"
    done
