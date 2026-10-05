#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

SELF="$(basename "$0")"

todos_in_filenames="$(
  rg --files \
    | rg -ivF "$SELF" \
    | rg -i 'todos?' \
    || true
)"

todos_in_contents="$(
  rg --line-number --column -i 'todos?' . \
    --glob "!$SELF" \
    --glob "!justfile" \
    || true
)"

if [[ -n "$todos_in_filenames" ]]; then
  echo "=== Filenames ==="
  printf '%s\n' "$todos_in_filenames"
fi

if [[ -n "$todos_in_contents" ]]; then
  echo "=== Contents ==="
  printf '%s\n' "$todos_in_contents"
fi
