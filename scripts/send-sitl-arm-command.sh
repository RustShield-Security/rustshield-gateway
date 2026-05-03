#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-127.0.0.1:14551}"
bind="${2:-127.0.0.1:0}"

cd "$repo_root"
cargo run --bin sitl-send-arm-command -- "$target" "$bind"
