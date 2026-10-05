#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Print each command before executing it.
set -x

# Run additional checks when inside a Nix development shell:
if [[ -n "${IN_NIX_SHELL:-}" ]]; then
    # Check Nix formatting:
    find . -type f -name '*.nix' -print0 | xargs -0 nix fmt -- --check

    # Check shell scripts:
    find . -type f -name '*.sh' -print0| xargs -0 shellcheck

    # Check TOML formatting:
    taplo check
fi

# Check that all crates adhere to workspace-level lints:
cargo workspace-lints

# Check syntax formatting:
cargo fmt --all -- --check

# Check comment line lengths:
scripts/checks/lint/comment-length.sh

# Check that clippy is happy:
CARGO_BUILD_WARNINGS=deny \
  cargo clippy --all-targets --all-features --keep-going
