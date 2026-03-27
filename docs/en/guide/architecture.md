# Architecture Overview

SimpleBTC is a fully-featured educational implementation of the Bitcoin blockchain, written in Rust. This chapter introduces the project's overall architectural design, module organization, core data flows, and key abstraction layers.

---

## Project Structure

```
SimpleBTC/
├── src/
│   ├── lib.rs               # Crate entry point, module exports and re-exports
│   ├── main.rs              # Executable entry point (demo / interactive CLI)
│   │
│   │── ── Core Layer ──
│   ├── block.rs             # Block structure + Proof of Work
│   ├── blockchain.rs        # Core blockchain logic (UTXO management, mining, validation)
│   ├── transaction.rs       # Transaction structures (TxInput / TxOutput / Transaction)
│   ├── wallet.rs            # Wallet + secp256k1 key management
│   ├── utxo.rs              # UTXO set (unspent transaction output management)
│   ├── crypto.rs            # Extended cryptography (Bech32, WIF export)
│   │
│   │── ── Advanced Feature Layer ──
│   ├── merkle.rs            # Merkle tree (transaction inclusion proofs)
│   ├── multisig.rs          # Multi-signature (M-of-N)
│   ├── advanced_tx.rs       # Advanced transactions (RBF, TimeLock)
│   ├── mempool.rs           # Memory pool (sorted by fee rate)
│   ├── script.rs            # Bitcoin Script system
│   ├── spv.rs               # SPV lightweight client verification
│   ├── parallel_mining.rs   # Multi-threaded parallel PoW mining
│   ├── network.rs           # P2P network layer
│   │
│   │── ── Infrastructure Layer ──
│   ├── storage.rs           # RocksDB high-performance persistent storage
│   ├── persistence.rs       # Serialization / deserialization helpers
│   ├── config.rs            # Global configuration (difficulty, rewards, etc.)
│   ├── logging.rs           # Structured logging (tracing)
│   ├── security.rs          # Security validation
│   ├── indexer.rs           # Transaction indexer (accelerates address queries)
│   └── error.rs             # Unified error types
│
├── docs/                    # mdBook documentation
├── Cargo.toml
└── README.md
```

---

## Three-Layer Architecture Model

SimpleBTC's modules are organized into three responsibility layers:

```
┌─────────────────────────────────────────────────────────┐
│                     Core Layer                           │
│  block  blockchain  transaction  wallet  utxo  crypto   │
│  ─── Implements Bitcoin's fundamental data structures    │
│      and protocol rules ───                             │
├─────────────────────────────────────────────────────────┤
│                 Advanced Feature Layer                   │
│  merkle  multisig  advanced_tx  mempool  script  spv    │
│  parallel_mining  network                               │
│  ─── Implements Bitcoin's advanced features and         │
│      extended protocols ───                             │
├─────────────────────────────────────────────────────────┤
│                  Infrastructure Layer                    │
│  storage  persistence  config  logging  security        │
│  indexer  error                                         │
│  ─── Provides general-purpose capabilities: storage,    │
│      logging, configuration, etc. ───                   │
└─────────────────────────────────────────────────────────┘
```

### Core Layer Module Details

| Module | File | Responsibility |
|--------|------|----------------|
| `block` | `block.rs` | Defines the `Block` struct, containing block header fields (index, timestamp, nonce, merkle_root, previous_hash, hash) and the single-threaded `mine_block()` method |
| `blockchain` | `blockchain.rs` | The `Blockchain` main struct, coordinating all blockchain operations: genesis block, transaction creation, mempool management, parallel mining, UTXO updates, and chain validation |
| `transaction` | `transaction.rs` | Three core structs: `TxInput`, `TxOutput`, `Transaction`; Coinbase transaction construction; ECDSA signature verification |
| `wallet` | `wallet.rs` | `Wallet` struct, uses secp256k1 to generate real key pairs, P2PKH address derivation, ECDSA signing and verification |
| `utxo` | `utxo.rs` | `UTXOSet` manages all unspent transaction outputs, supports balance queries and spendable UTXO retrieval |
| `crypto` | `crypto.rs` | `CryptoWallet` extended implementation: Bech32 addresses, WIF private key format import/export |

### Advanced Feature Layer Module Details

| Module | File | Responsibility |
|--------|------|----------------|
| `merkle` | `merkle.rs` | Merkle tree construction and Merkle proof generation/verification (foundation for SPV) |
| `multisig` | `multisig.rs` | M-of-N multi-signature scheme |
| `advanced_tx` | `advanced_tx.rs` | RBF (Replace-By-Fee) fee replacement, TimeLock time-locked transactions |
| `mempool` | `mempool.rs` | Memory pool, sorts pending transactions by fee rate (satoshi/byte) |
| `script` | `script.rs` | Bitcoin Script opcode interpreter |
| `spv` | `spv.rs` | Simple Payment Verification — validates transactions using Merkle proofs without downloading the full chain |
| `parallel_mining` | `parallel_mining.rs` | `ParallelMiner`: multi-threaded PoW that fully utilizes multi-core CPUs |
| `network` | `network.rs` | P2P network message propagation layer |

### Infrastructure Layer Module Details

