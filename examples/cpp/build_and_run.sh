#!/usr/bin/env bash
# Build and run the hyprlog C++ FFI example
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== hyprlog C++ FFI Example Build Script ==="
echo ""

# Step 1: Build Rust library
echo "[1/3] Building Rust library (cargo build --release)..."
cd "$PROJECT_ROOT"
cargo build --release
echo "      -> libhl_ffi.so created"
echo ""

# Step 2: Build C++ example
echo "[2/3] Building C++ example (cmake + make)..."
cd "$SCRIPT_DIR"
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make
echo "      -> hyprlog_example created"
echo ""

# Step 3: Run
echo "[3/3] Running example..."
echo ""
echo "========================================"
./hyprlog_example
echo "========================================"
echo ""
echo "Done!"
