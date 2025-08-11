#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}[INFO]${NC} 🔧 Fixing tch-rs version compatibility issue..."

# Activate virtual environment
if [[ -f ~/pytorch-venv/bin/activate ]]; then
    echo -e "${BLUE}[INFO]${NC} Activating virtual environment..."
    source ~/pytorch-venv/bin/activate
else
    echo -e "${RED}[ERROR]${NC} Virtual environment not found at ~/pytorch-venv"
    exit 1
fi

# Check current PyTorch version
current_version=$(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "not installed")
echo -e "${BLUE}[INFO]${NC} Current PyTorch version: $current_version"

# Set environment variable to bypass version check (needed for all versions with torch-sys 0.17.0)
export LIBTORCH_BYPASS_VERSION_CHECK=1
echo -e "${BLUE}[INFO]${NC} Set LIBTORCH_BYPASS_VERSION_CHECK=1 (torch-sys 0.17.0 has specific version expectations)"

# Clean build cache
echo -e "${BLUE}[INFO]${NC} Cleaning build cache..."
cargo clean
rm -rf target/release/build/torch-sys-*

# Set PyTorch environment variables
torch_path=$(python3 -c "import torch; print(torch.__file__)" 2>/dev/null)
if [[ -n "$torch_path" ]]; then
    export LIBTORCH="$(echo "$torch_path" | sed 's/__init__.py/lib/')"
    export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
    export LIBTORCH_INCLUDE="$(echo "$torch_path" | sed 's/__init__.py//')"
    export LIBTORCH_USE_PYTORCH=1
    export LIBTORCH_CXX11_ABI=0
    export LIBTORCH_STATIC=0
    
    echo -e "${BLUE}[INFO]${NC} Environment variables set:"
    echo -e "${BLUE}[INFO]${NC}   LIBTORCH: $LIBTORCH"
    echo -e "${BLUE}[INFO]${NC}   LIBTORCH_BYPASS_VERSION_CHECK: $LIBTORCH_BYPASS_VERSION_CHECK"
fi

# Try to build
echo -e "${BLUE}[INFO]${NC} Building with version check bypassed..."
if cargo build --release; then
    echo -e "${GREEN}[SUCCESS]${NC} ✅ Build completed successfully!"
    echo -e "${BLUE}[INFO]${NC} 🚀 You can now run: cargo run --release"
else
    echo -e "${RED}[ERROR]${NC} ❌ Build still failed. Try upgrading PyTorch:"
    echo -e "${YELLOW}[SUGGESTION]${NC} pip install torch==2.8.0 torchvision==0.23.0 torchaudio==2.8.0 --index-url https://download.pytorch.org/whl/cpu"
    exit 1
fi
