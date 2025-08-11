# 🔧 tch-rs PyTorch Compatibility Guide

Based on the official [tch-rs documentation](https://github.com/LaurentMazare/tch-rs), this guide provides solutions for PyTorch version compatibility issues.

## 📋 Official Requirements

According to the tch-rs documentation:

- **Required PyTorch Version**: v2.8.0
- **Supported Platforms**: Linux, macOS, Windows
- **Installation Methods**: System-wide, Python PyTorch, Manual libtorch

## 🎯 The Problem

The error you're seeing:
```
Error: this tch version expects PyTorch 2.8.0, got 2.1.2, this check can be bypassed by setting the LIBTORCH_BYPASS_VERSION_CHECK environment variable
```

This occurs because:
- `tch-rs` (used by `rust-bert`) expects PyTorch 2.8.0
- Your system has PyTorch 2.1.2 installed
- The version check is enforced by the build system

## 🛠️ Solutions

### Solution 1: Bypass Version Check (Quick Fix)

```bash
# Set the bypass environment variable
export LIBTORCH_BYPASS_VERSION_CHECK=1

# Clean and rebuild
cargo clean
cargo build --release
```

**Use our quick fix script:**
```bash
./fix-torch-sys-version.sh
```

### Solution 2: Upgrade to PyTorch 2.8.0 (Recommended)

```bash
# Activate your virtual environment
source ~/pytorch-venv/bin/activate

# Upgrade to PyTorch 2.8.0
pip uninstall torch torchvision torchaudio -y
pip install torch==2.8.0 torchvision==0.23.0 torchaudio==2.8.0 --index-url https://download.pytorch.org/whl/cpu

# Verify installation
python3 -c "import torch; print(torch.__version__)"
```

### Solution 3: Use Our Updated Install Script

```bash
# The updated install-and-run.sh now handles this automatically
./install-and-run.sh --build-only
```

## 🔧 Environment Variables

Based on the official documentation, these environment variables are important:

| Variable | Purpose | Example |
|----------|---------|---------|
| `LIBTORCH_USE_PYTORCH` | Use Python PyTorch installation | `1` |
| `LIBTORCH_BYPASS_VERSION_CHECK` | Bypass version compatibility check | `1` |
| `LIBTORCH` | Path to libtorch installation | `/path/to/libtorch` |
| `LIBTORCH_INCLUDE` | Path to libtorch headers | `/path/to/libtorch/` |
| `LIBTORCH_LIB` | Path to libtorch libraries | `/path/to/libtorch/lib` |
| `LD_LIBRARY_PATH` | Library search path | `/path/to/libtorch/lib:$LD_LIBRARY_PATH` |

## 📦 Installation Methods

### Method 1: Python PyTorch (Recommended for our use case)

```bash
# Set environment variable
export LIBTORCH_USE_PYTORCH=1

# Install PyTorch via pip
pip install torch==2.8.0 torchvision==0.23.0 torchaudio==2.8.0 --index-url https://download.pytorch.org/whl/cpu
```

### Method 2: Manual libtorch Installation

```bash
# Download libtorch from PyTorch website
# Extract to /path/to/libtorch

# Set environment variables
export LIBTORCH=/path/to/libtorch
export LIBTORCH_INCLUDE=/path/to/libtorch/
export LIBTORCH_LIB=/path/to/libtorch/lib
export LD_LIBRARY_PATH=/path/to/libtorch/lib:$LD_LIBRARY_PATH
```

### Method 3: System-wide Installation

```bash
# On Linux, install to /usr/lib/libtorch.so
# The build script will automatically find it
```

## 🍓 Raspberry Pi Specific Notes

### ARM64 Compatibility

- PyTorch 2.8.0 is available for ARM64 (Raspberry Pi 4)
- Use CPU-only builds for better compatibility
- Memory requirements: 4GB+ recommended

### Installation Commands for Pi

```bash
# For Raspberry Pi ARM64
pip install torch==2.8.0 torchvision==0.23.0 torchaudio==2.8.0 --index-url https://download.pytorch.org/whl/cpu

# If PyTorch 2.8.0 is not available, use bypass method
export LIBTORCH_BYPASS_VERSION_CHECK=1
cargo build --release
```

## 🔍 Troubleshooting

### Common Issues

**Issue: "No matching distribution found for torch==2.8.0"**
```bash
# Try the bypass method
export LIBTORCH_BYPASS_VERSION_CHECK=1
cargo build --release
```

**Issue: "error while loading shared libraries: libtorch_cpu.so"**
```bash
# Add libtorch to library path
export LD_LIBRARY_PATH=/path/to/libtorch/lib:$LD_LIBRARY_PATH
```

**Issue: "Failed to initialize NumPy"**
```bash
# Downgrade NumPy to 1.x
pip uninstall numpy -y
pip install "numpy<2.0"
```

### Debug Commands

```bash
# Check PyTorch version
python3 -c "import torch; print(torch.__version__)"

# Check libtorch path
python3 -c "import torch; print(torch.__file__)"

# Check environment variables
echo "LIBTORCH_USE_PYTORCH: ${LIBTORCH_USE_PYTORCH:-'not set'}"
echo "LIBTORCH_BYPASS_VERSION_CHECK: ${LIBTORCH_BYPASS_VERSION_CHECK:-'not set'}"
echo "LIBTORCH: ${LIBTORCH:-'not set'}"
```

## 📚 References

- [tch-rs GitHub Repository](https://github.com/LaurentMazare/tch-rs)
- [tch-rs Documentation](https://docs.rs/tch/)
- [PyTorch Installation Guide](https://pytorch.org/get-started/locally/)
- [rust-bert Documentation](https://github.com/guillaume-be/rust-bert)

## 🎯 Quick Reference

### For Immediate Fix
```bash
export LIBTORCH_BYPASS_VERSION_CHECK=1
cargo clean
cargo build --release
```

### For Long-term Solution
```bash
pip install torch==2.8.0 torchvision==0.23.0 torchaudio==2.8.0 --index-url https://download.pytorch.org/whl/cpu
```

### For Raspberry Pi
```bash
./fix-torch-sys-version.sh
```

---

**Note:** This guide is based on the official tch-rs documentation and our experience with the FinBERT Rust API project.
