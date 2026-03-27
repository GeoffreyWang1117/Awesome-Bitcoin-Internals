# Installation and Configuration

This section will guide you through the installation and basic configuration of SimpleBTC.

## System Requirements

### Minimum Requirements
- **Operating System**: Linux, macOS, or Windows (WSL2)
- **Rust Version**: 1.70.0 or higher
- **Memory**: At least 2GB RAM
- **Storage**: At least 500MB free space

### Recommended Configuration
- **Operating System**: Linux/macOS
- **Rust Version**: Latest stable release
- **Memory**: 4GB+ RAM
- **Storage**: 1GB+ free space
- **CPU**: Multi-core processor (better mining performance)

## Installing Rust

If you have not installed Rust yet, visit [rust-lang.org](https://www.rust-lang.org/) or use the following command:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

## Cloning the Project

```bash
# Clone via HTTPS
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git

# Or clone via SSH
git clone git@github.com:GeoffreyWang1117/SimpleBTC.git

# Enter the project directory
cd SimpleBTC
```

## Building the Project

### Development Build

```bash
# Fast build (unoptimized, compiles quickly)
cargo build

# Run tests
cargo test

# Run the Demo
cargo run --bin btc-demo
```

### Production Build

```bash
# Optimized build (best performance, slower to compile)
cargo build --release

# Run the optimized binary
./target/release/btc-demo
./target/release/btc-server
```

## Running the Examples

SimpleBTC provides three hands-on examples:

```bash
# 1. Enterprise multi-signature wallet (2-of-3)
cargo run --example enterprise_multisig

# 2. Escrow service (buyer/seller/arbitrator)
cargo run --example escrow_service

# 3. Time-deposit savings (time lock)
cargo run --example timelock_savings
```

## Starting the REST API Server

```bash
# Development mode
cargo run --bin btc-server

# Production mode
cargo run --release --bin btc-server
```

The server will start at `http://localhost:3000`

### API Endpoints

- `GET /api/blockchain/info` - Get blockchain information
- `POST /api/wallet/create` - Create a new wallet
- `POST /api/transaction/create` - Create a transaction
- `POST /api/mine` - Mine a block
- `GET /api/balance/:address` - Query balance

## Starting the Electron GUI

```bash
# Install Node.js dependencies
cd frontend
npm install

# Launch the Electron application
npm start
```

The GUI provides a visual interface including:
- Blockchain explorer
- Wallet management
- Transaction creation
- Real-time mining
- One-click Demo mode

## Project Structure

```
SimpleBTC/
├── src/                    # Source code
│   ├── lib.rs             # Library entry point
│   ├── main.rs            # CLI Demo
│   ├── transaction.rs     # Transaction module
│   ├── block.rs           # Block module
│   ├── blockchain.rs      # Blockchain logic
│   ├── wallet.rs          # Wallet management
│   ├── utxo.rs           # UTXO management
│   ├── merkle.rs         # Merkle tree
│   ├── multisig.rs       # Multi-signature
│   ├── advanced_tx.rs    # Advanced transaction features
│   ├── persistence.rs    # Persistence
│   └── indexer.rs        # Indexer
├── examples/              # Example programs
│   ├── enterprise_multisig.rs
│   ├── escrow_service.rs
│   └── timelock_savings.rs
├── frontend/              # Electron GUI
│   ├── main.js
│   ├── app.js
│   └── index.html
├── docs/                  # Documentation
├── Cargo.toml            # Rust project configuration
└── README.md             # Project description
```

## Configuration Options

### Mining Difficulty

Modify in `src/blockchain.rs`:

```rust
pub fn new() -> Blockchain {
    let mut blockchain = Blockchain {
        difficulty: 3,  // Change here: 3-5 is suitable for demos, 6+ is more secure but slower
        // ...
    }
}
```

### Block Reward

```rust
pub fn new() -> Blockchain {
    let mut blockchain = Blockchain {
        mining_reward: 50,  // Modify the mining reward (satoshi)
        // ...
    }
}
```

### API Server Port

Modify in `src/bin/server.rs`:

```rust
let listener = TcpListener::bind("0.0.0.0:3000") // Change port here
    .await
    .unwrap();
```

## Frequently Asked Questions

### Build Errors

**Problem**: `error: failed to fetch`
```bash
# Solution: Update the Cargo index
cargo update
```

**Problem**: `error: linker 'cc' not found`
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS (install Xcode command-line tools)
xcode-select --install
```

### Runtime Errors

**Problem**: `Address already in use (os error 98)`
```bash
# Port 3000 is occupied; kill the process using it or change the port
lsof -ti:3000 | xargs kill
```

**Problem**: Mining is too slow
```bash
# Lower the difficulty
# Set difficulty: 2 in blockchain.rs
```

## Next Steps

- 📖 Read [Quick Start](./quickstart.md) for basic usage
- 🎓 Study [Core Concepts](./concepts.md) to understand the principles
- 🔨 Browse [Practical Examples](../examples/enterprise-multisig.md)

## Getting Help

- GitHub Issues: https://github.com/GeoffreyWang1117/SimpleBTC/issues
- Project Documentation: this site
- Rust Community: https://users.rust-lang.org/
