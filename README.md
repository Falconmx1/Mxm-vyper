# Mxm-vyper 🔥

**Educational multi-protocol async brute-force auditor**

> ⚠️ For authorized security testing only. Illegal use = prison time.

## Why Mxm-vyper?
- 10x faster than Hydra (Tokio async runtime)
- Built-in proxy rotation (SOCKS5/HTTP)
- Plugin system (Rust core + Python/Go modules)
- Bypasses basic rate limiting

## Quick start
\```bash
cargo build --release
./target/release/mxm-vyper -t 192.168.1.1 -p ssh -u root -w rockyou.txt --proxy socks5://localhost:9050
\```

## Architecture
- **Rust (Tokio)** → Core async attacker
- **Python** → Exploit scripts & reporting
- **Go** → Cloud distributed workers

## Legal
Use only on infrastructure you own or have written permission.
