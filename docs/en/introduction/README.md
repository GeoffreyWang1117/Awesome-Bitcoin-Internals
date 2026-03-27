# SimpleBTC - Bitcoin-Based Banking System Demo

Welcome to the SimpleBTC project documentation!

## Project Introduction

SimpleBTC is a Bitcoin banking system demo implemented in Rust, providing a complete implementation of Bitcoin's core principles and advanced features. This project is not only a learning tool but also a fully functional blockchain system demo.

### Core Features

#### 🔐 Complete UTXO Model
- Unspent Transaction Output (UTXO) management
- Double-spend prevention mechanism
- Balance calculation
- UTXO selection algorithm

#### ⛓️ Blockchain Core Functionality
- Proof of Work (PoW)
- Blockchain validation
- Merkle tree implementation
- Chain hash structure

#### 💼 Wallet System
- Key pair generation (simplified)
- Address generation
- Digital signatures
- Transaction creation

#### 📊 Advanced Transaction Features
- **Replace-By-Fee (RBF)**: Replace unconfirmed transactions to speed up confirmation
- **Time Lock (TimeLock)**: Term deposits, inheritance planning
- **Multi-Signature (MultiSig)**: 2-of-3 enterprise wallets, escrow services
- **Transaction Priority**: Priority sorting based on fees

#### 🌳 Merkle Tree and SPV
- Efficient transaction verification
- Lightweight client support
- Merkle proof generation and verification

#### 🔧 Engineering Features
- REST API server (Axum framework)
- Persistent storage (JSON)
- Transaction indexer
- Electron visualization interface

### Why Choose SimpleBTC?

1. **Educational Value**
   - Deep understanding of Bitcoin principles
   - Learn Rust blockchain development
   - Master cryptographic fundamentals

2. **Complete Implementation**
   - Conforms to ACID transaction properties
   - Implements Bitcoin core protocol
   - Includes advanced BIP features

3. **Practical Examples**
   - Corporate fund management
   - Escrow transaction services
   - Term deposit systems

4. **Easy to Extend**
   - Modular design
   - Clear code structure
   - Detailed comments

## Quick Start

```bash
# Clone the project
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC

# Build the project
cargo build --release

# Run the demo
cargo run --bin btc-demo

# Run the REST API server
cargo run --bin btc-server

# Run examples
cargo run --example enterprise_multisig
cargo run --example escrow_service
cargo run --example timelock_savings
```

## System Architecture

```
SimpleBTC/
├── src/
│   ├── transaction.rs     # Transaction module (UTXO model)
│   ├── block.rs          # Block structure
│   ├── blockchain.rs     # Blockchain core logic
│   ├── wallet.rs         # Wallet management
│   ├── utxo.rs          # UTXO set management
│   ├── merkle.rs        # Merkle tree implementation
│   ├── multisig.rs      # Multi-signature
│   ├── advanced_tx.rs   # RBF, time locks, priority
│   ├── persistence.rs   # Persistent storage
│   └── indexer.rs       # Transaction indexer
├── examples/            # Practical examples
├── frontend/            # Electron GUI
└── docs/               # This documentation
```

## Tech Stack

- **Language**: Rust (Edition 2021)
- **Core Libraries**:
  - sha2 - SHA256 hashing
  - serde - Serialization
  - rand - Random number generation
- **Web Framework**: Axum (async REST API)
- **Frontend**: Electron + JavaScript
- **Documentation**: mdBook

## Learning Path

### Beginner: Understanding Basic Concepts
1. [Basic Concepts](../guide/concepts.md) - UTXO, blocks, hashing
2. [Wallet Management](../guide/wallet.md) - Creating wallets, sending transactions
3. [Transaction Processing](../guide/transactions.md) - Transaction structure, validation

### Intermediate: Mastering Core Mechanisms
1. [Blockchain Operations](../guide/blockchain.md) - Mining, validation
2. [UTXO Management](../guide/utxo.md) - UTXO selection, double-spend protection
3. [Merkle Tree](../advanced/merkle.md) - SPV verification

### Advanced: Implementing Complex Applications
1. [Multi-Signature](../advanced/multisig.md) - Enterprise wallets
2. [Time Lock](../advanced/timelock.md) - Term deposits
3. [RBF Mechanism](../advanced/rbf.md) - Transaction acceleration

## Differences from Bitcoin

SimpleBTC is a simplified educational implementation; the main differences from real Bitcoin are:

| Feature | SimpleBTC | Real Bitcoin |
|---------|-----------|--------------|
| Cryptography | Simplified SHA256 | secp256k1 elliptic curve |
| Signatures | Simplified validation | ECDSA signatures |
| Scripts | Simplified scripts | Full Script language |
| P2P Network | No network layer | Full P2P protocol |
| Storage | JSON files | LevelDB database |
| Difficulty adjustment | Fixed difficulty | Dynamic difficulty adjustment |

## Project Status

- ✅ UTXO model
- ✅ Proof of Work
- ✅ Merkle tree
- ✅ Multi-signature
- ✅ RBF mechanism
- ✅ Time lock
- ✅ REST API
- ✅ GUI interface
- ✅ Full documentation

## Contributing

Contributions of code, documentation, or bug reports are welcome!

See the [Contributing Guide](../appendix/contributing.md) for details.

## License

This project is licensed under the MIT License.

---

Let's start exploring the world of Bitcoin! 🚀
