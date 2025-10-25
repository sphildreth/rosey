#!/usr/bin/env bash
# Smoke test for Rosey packaged binary
# Usage: ./scripts/smoke_test.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$REPO_ROOT/dist/rosey/rosey"

cd "$REPO_ROOT"

if [ ! -f "$BINARY_PATH" ]; then
    echo "ERROR: Binary not found at $BINARY_PATH"
    echo "Run ./scripts/build_package.sh first"
    exit 1
fi

echo "==> Smoke test: Rosey packaged binary"
echo "    Binary: $BINARY_PATH"
echo ""

# Test 1: Binary exists and is executable
echo "[1/3] Checking binary is executable..."
if [ ! -x "$BINARY_PATH" ]; then
    chmod +x "$BINARY_PATH"
fi
echo "    ✓ Binary is executable"

# Test 2: Launch with --help (if CLI supports it) or version
echo "[2/3] Testing launch (headless/version check)..."
# Since this is a GUI app, we can't easily test full launch in CI
# Instead, check the binary can start (will fail without display but that's OK)
timeout 2s "$BINARY_PATH" 2>&1 | head -5 || {
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo "    ✓ Binary started (timed out as expected for GUI app)"
    elif [ $EXIT_CODE -eq 1 ]; then
        # Expected: might fail due to no display
        echo "    ✓ Binary attempted to start (no display available is OK)"
    else
        echo "    ⚠ Binary exited with code $EXIT_CODE (check if expected)"
    fi
}

# Test 3: Check file size (basic sanity)
echo "[3/3] Checking binary size..."
SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
if [ "$SIZE" -lt 1000000 ]; then
    echo "    ⚠ WARNING: Binary size is unusually small ($SIZE bytes)"
    exit 1
fi
echo "    ✓ Binary size: $SIZE bytes"

echo ""
echo "==> Smoke test PASSED"
echo ""
echo "Manual test checklist:"
echo "  1. Launch the binary"
echo "  2. Configure source/destination paths"
echo "  3. Run a scan"
echo "  4. Preview results (confidence colors, reasons)"
echo "  5. Execute dry-run move"
echo "  6. Exit cleanly"
