# 🍓 Raspberry Pi PyTorch Linking Troubleshooting Guide

## Problem Overview

The error you're encountering is a **PyTorch API compatibility issue** between:
- `torch-sys = "0.17.0"` (used by `rust-bert = "0.23.0"`)
- PyTorch 2.8.0+ (which has removed/changed many internal APIs)

## Error Analysis

The compilation errors show that many PyTorch internal functions have been removed or changed:
- `torch::_assert_scalar` → removed
- `torch::_batch_norm_no_update` → removed  
- `torch::_cslt_sparse_mm` → API changed
- `torch::_efficient_attention_backward` → too many arguments
- And many more...

## Solutions

### 🎯 Solution 1: Downgrade PyTorch (Recommended)

**Why this works:** PyTorch 2.1.2 is the last version that maintains API compatibility with `torch-sys 0.17.0`.

```bash
# Run the updated fix script
./fix-pytorch-linking.sh
```

**What it does:**
- Downgrades PyTorch to 2.1.2
- Fixes NumPy version conflicts
- Sets correct environment variables
- Builds with verbose output

### 🚀 Solution 2: Upgrade rust-bert

**Why this works:** `rust-bert 0.25.0` uses a newer `torch-sys` version compatible with PyTorch 2.8.0+.

```bash
# Use the comprehensive script
./fix-pytorch-pi-comprehensive.sh
```

**What it does:**
- Upgrades `rust-bert` to 0.25.0
- Maintains your current PyTorch version
- Falls back to Solution 1 if needed

### 🔧 Solution 3: Manual Fix

If the scripts don't work, try this manual approach:

```bash
# 1. Clean everything
cargo clean
rm -rf target/release/build/torch-sys-*

# 2. Activate virtual environment
source ~/pytorch-venv/bin/activate

# 3. Downgrade PyTorch
pip uninstall torch torchvision torchaudio -y
pip install torch==2.1.2 torchvision==0.16.2 torchaudio==2.1.2 --index-url https://download.pytorch.org/whl/cpu

# 4. Fix NumPy if needed
pip uninstall numpy -y
pip install "numpy<2.0"

# 5. Set environment variables
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | sed 's/__init__.py/lib/')"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
export LIBTORCH_INCLUDE="$(python3 -c "import torch; print(torch.__file__)" | sed 's/__init__.py//')"
export LIBTORCH_USE_PYTORCH=1
export LIBTORCH_CXX11_ABI=0
export LIBTORCH_STATIC=0

# 6. Build
cargo build --release
```

## Version Compatibility Matrix

| rust-bert | tch-rs | PyTorch Compatible Versions |
|-----------|--------|----------------------------|
| 0.23.0    | 0.17.0 | 2.8.0 (with bypass flag for older) |
| 0.24.0    | 0.18.0 | 2.8.0+                     |
| 0.25.0    | 0.19.0 | 2.8.0+                     |

**Note:** tch-rs officially requires PyTorch 2.8.0, but can work with older versions using `LIBTORCH_BYPASS_VERSION_CHECK=1`

## Common Issues & Fixes

### Issue: "No module named 'torch'"
```bash
# Ensure virtual environment is activated
source ~/pytorch-venv/bin/activate
python3 -c "import torch; print(torch.__version__)"
```

### Issue: "Library not found"
```bash
# Check if PyTorch libraries exist
ls -la ~/pytorch-venv/lib/python3.*/site-packages/torch/lib/
```

### Issue: "Wrong architecture"
```bash
# Ensure you're using ARM64 PyTorch
file ~/pytorch-venv/lib/python3.*/site-packages/torch/lib/*.so
```

### Issue: "Memory exhausted"
```bash
# Increase swap space or reduce parallel jobs
export CARGO_BUILD_JOBS=1
cargo build --release
```

### Issue: "tch-rs expects PyTorch 2.8.0, got 2.1.2"
```bash
# Quick fix - bypass version check
./fix-torch-sys-version.sh

# Or manually:
export LIBTORCH_BYPASS_VERSION_CHECK=1
cargo clean
cargo build --release

# Or upgrade to PyTorch 2.8.0 (recommended):
pip install torch==2.8.0 torchvision==0.23.0 torchaudio==2.8.0 --index-url https://download.pytorch.org/whl/cpu
```

## Performance Optimization

### For Raspberry Pi 4 (4GB+ RAM):
```bash
# Use all cores for compilation
export CARGO_BUILD_JOBS=$(nproc)
cargo build --release
```

### For Raspberry Pi 3 or lower RAM:
```bash
# Use single core to avoid memory issues
export CARGO_BUILD_JOBS=1
cargo build --release
```

## Verification Steps

After successful build, verify everything works:

```bash
# 1. Check if binary was created
ls -la target/release/finbert-rs

# 2. Test the API
cargo run --release &
sleep 10
curl http://127.0.0.1:3000/health

# 3. Test sentiment analysis
curl http://127.0.0.1:3000/analyze
```

## Environment Variables Reference

| Variable | Purpose | Example |
|----------|---------|---------|
| `LIBTORCH` | PyTorch library path | `/home/user/pytorch-venv/lib/python3.11/site-packages/torch/lib/` |
| `LIBTORCH_INCLUDE` | PyTorch headers path | `/home/user/pytorch-venv/lib/python3.11/site-packages/torch/` |
| `LD_LIBRARY_PATH` | Library search path | `$LIBTORCH:$LD_LIBRARY_PATH` |
| `LIBTORCH_USE_PYTORCH` | Use PyTorch installation | `1` |
| `LIBTORCH_CXX11_ABI` | C++11 ABI setting | `0` |
| `LIBTORCH_STATIC` | Static linking | `0` |

## Troubleshooting Checklist

- [ ] Python 3.8-3.11 installed
- [ ] Virtual environment created and activated
- [ ] PyTorch 2.1.2 installed (or compatible version)
- [ ] NumPy < 2.0 installed
- [ ] Environment variables set correctly
- [ ] Build cache cleaned
- [ ] Sufficient memory available (4GB+ recommended)
- [ ] ARM64 architecture detected correctly

## Getting Help

If you're still experiencing issues:

1. **Check the logs:** Look for specific error messages in the build output
2. **Verify versions:** Ensure all versions match the compatibility matrix
3. **Memory check:** Ensure your Pi has enough RAM/swap
4. **Architecture:** Confirm you're using ARM64 PyTorch, not x86_64

## Quick Commands Reference

```bash
# Check current versions
python3 -c "import torch; print(torch.__version__)"
python3 -c "import numpy; print(numpy.__version__)"

# Clean and rebuild
cargo clean && cargo build --release

# Run with debug output
RUST_LOG=debug cargo run --release

# Check system resources
free -h
df -h
```

---

**Note:** This guide is specifically for Raspberry Pi ARM64 architecture. For other platforms, the solutions may differ.
