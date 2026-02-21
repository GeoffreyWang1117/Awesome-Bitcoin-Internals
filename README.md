# SimpleBTC

<div align="center">

**A complete Bitcoin blockchain implementation for education and learning.**

Built with Rust | Web UI Included | Real Bitcoin Features

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)
[![CI](https://github.com/GeoffreyWang1117/SimpleBTC/actions/workflows/ci.yml/badge.svg)](https://github.com/GeoffreyWang1117/SimpleBTC/actions)

[English](#features) | [中文](#中文说明)

</div>

---

## Quick Start

```bash
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC
cargo run --bin btc-server --release
```

Open **http://localhost:3000** in your browser. Click "Run Demo" to see it in action.

## Features

### Core Blockchain
- **UTXO Model** — Bitcoin's native unspent transaction output model with double-spend prevention
- **Proof of Work** — SHA256-based mining with configurable difficulty
- **Merkle Trees** — Efficient transaction verification with SPV proof support
- **Transaction System** — Full lifecycle: create, sign, validate, confirm

### Advanced Bitcoin Features
- **Multi-Signature (M-of-N)** — 2-of-3, 3-of-5, arbitrary M-of-N wallets
- **Replace-By-Fee (RBF)** — BIP125-style transaction replacement
- **TimeLock** — Time-based and block-height-based locks
- **Bitcoin Script** — Simplified script execution engine
- **SPV Client** — Simplified Payment Verification light client
- **Mempool** — Transaction pool with fee-rate priority indexing

### Infrastructure
- **Real Cryptography** — secp256k1 ECDSA (same curve as Bitcoin)
- **RocksDB Storage** — High-performance persistent storage
- **Parallel Mining** — Multi-threaded PoW using Rayon
- **REST API** — Full HTTP API for all operations
- **Web UI** — Built-in browser interface, no extra dependencies
- **CI/CD** — GitHub Actions for testing on Linux/macOS/Windows

## Architecture

```
src/
├── lib.rs              # Library entry — exports all modules
├── main.rs             # CLI demo (btc-demo)
├── bin/server.rs       # API server + Web UI (btc-server)
│
├── block.rs            # Block structure, PoW
├── blockchain.rs       # Chain logic, validation
├── transaction.rs      # Transactions (inputs/outputs)
├── wallet.rs           # Key management, addresses
├── utxo.rs             # UTXO set tracking
├── crypto.rs           # Real secp256k1 ECDSA
│
├── merkle.rs           # Merkle tree + SPV proofs
├── multisig.rs         # Multi-signature wallets
├── advanced_tx.rs      # RBF + TimeLock
├── mempool.rs          # Memory pool
├── script.rs           # Bitcoin Script engine
├── spv.rs              # SPV light client
├── parallel_mining.rs  # Multi-threaded mining
│
├── error.rs            # Error types
├── config.rs           # Configuration
├── persistence.rs      # Data persistence
├── storage.rs          # RocksDB layer
├── indexer.rs          # Transaction indexing
├── logging.rs          # Tracing system
└── security.rs         # Security validation

static/
└── index.html          # Web UI (self-contained)

examples/
├── enterprise_multisig.rs   # Corporate 2-of-3 multisig
├── escrow_service.rs        # Trustless escrow
└── timelock_savings.rs      # Time-locked savings

docs/                   # Full documentation (mdBook)
```

## Usage

### Web UI (Recommended)

```bash
cargo run --bin btc-server --release
# Open http://localhost:3000
```

### CLI Demo

```bash
cargo run --bin btc-demo --release
```

### REST API

```bash
# Start server
cargo run --bin btc-server --release

# Create wallet
curl -X POST http://localhost:3000/api/wallet/create

# Query balance
curl http://localhost:3000/api/wallet/balance/ADDRESS

# Send transaction
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{"from_address":"ADDR1","to_address":"ADDR2","amount":50,"fee":5}'

# Mine block
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address":"ADDRESS"}'
```

### Examples

```bash
cargo run --example enterprise_multisig
cargo run --example escrow_service
cargo run --example timelock_savings
```

## Bitcoin Comparison

| Feature | SimpleBTC | Real Bitcoin |
|---------|-----------|-------------|
| UTXO Model | Yes | Yes |
| Proof of Work (SHA256) | Yes | Yes (Double SHA256) |
| Merkle Trees | Yes | Yes |
| Multi-Signature | Yes (M-of-N) | Yes (P2SH/P2WSH) |
| RBF | Yes | Yes (BIP125) |
| TimeLock | Yes | Yes (CLTV/CSV) |
| ECDSA Signatures | Yes (secp256k1) | Yes (secp256k1/Schnorr) |
| Script System | Simplified | Full Script |
| P2P Network | No | Yes |
| Consensus | Single node | Distributed |

## Development

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code
cargo clippy --all-targets --all-features

# Format code
cargo fmt

# Build documentation
cd docs && mdbook serve --open
```

## Contributing

Contributions welcome! Some ideas:

- P2P networking layer
- SegWit support
- Lightning Network (Layer 2)
- More Script opcodes
- English documentation translation

See [CONTRIBUTING](./docs/src/appendix/contributing.md) for guidelines.

## License

[MIT](./LICENSE)

---

## 中文说明

SimpleBTC 是一个完整的比特币区块链教育实现，使用 Rust 构建。

### 核心特性

- **UTXO模型** — 比特币原生的未花费交易输出模型，防双花
- **工作量证明** — SHA256挖矿，可配置难度
- **Merkle树** — 高效交易验证，支持SPV证明
- **多重签名** — M-of-N多签钱包（企业级安全）
- **RBF** — BIP125交易替换（加速确认）
- **时间锁** — 基于时间/区块高度的锁定
- **Bitcoin脚本** — 简化版脚本执行引擎
- **SPV轻客户端** — 简化支付验证
- **内存池** — 手续费优先级排序的交易池
- **真实密码学** — secp256k1 ECDSA签名
- **Web界面** — 内置浏览器界面，无需额外依赖

### 快速开始

```bash
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC
cargo run --bin btc-server --release
# 打开浏览器访问 http://localhost:3000
```

### 学习路径

1. **入门**: 运行 `btc-demo`，阅读 `docs/src/guide/` 目录
2. **进阶**: 学习高级特性（多签、RBF、TimeLock），研究 `examples/`
3. **深入**: 阅读核心源码 `blockchain.rs`、`transaction.rs`、`block.rs`

### 完整文档

```bash
cargo install mdbook
cd docs && mdbook serve --open
```

15,000+ 字系统化中文文档，涵盖从入门到精通的所有内容。

---

<div align="center">

**SimpleBTC** — Learn Bitcoin by building it.

</div>