| Module | File | Responsibility |
|--------|------|----------------|
| `storage` | `storage.rs` | High-performance key-value storage based on RocksDB |
| `persistence` | `persistence.rs` | Blockchain data serialization and deserialization |
| `config` | `config.rs` | Global parameters (mining difficulty, block reward, network parameters, etc.) |
| `logging` | `logging.rs` | Structured logging (based on the `tracing` crate) |
| `security` | `security.rs` | Additional security validation logic |
| `indexer` | `indexer.rs` | `TransactionIndexer`: builds an address → transaction ID index to accelerate balance queries |
| `error` | `error.rs` | `BitcoinError` unified error enum, `Result<T>` type alias |

---

## Core Data Flow

### Complete Value Transfer Flow

```
User initiates a transfer request
       │
       ▼
┌─────────────────────────────────────┐
│  Blockchain::create_transaction()   │
│  1. Find spendable outputs in UTXOSet│
│  2. Sign inputs with Wallet::sign() │
│  3. Construct TxInput + TxOutput    │
│  4. Generate Transaction (with hash ID)│
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Blockchain::add_transaction()      │
│  1. Transaction::verify() — verify signatures│
│  2. Check UTXO exists + balance sufficient│
│  3. Record pending_spent to prevent double-spend│
│  4. Add to Mempool (sorted by fee rate)│
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Blockchain::mine_pending_transactions()│
│  1. Retrieve high-fee-rate transactions from Mempool│
│  2. Construct Coinbase transaction (reward + fees)│
│  3. Block::new() computes Merkle Root│
│  4. ParallelMiner multi-threaded PoW│
│  5. Validate all transactions in the block│
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Atomic UTXO set update             │
│  1. Consume UTXOs referenced by inputs│
│  2. Add outputs as new UTXOs        │
│  3. Clear pending_spent             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Block appended to chain            │
│  1. indexer.index_block() — build index│
│  2. chain.push(block)               │
│  3. Remove confirmed transactions from Mempool│
└─────────────────────────────────────┘
```

### Data Structure Relationship Diagram

```
Blockchain
├── chain: Vec<Block>
│   └── Block
│       ├── index, timestamp, nonce
│       ├── previous_hash → previous block's hash (chain linkage)
│       ├── merkle_root   → computed by MerkleTree
│       ├── hash          → SHA256(index+timestamp+merkle_root+prev+nonce)
│       └── transactions: Vec<Transaction>
│           └── Transaction
│               ├── id      → SHA256(transaction content)
│               ├── inputs: Vec<TxInput>
│               │   └── TxInput {txid, vout, signature, pub_key}
│               └── outputs: Vec<TxOutput>
│                   └── TxOutput {value, pub_key_hash}
│
├── utxo_set: UTXOSet   ← fast balance query and UTXO retrieval
├── mempool: Mempool    ← pending transactions (sorted by fee rate)
├── indexer: TransactionIndexer  ← address → transaction index
└── miner: ParallelMiner         ← multi-threaded PoW
```

---

## Quick Start

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

// 1. Create a blockchain (includes genesis block; genesis wallet receives 10M satoshi initial funds)
let mut blockchain = Blockchain::new();

// 2. Get the pre-funded genesis wallet + create new user wallets
let genesis = Blockchain::genesis_wallet();
let alice = Wallet::new();
let bob = Wallet::new();

// 3. Create transaction: genesis → alice, transfer 1000 satoshi, fee 10
let tx = blockchain.create_transaction(&genesis, alice.address.clone(), 1000, 10)?;
blockchain.add_transaction(tx)?;

// 4. Mine (alice receives the block reward as miner)
blockchain.mine_pending_transactions(alice.address.clone())?;

// 5. Query balance
println!("Alice's balance: {} satoshi", blockchain.get_balance(&alice.address));

// 6. Validate the entire chain's integrity
assert!(blockchain.is_valid());
# Ok::<(), String>(())
```

---

## Cryptography Choices

| Algorithm | Library | Purpose |
|-----------|---------|---------|
| secp256k1 ECDSA | `secp256k1` crate | Private key generation, transaction signing, signature verification |
| SHA-256 | `bitcoin_hashes` | Block hash, transaction hash, address derivation |
| RIPEMD-160 | `ripemd` crate | Public key hash (intermediate step in P2PKH address) |
| Base58Check | `bs58` crate | P2PKH address encoding, WIF private key encoding |
| Bech32 | `bech32` crate | Native SegWit addresses |
| SHA-256d | `bitcoin_hashes` | Double hash (checksum computation) |

All cryptographic implementations are compatible with the Bitcoin mainnet — addresses generated by `Wallet::genesis()` can be used legitimately in the real Bitcoin protocol.

---

## Concurrency Design

Mining (`ParallelMiner`) is the only module in the project that makes heavy use of multi-threading. The blockchain state itself (the `Blockchain` struct) follows a single-threaded ownership model; Rust's borrow checker guarantees data safety at compile time, eliminating the need for runtime lock overhead.

```rust
// Parallel mining: automatically partitions the nonce search space based on CPU core count
self.miner
    .mine_block(&mut block, self.difficulty)
    .map_err(|e| format!("Mining failed: {}", e))?;
```

---

## Error Handling

All public APIs return `Result<T, String>` or `crate::error::Result<T>` (i.e., `Result<T, BitcoinError>`). `BitcoinError` is a unified enum type covering:

- `PrivateKeyError` — key format error
- Insufficient balance, UTXO not found, signature verification failure, and other domain errors

```rust
use bitcoin_simulation::{BitcoinError, Result};

fn example() -> Result<()> {
    let blockchain = Blockchain::new();
    // ...
    Ok(())
}
```
