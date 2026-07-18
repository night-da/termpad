#!/usr/bin/env bash
# Install termpad to ~/.cargo/bin so you can run: termpad [file]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo install --path . --force

echo ""
echo "Installed: termpad -> ~/.cargo/bin/termpad"
echo ""
echo "Make sure ~/.cargo/bin is in your PATH, then run:"
echo "  termpad              # empty buffer"
echo "  termpad README.md    # open a file"
