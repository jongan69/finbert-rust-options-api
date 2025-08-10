# FinBERT on Raspberry Pi - Complete Guide

This guide helps you run the FinBERT sentiment analysis API on Raspberry Pi's ARM64 architecture.

## 🚨 **Important: Externally Managed Environment Fix**

**NEW**: Modern Debian/Raspberry Pi OS uses "externally managed environments" that prevent system-wide pip installations. We now use Python virtual environments to solve this.

## 🎯 **Quick Start (Recommended)**

### **Option 1: Automatic Fix Script**

```bash
# Download and run the fix script
chmod +x fix-pytorch-pi.sh
./fix-pytorch-pi.sh
```

This script will:
- Install required system packages
- Create a Python virtual environment
- Install ARM64-compatible PyTorch
- Set up environment variables
- Verify the installation

### **Option 2: Manual Installation**

```bash
# 1. Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    python3 \
    python3-pip \
    python3-dev \
    python3-venv \
    python3-full \
    libopenblas-dev \
    liblapack-dev \
    libgomp1 \
    libnuma-dev \
    pkg-config

# 2. Create virtual environment
python3 -m venv ~/pytorch-venv

# 3. Activate virtual environment
source ~/pytorch-venv/bin/activate

# 4. Install PyTorch
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu

# 5. Verify installation
python3 -c "import torch; print(f'PyTorch version: {torch.__version__}')"
```

## 🔧 **Environment Setup**

After installation, the script adds these to your `~/.bashrc`:

```bash
# PyTorch Virtual Environment
export PYTORCH_VENV=~/pytorch-venv
export LIBTORCH=/path/to/pytorch/lib
export LD_LIBRARY_PATH=/path/to/pytorch/lib:$LD_LIBRARY_PATH
alias activate-pytorch='source ~/pytorch-venv/bin/activate'
```

**Reload your environment:**
```bash
source ~/.bashrc
```

## 🏗️ **Building the Rust Project**

```bash
# Clean previous builds
cargo clean

# Build with release optimizations
cargo build --release

# Run the API
cargo run
```

## 📊 **Performance Expectations**

| Component | Time | Notes |
|-----------|------|-------|
| PyTorch Installation | 10-30 min | Depends on internet speed |
| First API Request | 10-30 sec | Model loading |
| Subsequent Requests | 1-3 sec | Cached model |
| Memory Usage | 2-4 GB | During model loading |
| CPU Usage | 50-80% | During inference |

## 🛠️ **Troubleshooting**

### **"externally-managed-environment" Error**

**Solution**: Use the virtual environment approach above. This is the modern way to handle Python packages on Debian systems.

### **Memory Issues**

```bash
# Check available memory
free -h

# If low memory, increase swap
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### **Thermal Throttling**

```bash
# Monitor CPU temperature
vcgencmd measure_temp

# If overheating, add cooling or reduce load
```

### **Installation Failures**

```bash
# Try alternative PyTorch sources
pip install --pre torch torchvision torchaudio --extra-index-url https://download.pytorch.org/whl/nightly/cpu

# Or build from source (slower but more reliable)
chmod +x build-pytorch-pi.sh
./build-pytorch-pi.sh
```

### **Rust Build Errors**

```bash
# Update Rust
rustup update

# Check PyTorch environment
echo $LIBTORCH
echo $LD_LIBRARY_PATH

# Reinstall rust-bert
cargo clean
cargo update
```

## 🔄 **Alternative: Build from Source**

If pre-built packages don't work, build PyTorch from source:

```bash
chmod +x build-pytorch-pi.sh
./build-pytorch-pi.sh
```

**Note**: This takes 2-6 hours but ensures compatibility.

## 🐳 **Docker Alternative**

If you prefer containerization:

```dockerfile
FROM arm64v8/python:3.11-slim

RUN apt-get update && apt-get install -y \
    build-essential \
    libopenblas-dev \
    liblapack-dev \
    && rm -rf /var/lib/apt/lists/*

RUN pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu

# Add your Rust application here
```

## 📋 **System Requirements**

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 4 GB | 8 GB |
| Storage | 8 GB free | 16 GB free |
| CPU | ARM64 (aarch64) | ARM64 with 4+ cores |
| OS | Raspberry Pi OS 11+ | Raspberry Pi OS 12+ |

## 🔍 **Verification Steps**

1. **PyTorch Installation:**
   ```bash
   source ~/pytorch-venv/bin/activate
   python3 -c "import torch; print(torch.__version__)"
   ```

2. **Environment Variables:**
   ```bash
   echo $LIBTORCH
   echo $LD_LIBRARY_PATH
   ```

3. **Rust Build:**
   ```bash
   cargo build --release
   ```

4. **API Test:**
   ```bash
   cargo run &
   curl http://localhost:3000/health
   ```

## 🚀 **Production Deployment**

### **Systemd Service**

```bash
# Install as system service
sudo ./install-service.sh

# Manage service
./manage-service.sh start
./manage-service.sh status
./manage-service.sh logs
```

### **Performance Optimization**

```bash
# Set CPU governor to performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Increase file descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf
```

## 📈 **Monitoring**

```bash
# Check API health
curl http://localhost:3000/health

# View metrics
curl http://localhost:3000/metrics

# Monitor system resources
htop
iotop
```

## 🔧 **Maintenance**

### **Update PyTorch**

```bash
source ~/pytorch-venv/bin/activate
pip install --upgrade torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
```

### **Update Rust Dependencies**

```bash
cargo update
cargo build --release
```

### **Clean Up**

```bash
# Remove old builds
cargo clean

# Clean pip cache
pip cache purge
```

## ✅ **Success Indicators**

- ✅ PyTorch imports without errors
- ✅ `cargo build --release` completes successfully
- ✅ API starts and responds to health checks
- ✅ Sentiment analysis returns results
- ✅ Memory usage stabilizes after model loading

## 🆘 **Getting Help**

If you encounter issues:

1. Check the troubleshooting section above
2. Verify system requirements
3. Ensure virtual environment is activated
4. Check environment variables are set
5. Review logs: `./manage-service.sh logs`

## 🎉 **Next Steps**

Once your API is running:

1. **Configure Alpaca API keys** in `.env`
2. **Test the analysis endpoint**: `curl http://localhost:3000/analyze`
3. **Integrate with your trading bot**
4. **Set up monitoring and alerts**
5. **Optimize for your specific use case**

**Happy coding on your Raspberry Pi! 🍓🚀**
