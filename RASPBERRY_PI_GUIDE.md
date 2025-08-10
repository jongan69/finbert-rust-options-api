# 🍓 **FinBERT on Raspberry Pi - Complete Guide**

This guide will help you get FinBERT running on your Raspberry Pi with ARM64 architecture.

## 🎯 **The Problem**

Your Raspberry Pi runs ARM64 architecture, but PyTorch libraries are typically built for x86_64. This causes linking errors when trying to compile Rust projects that depend on `rust-bert` and `tch` (PyTorch bindings).

## 🔧 **Solutions**

### **Option 1: Install Pre-built ARM64 PyTorch (Recommended - Faster)**

This is the fastest approach and works for most cases:

```bash
# Make the script executable
chmod +x install-pytorch-pi.sh

# Run the installation script
./install-pytorch-pi.sh
```

**What this does:**
- Installs system dependencies
- Downloads pre-built ARM64 PyTorch wheels
- Sets up environment variables
- Verifies the installation

**Time:** ~10-30 minutes

### **Option 2: Build PyTorch from Source (More Reliable)**

If the pre-built approach doesn't work, build PyTorch from source:

```bash
# Make the script executable
chmod +x build-pytorch-pi.sh

# Run the build script
./build-pytorch-pi.sh
```

**What this does:**
- Installs all build dependencies
- Clones PyTorch source code
- Builds PyTorch for ARM64 architecture
- Installs the built libraries
- Sets up environment variables

**Time:** 2-4 hours (depending on Pi model)

**Requirements:**
- Raspberry Pi 4 with 4GB+ RAM recommended
- Adequate cooling (fan/heatsink)
- Stable power supply
- At least 8GB free space

## 🚀 **Quick Start**

### **Step 1: Check Your Pi Architecture**
```bash
uname -m
```
Should output: `aarch64` (for Pi 4) or `armv7l` (for older Pi models)

### **Step 2: Install PyTorch**
```bash
# Try the fast method first
./install-pytorch-pi.sh

# If that fails, use the build method
./build-pytorch-pi.sh
```

### **Step 3: Restart Terminal**
```bash
source ~/.bashrc
```

### **Step 4: Build Your Rust Project**
```bash
# Clean previous builds
cargo clean

# Build the project
cargo build --release
```

### **Step 5: Run the API**
```bash
cargo run
```

## 🔍 **Troubleshooting**

### **Common Issues**

#### **1. Memory Issues During Build**
```bash
# Increase swap space
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Change CONF_SWAPSIZE=100 to CONF_SWAPSIZE=2048
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

#### **2. Thermal Throttling**
```bash
# Monitor CPU temperature
vcgencmd measure_temp

# If > 80°C, add cooling or reduce build jobs
export MAX_JOBS=1
```

#### **3. PyTorch Installation Fails**
```bash
# Try alternative installation methods
pip3 install torch --no-cache-dir --force-reinstall
pip3 install torch --index-url https://download.pytorch.org/whl/cpu
```

#### **4. Rust Build Still Fails**
```bash
# Check if PyTorch is properly installed
python3 -c "import torch; print(torch.__version__)"

# Check environment variables
echo $LIBTORCH
echo $LD_LIBRARY_PATH

# Reinstall Rust dependencies
cargo clean
cargo update
```

### **Alternative: Cross-Compilation**

If building on Pi is too slow, build on your PC and transfer:

```bash
# On your PC (x86_64)
cargo build --release --target aarch64-unknown-linux-gnu

# Transfer binary to Pi
scp target/aarch64-unknown-linux-gnu/release/finbert-rs pi@your-pi-ip:/home/pi/
```

## 📊 **Performance Considerations**

### **Memory Usage**
- **FinBERT Model:** ~500MB RAM
- **PyTorch Runtime:** ~200MB RAM
- **Total:** ~1GB RAM minimum

### **CPU Usage**
- **Model Loading:** High CPU usage initially
- **Inference:** Moderate CPU usage
- **Recommendation:** Pi 4 with 4GB+ RAM

### **Storage**
- **PyTorch Libraries:** ~2GB
- **FinBERT Model:** ~500MB
- **Total:** ~3GB free space needed

## 🔧 **Optimization Tips**

### **1. Use Release Build**
```bash
cargo build --release
```

### **2. Enable CPU Optimizations**
```bash
export RUSTFLAGS="-C target-cpu=native"
cargo build --release
```

### **3. Reduce Model Size**
```bash
# In your code, use smaller model variants
let config = SentimentConfig {
    model_type: ModelType::Distilbert,
    // ... other config
};
```

### **4. Enable Caching**
```bash
# The API already caches the model in memory
# First request: 10-30 seconds
# Subsequent requests: <1 second
```

## 🐳 **Docker Alternative**

If you prefer Docker (requires QEMU emulation):

```dockerfile
FROM --platform=linux/amd64 rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM --platform=linux/amd64 debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/finbert-rs /usr/local/bin/
EXPOSE 3000
CMD ["finbert-rs"]
```

**Note:** Docker with emulation is slower than native ARM64 builds.

## 📋 **System Requirements**

### **Minimum**
- Raspberry Pi 4 (2GB RAM)
- 8GB SD card
- Active cooling

### **Recommended**
- Raspberry Pi 4 (4GB+ RAM)
- 16GB+ SD card
- Active cooling
- SSD storage (optional)

### **Optimal**
- Raspberry Pi 4 (8GB RAM)
- 32GB+ SD card
- Active cooling
- USB 3.0 SSD

## 🔄 **Maintenance**

### **Update PyTorch**
```bash
pip3 install --upgrade torch torchvision torchaudio
```

### **Update Rust Dependencies**
```bash
cargo update
cargo build --release
```

### **Monitor Performance**
```bash
# Check memory usage
free -h

# Check CPU usage
htop

# Check temperature
vcgencmd measure_temp
```

## 🆘 **Getting Help**

### **If PyTorch Build Fails**
1. Check your Pi model and RAM
2. Ensure adequate cooling
3. Try the pre-built installation first
4. Check PyTorch GitHub issues for ARM64

### **If Rust Build Fails**
1. Verify PyTorch installation
2. Check environment variables
3. Clean and rebuild
4. Check Rust toolchain version

### **If API Runs Slowly**
1. Use release builds
2. Monitor system resources
3. Consider model optimization
4. Check network connectivity

## 🎉 **Success Indicators**

You'll know it's working when:

1. **PyTorch installs successfully:**
   ```bash
   python3 -c "import torch; print('PyTorch OK')"
   ```

2. **Rust builds without errors:**
   ```bash
   cargo build --release
   ```

3. **API starts successfully:**
   ```bash
   cargo run
   # Should show: 🚀 Server running on http://127.0.0.1:3000
   ```

4. **First request works:**
   ```bash
   curl http://localhost:3000/health
   # Should return JSON response
   ```

## 📚 **Additional Resources**

- [PyTorch ARM64 Build Guide](https://github.com/pytorch/pytorch/issues/48865)
- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Raspberry Pi Performance Tuning](https://www.raspberrypi.org/documentation/configuration/performance-tuning.md)

---

**Happy coding on your Raspberry Pi! 🍓🚀**
