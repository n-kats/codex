#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  exit 0
fi

cd "$(dirname "${BASH_SOURCE[0]}")/.."
cd codex-rs
cargo build -p codex-linux-sandbox -p codex-bwrap
