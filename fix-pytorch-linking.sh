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

# Step 3: Check PyTorch version and fix compatibility issues
echo -e "${BLUE}[INFO]${NC} 🔍 Checking PyTorch version compatibility..."
PYTORCH_VERSION=$(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "not installed")
echo -e "${BLUE}[DEBUG]${NC} Current PyTorch version: $PYTORCH_VERSION"

# Check NumPy version first
NUMPY_VERSION=$(python3 -c "import numpy; print(numpy.__version__)" 2>/dev/null || echo "not installed")
echo -e "${BLUE}[DEBUG]${NC} Current NumPy version: $NUMPY_VERSION"

# Fix NumPy version if needed
if [[ "$NUMPY_VERSION" == "2."* ]]; then
    echo -e "${YELLOW}[WARNING]${NC} NumPy 2.x detected, downgrading to NumPy 1.x for PyTorch compatibility..."
    pip uninstall numpy -y
    pip install "numpy<2.0"
    echo -e "${BLUE}[DEBUG]${NC} New NumPy version: $(python3 -c "import numpy; print(numpy.__version__)")"
fi

# Check if we need to downgrade PyTorch for compatibility with torch-sys 0.17.0
# torch-sys 0.17.0 is compatible with PyTorch 2.0.x - 2.1.x
if [[ "$PYTORCH_VERSION" == "2.8."* ]] || [[ "$PYTORCH_VERSION" == "2.7."* ]] || [[ "$PYTORCH_VERSION" == "2.6."* ]] || [[ "$PYTORCH_VERSION" == "2.5."* ]] || [[ "$PYTORCH_VERSION" == "2.4."* ]] || [[ "$PYTORCH_VERSION" == "2.3."* ]] || [[ "$PYTORCH_VERSION" == "2.2."* ]]; then
    echo -e "${YELLOW}[WARNING]${NC} PyTorch $PYTORCH_VERSION has API compatibility issues with torch-sys 0.17.0"
    echo -e "${BLUE}[INFO]${NC} Downgrading to PyTorch 2.1.2 for maximum compatibility..."
    
    pip uninstall torch torchvision torchaudio -y
    pip install torch==2.1.2 torchvision==0.16.2 torchaudio==2.1.2 --index-url https://download.pytorch.org/whl/cpu
    
    # Verify the downgrade
    NEW_VERSION=$(python3 -c "import torch; print(torch.__version__)")
    echo -e "${BLUE}[DEBUG]${NC} New PyTorch version: $NEW_VERSION"
    
    if [[ "$NEW_VERSION" != "2.1.2" ]]; then
        echo -e "${RED}[ERROR]${NC} Failed to downgrade PyTorch to 2.1.2"
        exit 1
    fi
fi

# Step 4: Set environment variables
echo -e "${BLUE}[INFO]${NC} ⚙️ Setting environment variables..."

# Debug: Show Python torch path
echo -e "${BLUE}[DEBUG]${NC} Python torch path: $(python3 -c "import torch; print(torch.__file__)")"

# Set LIBTORCH to the lib directory
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib/')"
echo -e "${BLUE}[INFO]${NC} 📁 LIBTORCH: $LIBTORCH"
echo -e "${BLUE}[DEBUG]${NC} Checking if LIBTORCH directory exists: $(ls -la "$LIBTORCH" 2>/dev/null | head -1 || echo 'Directory not found')"

# Set LD_LIBRARY_PATH
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
echo -e "${BLUE}[INFO]${NC} 🔗 LD_LIBRARY_PATH: $LD_LIBRARY_PATH"

# Set the correct include path for headers (without /include suffix since torch-sys adds it)
export LIBTORCH_INCLUDE="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py//')"
echo -e "${BLUE}[INFO]${NC} 📁 LIBTORCH_INCLUDE: $LIBTORCH_INCLUDE"

