#!/usr/bin/env bash
set -euo pipefail

# Be in the repository root:
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

# Complete the release checks before asking for authorization to upload crates:
bash scripts/checks/publish.sh

# Require stdin to be connected to a terminal:
if [[ ! -t 0 ]]; then
    printf '%s\n' 'Refusing to publish without an interactive confirmation.' >&2
    exit 1
fi

cat <<'EOF'
The release checks passed. This will publish:
  thisversion-derive
  thisversion
EOF
read -r -p 'Type "publish" to upload these crates to crates.io: ' confirmation

if [[ "$confirmation" != "publish" ]]; then
    printf '%s\n' 'Publication cancelled.' >&2
    exit 1
fi

# The runtime crate depends on the derive crate, so publish the derive crate
# first:
cargo publish --locked -p thisversion-derive
cargo publish --locked -p thisversion
