#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Print each command before executing it.
set -x

scripts/checks/lint.sh
scripts/checks/test.sh
scripts/checks/publish.sh
