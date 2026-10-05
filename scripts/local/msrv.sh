#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
repo_root="$(git -C "$(dirname "$0")/../.." rev-parse --show-toplevel)"
cd "$repo_root"

# Copy the read-only checkout into the container's writable filesystem:
docker run --rm -it \
    --platform linux/amd64 \
    -v "$repo_root:/src:ro" \
    rust:latest \
    bash -c '
        cp -r /src /tmp/project &&
        cd /tmp/project &&
        cargo install cargo-msrv --locked &&
        cargo msrv find
    '
