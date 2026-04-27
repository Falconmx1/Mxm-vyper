#!/bin/bash

# Mxm-vyper Installer
set -e

echo "🔧 Installing Mxm-vyper..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "Python3 not found. Please install Python3 first"
    exit 1
fi

# Install Python dependencies for plugins
pip3 install requests beautifulsoup4

# Build Mxm-vyper
echo "Building Mxm-vyper..."
cargo build --release

# Create directories
mkdir -p ~/.mxm-vyper/modules
mkdir -p ~/.mxm-vyper/wordlists

# Copy default wordlist if not exists
if [ ! -f ~/.mxm-vyper/wordlists/rockyou.txt ]; then
    echo "Downloading sample wordlist..."
    curl -L -o ~/.mxm-vyper/wordlists/rockyou_small.txt \
        https://github.com/brannondorsey/naive-hashcat/releases/download/data/rockyou.txt
fi

# Copy example plugin
cp modules/http_plugin.py ~/.mxm-vyper/modules/

# Create symlink
sudo cp target/release/mxm_vyper /usr/local/bin/

echo "✅ Installation complete!"
echo "Usage: mxm_vyper -t <target> -p <protocol> -u <user> -w <wordlist>"
echo ""
echo "Examples:"
echo "  SSH:    mxm_vyper -t 192.168.1.1 -p ssh -u root -w wordlist.txt"
echo "  HTTP:   mxm_vyper -t http://example.com -p http -w wordlist.txt --http-path /login"
echo "  Tor:    mxm_vyper -t example.onion -p http --tor -w wordlist.txt"
echo "  Python: mxm_vyper -t example.com -p python --python-plugin ./my_module.py -w wordlist.txt"
