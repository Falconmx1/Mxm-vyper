# Mxm-vyper ⚡

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-GPL%203.0-red)](LICENSE)

**Async brute-force auditor - 10x faster than Hydra**

## Installation

```bash
git clone https://github.com/Falconmx1/Mxm-vyper.git
cd Mxm-vyper
cargo build --release
sudo cp target/release/mxm_vyper /usr/local/bin/

Usage
# SSH brute-force with 500 concurrent threads
mxm_vyper -t 192.168.1.100 -p ssh -u admin -w rockyou.txt --threads 500

# With proxy rotation
mxm_vyper -t example.com -p ssh -u root -w passwords.txt --proxy socks5://localhost:9050

# Timeout control
mxm_vyper -t 10.0.0.1 -p ssh -u user -w wordlist.txt --timeout 3
