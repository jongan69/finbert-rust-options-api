# FinBERT Rust Options API on Raspberry Pi - Complete Guide

This guide helps you run the FinBERT sentiment analysis API on Raspberry Pi's ARM64 architecture.

## 🚨 **Important: Externally Managed Environment Fix**

**NEW**: Modern Debian/Raspberry Pi OS uses "externally managed environments" that prevent system-wide pip installations. We now use Python virtual environments to solve this.

## 🚀 **Complete Setup Guide**

### **Option 1: One-Command Installation (Recommended)**

The easiest way to get everything running:

```bash
# Make the script executable and run it
chmod +x install-and-run.sh
./install-and-run.sh
```

**What this does:**
- ✅ Installs all system dependencies
- ✅ Creates Python virtual environment
- ✅ Installs ARM64-compatible PyTorch
- ✅ Downloads FinBERT model
- ✅ Builds the Rust project
- ✅ Sets up environment variables
- ✅ Creates configuration files
- ✅ Starts the API

**Time:** 15-45 minutes (depending on internet speed and Pi model)

### **Option 2: Quick Start (If PyTorch Already Installed)**

If you already have PyTorch set up:

```bash
chmod +x quick-start.sh
./quick-start.sh
```

### **Option 3: Manual Step-by-Step Installation**

If you prefer to understand each step:

#### **Step 1: Install System Dependencies**
```bash
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
    pkg-config \
    build-essential \
    curl \
    git
```

#### **Step 2: Create Python Virtual Environment**
```bash
# Create virtual environment
python3 -m venv ~/pytorch-venv

# Activate it
source ~/pytorch-venv/bin/activate

# Upgrade pip
pip install --upgrade pip
```

#### **Step 3: Install PyTorch**
```bash
# Install ARM64-compatible PyTorch
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu

# Verify installation
python3 -c "import torch; print(f'PyTorch version: {torch.__version__}')"
```

#### **Step 4: Set Up Environment Variables**
```bash
# Find PyTorch library path
PYTORCH_PATH=$(python3 -c "import torch; print(torch.__file__)" | head -1)
PYTORCH_LIB_PATH=$(dirname "$PYTORCH_PATH")/lib

# Set environment variables
export LIBTORCH="$PYTORCH_LIB_PATH"
export LD_LIBRARY_PATH="$PYTORCH_LIB_PATH:$LD_LIBRARY_PATH"

# Add to .bashrc for persistence
echo "" >> ~/.bashrc
echo "# PyTorch Virtual Environment" >> ~/.bashrc
echo "export PYTORCH_VENV=~/pytorch-venv" >> ~/.bashrc
echo "export LIBTORCH=$LIBTORCH" >> ~/.bashrc
echo "export LD_LIBRARY_PATH=$PYTORCH_LIB_PATH:\$LD_LIBRARY_PATH" >> ~/.bashrc
```

#### **Step 5: Download FinBERT Model**
```bash
# Clone the FinBERT model
git clone https://huggingface.co/ProsusAI/finbert
```

#### **Step 6: Build the Rust Project**
```bash
# Clean previous builds
cargo clean

# Build with release optimizations
cargo build --release
```

#### **Step 7: Configure API Keys**
```bash
# Create .env file
cat > .env << EOF
# Alpaca API Configuration
APCA_API_KEY_ID=your_alpaca_api_key_here
APCA_API_SECRET_KEY=your_alpaca_secret_key_here
APCA_BASE_URL=https://api.alpaca.markets

# Logging
RUST_LOG=info
EOF

# Edit with your actual API keys
nano .env
```

#### **Step 8: Run the API**
```bash
# Start the FinBERT API
cargo run --release
```

## 🎯 **After Installation - What's Next?**

### **API is Running!**

Once the installation completes, your FinBERT API will be running at:
- **Main API:** http://localhost:3000
- **Health Check:** http://localhost:3000/health
- **Analysis Endpoint:** http://localhost:3000/analyze

### **Test Your API**

```bash
# Check if API is running
curl http://localhost:3000/health

# Test the analysis endpoint
curl http://localhost:3000/analyze
```

### **Configure API Keys (Required)**

The API needs Alpaca Markets credentials to fetch real-time data:

```bash
# Edit the .env file
nano .env

# Add your actual API keys:
APCA_API_KEY_ID=your_actual_api_key_here
APCA_API_SECRET_KEY=your_actual_secret_key_here
```

