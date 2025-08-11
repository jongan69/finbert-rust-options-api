# 📜 Scripts Overview - FinBERT Rust Options API

This project includes several scripts to help with installation, setup, and troubleshooting on Raspberry Pi and other platforms.

## 🚀 Main Scripts

### `install-and-run.sh` ⭐ **RECOMMENDED**
**One-command solution for everything**

```bash
./install-and-run.sh
```

**Features:**
- ✅ Complete setup and run in one command
- ✅ Automatic PyTorch compatibility fixes
- ✅ Memory-aware build optimization
- ✅ Multiple usage modes
- ✅ Comprehensive error handling

**Usage modes:**
- `./install-and-run.sh` - Full setup and run
- `./install-and-run.sh --setup-only` - Setup only
- `./install-and-run.sh --build-only` - Build only
- `./install-and-run.sh --run-only` - Run only
- `./install-and-run.sh --clean` - Clean build

## 🔧 Specialized Scripts

### `fix-pytorch-linking.sh`
**PyTorch compatibility fix for ARM64**

```bash
./fix-pytorch-linking.sh
```

**Use when:**
- You have PyTorch 2.8.0+ installed
- Getting torch-sys compilation errors
- Need to downgrade to PyTorch 2.1.2

### `fix-pytorch-pi-comprehensive.sh`
**Interactive PyTorch fix with multiple solutions**

```bash
./fix-pytorch-pi-comprehensive.sh
```

**Use when:**
- You want to choose between downgrading PyTorch or upgrading rust-bert
- Need interactive troubleshooting
- Want automatic fallback options

### `quick-start.sh`
**Quick setup for development**

```bash
./quick-start.sh
```

**Use when:**
- You're on a development machine
- Don't need full production setup
- Want faster setup process

## 📋 Service Management Scripts

### `manage-service.sh`
**Systemd service management**

```bash
./manage-service.sh install    # Install as system service
./manage-service.sh start      # Start the service
./manage-service.sh stop       # Stop the service
./manage-service.sh status     # Check service status
./manage-service.sh uninstall  # Remove service
```

### `setup-service-for-user.sh`
**Setup service for specific user**

```bash
./setup-service-for-user.sh username
```

## 🍓 Raspberry Pi Specific Scripts

### `build-pytorch-pi.sh`
**Build PyTorch from source for Pi**

```bash
./build-pytorch-pi.sh
```

**Use when:**
- Pre-built PyTorch wheels don't work
- Need custom PyTorch build
- Have specific hardware requirements

## 📊 Script Comparison

| Script | Purpose | Best For | Complexity |
|--------|---------|----------|------------|
| `install-and-run.sh` | Complete solution | **Most users** | Low |
| `fix-pytorch-linking.sh` | PyTorch compatibility | ARM64 issues | Medium |
| `fix-pytorch-pi-comprehensive.sh` | Interactive fixes | Troubleshooting | High |
| `quick-start.sh` | Fast development setup | Developers | Low |
| `manage-service.sh` | Service management | Production | Medium |
| `build-pytorch-pi.sh` | Source build | Advanced users | High |

## 🎯 Recommended Workflow

### For New Users
1. **Start with:** `./install-and-run.sh`
2. **If issues:** Check `RASPBERRY_PI_TROUBLESHOOTING.md`
3. **For production:** Use `manage-service.sh`

### For Developers
1. **Quick setup:** `./quick-start.sh`
2. **Full setup:** `./install-and-run.sh --setup-only`
3. **Run:** `./install-and-run.sh --run-only`

### For Production
1. **Setup:** `./install-and-run.sh --setup-only`
2. **Build:** `./install-and-run.sh --build-only`
3. **Install service:** `./manage-service.sh install`
4. **Start service:** `./manage-service.sh start`

## 🔍 Troubleshooting Flow

```
Problem → Solution
├── PyTorch linking errors → fix-pytorch-linking.sh
├── Interactive fixes needed → fix-pytorch-pi-comprehensive.sh
├── Service issues → manage-service.sh
├── Build problems → install-and-run.sh --clean
└── General issues → RASPBERRY_PI_TROUBLESHOOTING.md
```

## 📝 Script Dependencies

### Required for all scripts:
- Bash shell
- Git
- Internet connection

### Required for main scripts:
- Rust/Cargo
- Python 3.8+
- 4GB+ RAM (recommended)

### Required for service scripts:
- systemd (Linux)
- sudo privileges

## 🛠️ Customization

All scripts can be customized by editing the configuration variables at the top of each script:

```bash
# Example customization in install-and-run.sh
PYTORCH_VERSION="2.1.2"
VENV_PATH="$HOME/pytorch-venv"
PROJECT_NAME="finbert-rs"
```

## 📞 Getting Help

1. **Check script help:** `./script-name.sh --help`
2. **Read documentation:** See individual script comments
3. **Check troubleshooting:** `RASPBERRY_PI_TROUBLESHOOTING.md`
4. **Quick start:** `QUICK_START.md`

---

**Choose the right tool for your needs! 🛠️**
