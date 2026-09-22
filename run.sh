#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/glance"
[[ -x "$BIN" ]] || BIN="$ROOT/target/debug/glance"
exec "$BIN" "$@"