**Get API Keys:**
1. Sign up at [Alpaca Markets](https://alpaca.markets/)
2. Go to Paper Trading (free)
3. Copy your API Key and Secret Key
4. Update the `.env` file

### **Restart API After Configuration**

```bash
# Stop the API (Ctrl+C)
# Then restart it
cargo run --release
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

## 🔄 **Daily Usage**

### **Starting the API**

```bash
# Quick start (if everything is set up)
./quick-start.sh

# Or manually
source ~/pytorch-venv/bin/activate
cargo run --release
```

### **Running as a Service (Recommended for Production)**

Install the API as a system service that starts on boot:

```bash
# Install as system service
sudo ./setup-service-for-user.sh

# Start the service
sudo systemctl start finbert-api

# Check status
sudo systemctl status finbert-api

# View logs
sudo journalctl -u finbert-api -f

# Enable to start on boot
sudo systemctl enable finbert-api
```

### **Managing the Service**

```bash
# Use the management script
./manage-service.sh start    # Start service
./manage-service.sh stop     # Stop service
./manage-service.sh status   # Check status
./manage-service.sh logs     # View logs
./manage-service.sh restart  # Restart service
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

### **Installation Issues**

#### **"externally-managed-environment" Error**
**Solution**: Use the virtual environment approach above. This is the modern way to handle Python packages on Debian systems.

#### **PyTorch Installation Fails**
```bash
# Try alternative PyTorch sources
pip install --pre torch torchvision torchaudio --extra-index-url https://download.pytorch.org/whl/nightly/cpu

# Or build from source (slower but more reliable)
# Use the build-pytorch-pi.sh script if available
```

#### **FinBERT Model Download Fails**
```bash
# Manual download
git clone https://huggingface.co/ProsusAI/finbert

# Or download from browser and extract
# https://huggingface.co/ProsusAI/finbert
```

### **Build Issues**

#### **PyTorch Linking Errors (Most Common)**
If you see errors like:
```
/usr/bin/ld: skipping incompatible /path/to/libtorch_cpu.so
/usr/bin/ld: cannot find -ltorch_cpu: No such file or directory
```

**Solution:**
```bash
# Use the fix script
./fix-pytorch-linking.sh

# Or manually:
cargo clean
rm -rf target/release/build/torch-sys-*
source ~/pytorch-venv/bin/activate
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib/')"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"
cargo build --release
```

#### **Missing PyTorch Headers**
If you see:
```
fatal error: torch/torch.h: No such file or directory
```

**Solution:**
```bash
# Reinstall PyTorch with development components
source ~/pytorch-venv/bin/activate
pip uninstall torch torchvision torchaudio -y
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu --force-reinstall
```

#### **Rust Build Errors**
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

### **Runtime Issues**

#### **Memory Issues**
```bash
# Check available memory
free -h

# If low memory, increase swap
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

#### **Thermal Throttling**
```bash
# Monitor CPU temperature
vcgencmd measure_temp

# If overheating, add cooling or reduce load
```

#### **API Won't Start**
```bash
# Check if port is in use
sudo netstat -tlnp | grep :3000

# Kill process if needed
sudo pkill -f finbert-rs

# Check logs
RUST_LOG=debug cargo run
```

#### **Service Won't Start**
```bash
# Check service status
sudo systemctl status finbert-api

# View service logs
sudo journalctl -u finbert-api -f

# Check environment variables
sudo systemctl show finbert-api --property=Environment
```

### **API Issues**

#### **"No API Keys Configured"**
```bash
# Edit .env file
nano .env

# Add your Alpaca API keys:
APCA_API_KEY_ID=your_actual_api_key
APCA_API_SECRET_KEY=your_actual_secret_key
```

#### **API Returns Errors**
```bash
# Check API health
curl http://localhost:3000/health

# Test with verbose logging
RUST_LOG=debug cargo run

# Check Alpaca API connectivity
curl -H "APCA-API-KEY-ID: your_key" -H "APCA-API-SECRET-KEY: your_secret" https://data.alpaca.markets/v1beta1/news
```

### **PyTorch Linking Errors (Most Common)**

If you see errors like:
```
/usr/bin/ld: skipping incompatible /path/to/libtorch_cpu.so
/usr/bin/ld: cannot find -ltorch_cpu: No such file or directory
```

**Solution:**
```bash
# 1. Clean build cache completely
cargo clean
rm -rf target/release/build/torch-sys-*
rm -rf ~/.cargo/registry/cache/*/torch-sys*

# 2. Activate virtual environment
source ~/pytorch-venv/bin/activate

# 3. Set environment variables correctly
export LIBTORCH="$(python3 -c "import torch; print(torch.__file__)" | head -1 | sed 's/__init__.py/lib/')"
export LD_LIBRARY_PATH="$LIBTORCH:$LD_LIBRARY_PATH"

# 4. Verify ARM64 libraries
echo "🔍 Checking library architecture:"
file "$LIBTORCH"/libtorch*.so

# 5. Build with correct environment
cargo build --release
```

**Expected Output:**
```
/home/jonathangan/pytorch-venv/lib/python3.11/site-packages/torch/lib/libtorch_cpu.so: ELF 64-bit LSB shared object, ARM aarch64
```

### **Quick Fix Script**

The `fix-pytorch-linking.sh` script is already included in the repository:

```bash
chmod +x fix-pytorch-linking.sh
./fix-pytorch-linking.sh
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

## 📋 **Quick Reference**

### **Essential Commands**
```bash
# Complete installation
./install-and-run.sh

# Quick start (if PyTorch exists)
./quick-start.sh

# Fix PyTorch linking issues
./fix-pytorch-linking.sh

# Install as service
sudo ./setup-service-for-user.sh

# Manage service
./manage-service.sh start|stop|status|logs|restart
```

### **Key Files**
- `install-and-run.sh` - Complete installation script
- `quick-start.sh` - Fast startup script
- `fix-pytorch-linking.sh` - Fix PyTorch linking issues
- `setup-service-for-user.sh` - Install as system service
- `manage-service.sh` - Service management
- `finbert-api.service` - Systemd service configuration
- `.env` - API configuration (add your Alpaca keys)
- `finbert/` - FinBERT model directory

### **API Endpoints**
- **Health:** http://localhost:3000/health
- **Analysis:** http://localhost:3000/analyze
- **Metrics:** http://localhost:3000/metrics

### **Environment Variables**
```bash
export PYTORCH_VENV=~/pytorch-venv
export LIBTORCH=~/pytorch-venv/lib/python3.11/site-packages/torch/lib
export LD_LIBRARY_PATH=$LIBTORCH:$LD_LIBRARY_PATH
```

## 🎉 **Success Checklist**

- ✅ PyTorch installed and verified
- ✅ FinBERT model downloaded
- ✅ Rust project builds successfully
- ✅ API starts without errors
- ✅ Health endpoint responds
- ✅ Alpaca API keys configured
- ✅ Analysis endpoint returns data
- ✅ Service installed (optional)

**Happy coding on your Raspberry Pi! 🍓🚀**