# Debug: Check what the final include path will be
FINAL_INCLUDE_PATH="$LIBTORCH_INCLUDE/include"
echo -e "${BLUE}[DEBUG]${NC} Final include path (LIBTORCH_INCLUDE + /include): $FINAL_INCLUDE_PATH"
echo -e "${BLUE}[DEBUG]${NC} Checking if final include path exists: $(ls -la "$FINAL_INCLUDE_PATH" 2>/dev/null | head -1 || echo 'Directory not found')"

# Debug: Check for specific headers
echo -e "${BLUE}[DEBUG]${NC} Looking for torch.h: $(find "$FINAL_INCLUDE_PATH" -name "torch.h" 2>/dev/null | head -1 || echo 'Not found')"
echo -e "${BLUE}[DEBUG]${NC} Looking for engine.h: $(find "$FINAL_INCLUDE_PATH" -name "engine.h" 2>/dev/null | head -1 || echo 'Not found')"

# Set additional environment variables for better compatibility
export LIBTORCH_USE_PYTORCH=1
export LIBTORCH_CXX11_ABI=0
export LIBTORCH_STATIC=0

# Debug: Show all environment variables that torch-sys might use
echo -e "${BLUE}[DEBUG]${NC} Environment variables:"
echo -e "${BLUE}[DEBUG]${NC}   LIBTORCH_USE_PYTORCH: ${LIBTORCH_USE_PYTORCH:-'not set'}"
echo -e "${BLUE}[DEBUG]${NC}   LIBTORCH: $LIBTORCH"
echo -e "${BLUE}[DEBUG]${NC}   LIBTORCH_INCLUDE: $LIBTORCH_INCLUDE"
echo -e "${BLUE}[DEBUG]${NC}   LIBTORCH_LIB: ${LIBTORCH_LIB:-'not set'}"
echo -e "${BLUE}[DEBUG]${NC}   LIBTORCH_CXX11_ABI: ${LIBTORCH_CXX11_ABI:-'not set'}"
echo -e "${BLUE}[DEBUG]${NC}   LIBTORCH_STATIC: ${LIBTORCH_STATIC:-'not set'}"

# Step 5: Check library architecture
echo -e "${BLUE}[INFO]${NC} 🔍 Checking library architecture..."
for lib in "$LIBTORCH"/lib*.so; do
    if [[ -f "$lib" ]]; then
        arch=$(file "$lib" | grep -o "ELF [0-9]*-bit")
        echo -e "${YELLOW}[WARNING]${NC} $(basename "$lib"): $arch"
    fi
done

# Step 6: Check for conflicting x86_64 libraries
echo -e "${BLUE}[INFO]${NC} 🔍 Checking for conflicting x86_64 libraries..."
if find /usr/lib -name "libtorch*.so" 2>/dev/null | grep -q .; then
    echo -e "${YELLOW}[WARNING]${NC} Found system PyTorch libraries that may conflict"
    find /usr/lib -name "libtorch*.so" 2>/dev/null
fi

# Step 7: Build with correct environment
echo -e "${BLUE}[INFO]${NC} ⚡️ Building with ARM64 PyTorch 2.1.2..."
echo -e "${BLUE}[DEBUG]${NC} Running: cargo build --release"

# Try building with verbose output to catch any remaining issues
if cargo build --release --verbose; then
    echo -e "${GREEN}[SUCCESS]${NC} ✅ Build completed successfully!"
    echo -e "${BLUE}[INFO]${NC} 🚀 You can now run: cargo run --release"
else
    echo -e "${RED}[ERROR]${NC} ❌ Build failed. Here are some additional troubleshooting steps:"
    echo -e "${YELLOW}[TROUBLESHOOTING]${NC} 🔧 Try these steps:"
    echo -e "   1. Ensure you're using Python 3.8-3.11 (PyTorch 2.1.2 compatibility)"
    echo -e "   2. Check if your system has enough memory (4GB+ recommended)"
    echo -e "   3. Try building with: RUSTFLAGS='-C target-cpu=native' cargo build --release"
    echo -e "   4. If still failing, try: pip install torch==2.0.1 torchvision==0.15.2 torchaudio==2.0.2"
    exit 1
fi
