#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}[INFO]${NC} 🔧 Fixing PyTorch linking for ARM64..."

# Step 1: Clean build cache
echo -e "${BLUE}[INFO]${NC} 🧹 Cleaning build cache..."
cargo clean
rm -rf target/release/build/torch-sys-*
rm -rf ~/.cargo/registry/cache/*/torch-sys*

# Step 2: Activate virtual environment
echo -e "${BLUE}[INFO]${NC} 🔌 Activating virtual environment..."
source ~/pytorch-venv/bin/activate

# Step 3: Set environment variables
echo -e "${BLUE}[INFO]${NC} ⚙️ Setting environment variables..."

# Set LIBTORCH to the lib directory
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib/')"
echo -e "${BLUE}[INFO]${NC} 📁 LIBTORCH: $LIBTORCH"

# Set LD_LIBRARY_PATH
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
echo -e "${BLUE}[INFO]${NC} 🔗 LD_LIBRARY_PATH: $LD_LIBRARY_PATH"

# Set the correct include path for headers (without /include suffix since torch-sys adds it)
export LIBTORCH_INCLUDE="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py//')"
echo -e "${BLUE}[INFO]${NC} 📁 LIBTORCH_INCLUDE: $LIBTORCH_INCLUDE"

# Step 4: Check library architecture
echo -e "${BLUE}[INFO]${NC} 🔍 Checking library architecture..."
for lib in "$LIBTORCH"/lib*.so; do
    if [[ -f "$lib" ]]; then
        arch=$(file "$lib" | grep -o "ELF [0-9]*-bit")
        echo -e "${YELLOW}[WARNING]${NC} $(basename "$lib"): $arch"
    fi
done

# Step 5: Check for conflicting x86_64 libraries
echo -e "${BLUE}[INFO]${NC} 🔍 Checking for conflicting x86_64 libraries..."
if find /usr/lib -name "libtorch*.so" 2>/dev/null | grep -q .; then
    echo -e "${YELLOW}[WARNING]${NC} Found system PyTorch libraries that may conflict"
    find /usr/lib -name "libtorch*.so" 2>/dev/null
fi

# Step 6: Build with correct environment
echo -e "${BLUE}[INFO]${NC} ⚡️ Building with ARM64 PyTorch..."
cargo build --release
