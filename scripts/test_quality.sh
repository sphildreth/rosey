#!/bin/bash

# Local Quality Check Script
# Mimics the CI checks for linting, testing, and UI smoke test

echo "=== Rosey Local Quality Check ==="
echo

# Check if we're in the right directory
if [ ! -f "pyproject.toml" ]; then
    echo "Error: Run this script from the rosey project root directory"
    exit 1
fi

#!/bin/bash

# Local Quality Check Script
# Mimics the CI checks for linting, testing, and UI smoke test

echo "=== Rosey Local Quality Check ==="
echo

# Check if we're in the right directory
if [ ! -f "pyproject.toml" ]; then
    echo "Error: Run this script from the rosey project root directory"
    exit 1
fi

# Activate virtual environment if it exists
if [ -f ".venv/bin/activate" ]; then
    echo "Activating virtual environment..."
    source .venv/bin/activate
elif [ -f "venv/bin/activate" ]; then
    echo "Activating virtual environment..."
    source venv/bin/activate
else
    echo "Warning: No virtual environment found. Make sure dependencies are installed."
    echo "Run: python -m venv .venv && source .venv/bin/activate && pip install -e \".[dev]\""
    echo
fi

FAILED=0

echo "1. Running ruff check..."
if ruff check .; then
    echo "✓ ruff check passed"
else
    echo "✗ ruff check failed"
    FAILED=1
fi
echo

echo "2. Running ruff format check..."
if ruff format --check .; then
    echo "✓ ruff format check passed"
else
    echo "✗ ruff format check failed"
    FAILED=1
fi
echo

echo "3. Running mypy..."
if mypy src/rosey; then
    echo "✓ mypy check passed"
else
    echo "✗ mypy check failed"
    FAILED=1
fi
echo

echo "4. Running pytest with coverage..."
if pytest -q --cov=rosey --cov-report=xml; then
    echo "✓ pytest passed"
else
    echo "✗ pytest failed"
    FAILED=1
fi
echo

echo "5. Running UI smoke test..."
# Check if xvfb is available (Linux)
if command -v xvfb-run &> /dev/null; then
    if xvfb-run -a python -c "from rosey.app import main; import sys; sys.exit(0)"; then
        echo "✓ UI smoke test passed"
    else
        echo "✗ UI smoke test failed"
        FAILED=1
    fi
else
    echo "Warning: xvfb-run not found. Skipping UI smoke test."
    echo "On Ubuntu/Debian: sudo apt-get install xvfb"
    echo "On other systems: install Xvfb"
fi

echo
if [ $FAILED -eq 0 ]; then
    echo "=== All quality checks passed! ==="
    exit 0
else
    echo "=== Some quality checks failed! ==="
    exit 1
fi
