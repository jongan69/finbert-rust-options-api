#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}[INFO]${NC} 🔧 Comprehensive PyTorch Fix for Raspberry Pi ARM64"
echo -e "${BLUE}[INFO]${NC} This script provides multiple solutions for torch-sys compatibility issues"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check Python version
check_python_version() {
    local python_version=$(python3 --version 2>&1 | grep -oP '\d+\.\d+' | head -1)
    echo -e "${BLUE}[DEBUG]${NC} Python version: $python_version"
    
    if [[ "$python_version" < "3.8" ]]; then
        echo -e "${RED}[ERROR]${NC} Python 3.8+ required, found $python_version"
        return 1
    elif [[ "$python_version" > "3.11" ]]; then
        echo -e "${YELLOW}[WARNING]${NC} Python $python_version may have compatibility issues with PyTorch 2.1.2"
    fi
    return 0
}

# Function to clean build cache
clean_build_cache() {
    echo -e "${BLUE}[INFO]${NC} 🧹 Cleaning build cache..."
    cargo clean
    rm -rf target/release/build/torch-sys-*
    rm -rf ~/.cargo/registry/cache/*/torch-sys*
}

# Function to activate virtual environment
activate_venv() {
    if [[ -f ~/pytorch-venv/bin/activate ]]; then
        echo -e "${BLUE}[INFO]${NC} 🔌 Activating virtual environment..."
        source ~/pytorch-venv/bin/activate
    else
        echo -e "${RED}[ERROR]${NC} Virtual environment not found at ~/pytorch-venv"
        echo -e "${BLUE}[INFO]${NC} Creating virtual environment..."
        python3 -m venv ~/pytorch-venv
        source ~/pytorch-venv/bin/activate
    fi
}

# Function to fix NumPy version
fix_numpy() {
    local numpy_version=$(python3 -c "import numpy; print(numpy.__version__)" 2>/dev/null || echo "not installed")
    echo -e "${BLUE}[DEBUG]${NC} Current NumPy version: $numpy_version"
    
    if [[ "$numpy_version" == "2."* ]]; then
        echo -e "${YELLOW}[WARNING]${NC} NumPy 2.x detected, downgrading to NumPy 1.x for PyTorch compatibility..."
        pip uninstall numpy -y
        pip install "numpy<2.0"
        echo -e "${BLUE}[DEBUG]${NC} New NumPy version: $(python3 -c "import numpy; print(numpy.__version__)")"
    fi
}

# Function to downgrade PyTorch (Solution 1)
downgrade_pytorch() {
    echo -e "${PURPLE}[SOLUTION 1]${NC} 🔄 Downgrading PyTorch to 2.1.2 for torch-sys 0.17.0 compatibility"
    
    local pytorch_version=$(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "not installed")
    echo -e "${BLUE}[DEBUG]${NC} Current PyTorch version: $pytorch_version"
    
    if [[ "$pytorch_version" != "2.1.2" ]]; then
        echo -e "${BLUE}[INFO]${NC} Installing PyTorch 2.1.2..."
        pip uninstall torch torchvision torchaudio -y
        pip install torch==2.1.2 torchvision==0.16.2 torchaudio==2.1.2 --index-url https://download.pytorch.org/whl/cpu
        
        local new_version=$(python3 -c "import torch; print(torch.__version__)")
        if [[ "$new_version" == "2.1.2" ]]; then
            echo -e "${GREEN}[SUCCESS]${NC} ✅ PyTorch downgraded to 2.1.2"
        else
            echo -e "${RED}[ERROR]${NC} Failed to downgrade PyTorch"
            return 1
        fi
    else
        echo -e "${GREEN}[INFO]${NC} PyTorch already at 2.1.2"
    fi
}

# Function to upgrade rust-bert (Solution 2)
upgrade_rust_bert() {
    echo -e "${PURPLE}[SOLUTION 2]${NC} ⬆️ Upgrading rust-bert to 0.25.0 for newer PyTorch compatibility"
    
    # Backup original Cargo.toml
    cp Cargo.toml Cargo.toml.backup
    
    # Update rust-bert version
    sed -i 's/rust-bert = "0.23.0"/rust-bert = "0.25.0"/' Cargo.toml
    
    echo -e "${BLUE}[INFO]${NC} Updated Cargo.toml with rust-bert 0.25.0"
    echo -e "${BLUE}[INFO]${NC} This version is compatible with PyTorch 2.8.0+"
}

# Function to set environment variables
set_environment_variables() {
    echo -e "${BLUE}[INFO]${NC} ⚙️ Setting environment variables..."
    
    # Get PyTorch path
    local torch_path=$(python3 -c "import torch; print(torch.__file__)" 2>/dev/null)
    if [[ -z "$torch_path" ]]; then
        echo -e "${RED}[ERROR]${NC} PyTorch not found"
        return 1
    fi
    
    # Set LIBTORCH to the lib directory
    export LIBTORCH="$(echo "$torch_path" | sed 's/__init__.py/lib/')"
    echo -e "${BLUE}[INFO]${NC} 📁 LIBTORCH: $LIBTORCH"
    
    # Set LD_LIBRARY_PATH
    export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
    echo -e "${BLUE}[INFO]${NC} 🔗 LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
    
    # Set include path
    export LIBTORCH_INCLUDE="$(echo "$torch_path" | sed 's/__init__.py//')"
    echo -e "${BLUE}[INFO]${NC} 📁 LIBTORCH_INCLUDE: $LIBTORCH_INCLUDE"
    
    # Set additional environment variables
    export LIBTORCH_USE_PYTORCH=1
    export LIBTORCH_CXX11_ABI=0
    export LIBTORCH_STATIC=0
}

# Function to build the project
build_project() {
    echo -e "${BLUE}[INFO]${NC} ⚡️ Building project..."
    
    # Try building with verbose output
    if cargo build --release --verbose; then
        echo -e "${GREEN}[SUCCESS]${NC} ✅ Build completed successfully!"
        echo -e "${BLUE}[INFO]${NC} 🚀 You can now run: cargo run --release"
        return 0
    else
        echo -e "${RED}[ERROR]${NC} ❌ Build failed"
        return 1
    fi
}

# Function to restore original Cargo.toml
restore_cargo_toml() {
    if [[ -f Cargo.toml.backup ]]; then
        echo -e "${BLUE}[INFO]${NC} 🔄 Restoring original Cargo.toml..."
        mv Cargo.toml.backup Cargo.toml
    fi
}

# Main execution
main() {
    # Check prerequisites
    if ! command_exists cargo; then
        echo -e "${RED}[ERROR]${NC} Cargo not found. Please install Rust first."
        exit 1
    fi
    
    if ! command_exists python3; then
        echo -e "${RED}[ERROR]${NC} Python3 not found. Please install Python 3.8+ first."
        exit 1
    fi
    
    if ! check_python_version; then
        exit 1
    fi
    
    # Clean build cache
    clean_build_cache
    
    # Activate virtual environment
    activate_venv
    
    # Fix NumPy version
    fix_numpy
    
    # Get current PyTorch version
    local pytorch_version=$(python3 -c "import torch; print(torch.__version__)" 2>/dev/null || echo "not installed")
    echo -e "${BLUE}[DEBUG]${NC} Current PyTorch version: $pytorch_version"
    
    # Choose solution based on PyTorch version
    if [[ "$pytorch_version" == "2.8."* ]] || [[ "$pytorch_version" == "2.7."* ]] || [[ "$pytorch_version" == "2.6."* ]] || [[ "$pytorch_version" == "2.5."* ]] || [[ "$pytorch_version" == "2.4."* ]] || [[ "$pytorch_version" == "2.3."* ]] || [[ "$pytorch_version" == "2.2."* ]]; then
        echo -e "${YELLOW}[WARNING]${NC} PyTorch $pytorch_version detected - API compatibility issues expected"
        
        echo -e "${BLUE}[INFO]${NC} Choose a solution:"
        echo -e "  1. Downgrade PyTorch to 2.1.2 (recommended for stability)"
        echo -e "  2. Upgrade rust-bert to 0.25.0 (for newer PyTorch compatibility)"
        echo -e "  3. Try both solutions"
        
        read -p "Enter choice (1/2/3): " choice
        
        case $choice in
            1)
                downgrade_pytorch
                set_environment_variables
                build_project
                ;;
            2)
                upgrade_rust_bert
                set_environment_variables
                if ! build_project; then
                    echo -e "${YELLOW}[WARNING]${NC} Solution 2 failed, trying Solution 1..."
                    restore_cargo_toml
                    downgrade_pytorch
                    set_environment_variables
                    build_project
                fi
                ;;
            3)
                echo -e "${BLUE}[INFO]${NC} Trying both solutions..."
                upgrade_rust_bert
                set_environment_variables
                if ! build_project; then
                    echo -e "${YELLOW}[WARNING]${NC} Solution 2 failed, trying Solution 1..."
                    restore_cargo_toml
                    downgrade_pytorch
                    set_environment_variables
                    build_project
                fi
                ;;
            *)
                echo -e "${RED}[ERROR]${NC} Invalid choice"
                exit 1
                ;;
        esac
    else
        echo -e "${GREEN}[INFO]${NC} PyTorch version $pytorch_version should be compatible"
        set_environment_variables
        build_project
    fi
}

# Run main function
main "$@"
