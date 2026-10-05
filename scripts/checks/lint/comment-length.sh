#!/usr/bin/env bash
set -euo pipefail

# This script can be replaced with rustfmt when the feature becomes stable.
# https://github.com/rust-lang/rustfmt/issues/3349

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Find comments exceeding 80 graphemes
! rg --no-heading '^[[:space:]]*//.{79,}' -g '*.rs' -g '!target/' -n
