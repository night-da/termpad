#!/usr/bin/env bash
# Run before commit: format check, clippy, tests
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "cargo fmt --check ..."
cargo fmt -- --check

echo "cargo clippy ..."
cargo clippy -- -D warnings

echo "cargo test ..."
cargo test

echo "All checks passed."
