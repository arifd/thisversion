#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Print each command before executing it.
set -x

# Check that we can compile with the reported MSRV for all publishable crates:
cargo hack check --rust-version --workspace --ignore-private --locked

# Lint a new release for SemVer breakage:
# cargo semver-checks

# Build and verify both distributable crates against the local derive source.
# This does not publish anything.
cargo package -p thisversion-derive
cargo package \
    --config 'patch.crates-io.thisversion-derive.path="crates/thisversion-derive"' \
    -p thisversion
