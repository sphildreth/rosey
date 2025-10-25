#!/usr/bin/env bash
# Build Rosey package with PyInstaller
# Usage: ./scripts/build_package.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

echo "==> Installing PyInstaller..."
pip install pyinstaller

echo ""
echo "==> Building Rosey package..."
pyinstaller rosey.spec --clean --noconfirm

echo ""
echo "==> Package built successfully!"
echo "    Binary location: dist/rosey/"
echo ""
echo "To run smoke tests:"
echo "    ./scripts/smoke_test.sh"
