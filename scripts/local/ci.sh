#!/usr/bin/env bash
set -euo pipefail

# Be in repo root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Point act at the Docker endpoint for the active context:
docker_context="$(docker context show)"
docker_host_format='{{ .Endpoints.docker.Host }}'
docker_host="$(docker context inspect "$docker_context" --format "$docker_host_format")"

# Run GitHub Actions CI locally:
DOCKER_HOST="$docker_host" nix run nixpkgs#act -- pull_request \
    --container-architecture linux/amd64 \
    -P ubuntu-latest=catthehacker/ubuntu:act-latest
