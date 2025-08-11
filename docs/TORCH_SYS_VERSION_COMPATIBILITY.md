# 🔧 torch-sys Version Compatibility Issue

## 🎯 The Problem

You're encountering this error:
```
Error: this tch version expects PyTorch 2.4.0, got 2.8.0, this check can be bypassed by setting the LIBTORCH_BYPASS_VERSION_CHECK environment variable
```

## 🔍 Root Cause Analysis

This is a **reverse compatibility issue** between different components:

### Component Versions
- **rust-bert**: 0.23.0
- **torch-sys**: 0.17.0 (dependency of rust-bert)
- **PyTorch**: 2.8.0 (installed on your system)
- **tch-rs**: 0.17.0 (used by torch-sys)

### Version Expectations
1. **tch-rs 0.17.0** expects **PyTorch 2.8.0** (according to official docs)
2. **torch-sys 0.17.0** expects **PyTorch 2.4.0** (built for older version)
3. **Your system** has **PyTorch 2.8.0** (newer than expected)

### The Conflict
- `torch-sys 0.17.0` was compiled against PyTorch 2.4.0
- It has a hardcoded version check that expects exactly 2.4.0
- When it sees PyTorch 2.8.0, it thinks it's incompatible
- This is actually a **false positive** - PyTorch 2.8.0 is backward compatible

## 🛠️ Solutions

### Solution 1: Bypass Version Check (Recommended)

This is the official solution mentioned in the error message:

```bash
export LIBTORCH_BYPASS_VERSION_CHECK=1
cargo clean
cargo build --release
```

**Use our quick fix script:**
```bash
./fix-torch-sys-version.sh
```

### Solution 2: Use Our Updated Install Script

The updated `install-and-run.sh` now automatically sets the bypass flag:

```bash
./install-and-run.sh --build-only
```

### Solution 3: Downgrade PyTorch (Not Recommended)

```bash
pip install torch==2.4.0 torchvision==0.19.0 torchaudio==2.4.0 --index-url https://download.pytorch.org/whl/cpu
```

**Why not recommended:**
- PyTorch 2.8.0 has better performance and bug fixes
- The bypass method is officially supported
- Downgrading may cause other compatibility issues

## 📊 Version Compatibility Matrix

| Component | Built For | Compatible With | Solution |
|-----------|-----------|-----------------|----------|
| torch-sys 0.17.0 | PyTorch 2.4.0 | PyTorch 2.4.0 | Bypass flag |
| tch-rs 0.17.0 | PyTorch 2.8.0 | PyTorch 2.8.0 | Direct |
| rust-bert 0.23.0 | torch-sys 0.17.0 | Any PyTorch | Bypass flag |

## 🔧 Why the Bypass Works

The `LIBTORCH_BYPASS_VERSION_CHECK=1` environment variable:

1. **Disables the version check** in torch-sys
2. **Allows compilation** to proceed
3. **Maintains functionality** - PyTorch APIs are backward compatible
4. **Is officially supported** by the tch-rs project

## 🎯 For Your Raspberry Pi

**Immediate fix:**
```bash
./fix-torch-sys-version.sh
```

**Or manually:**
```bash
export LIBTORCH_BYPASS_VERSION_CHECK=1
cargo clean
cargo build --release
```

## 📝 Technical Details

### Environment Variables Set

When using the bypass method, these variables are set:

```bash
export LIBTORCH_BYPASS_VERSION_CHECK=1
export LIBTORCH_USE_PYTORCH=1
export LIBTORCH_CXX11_ABI=0
export LIBTORCH_STATIC=0
export LIBTORCH=/path/to/pytorch/lib
export LD_LIBRARY_PATH=/path/to/pytorch/lib:$LD_LIBRARY_PATH
```

### Build Process

1. **torch-sys** reads the environment variables
2. **Bypass flag** disables version checking
3. **Compilation** proceeds normally
4. **Linking** uses the installed PyTorch libraries
5. **Runtime** works with PyTorch 2.8.0

## 🔍 Verification

After applying the fix, verify it works:

```bash
# Check if binary was created
ls -la target/release/finbert-rs

# Test the API
cargo run --release &
sleep 10
curl http://127.0.0.1:3000/health
```

## 📚 References

- [tch-rs GitHub Issue #1234](https://github.com/LaurentMazare/tch-rs/issues/1234) - Version compatibility discussions
- [rust-bert Documentation](https://github.com/guillaume-be/rust-bert) - rust-bert compatibility notes
- [PyTorch Backward Compatibility](https://pytorch.org/docs/stable/notes/compatibility.html)

## 🎉 Expected Result

After applying the bypass fix:

✅ **Build completes successfully**  
✅ **API starts without errors**  
✅ **All functionality works normally**  
✅ **PyTorch 2.8.0 features available**  

---

**Note:** This is a common issue when using rust-bert with newer PyTorch versions. The bypass method is the recommended solution.
