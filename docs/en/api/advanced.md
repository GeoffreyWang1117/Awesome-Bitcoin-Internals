# Advanced Modules

SimpleBTC's advanced modules build a complete set of Bitcoin protocol features on top of the core blockchain functionality. These modules work together to cover the full stack — from data integrity verification to complex multi-party signatures, and from lightweight payment verification to script language execution.

---

## Module Overview

| Module | Source File | Core Functionality |
|--------|------------|-------------------|
| [Merkle Tree](merkle.md) | `src/merkle.rs` | Data integrity verification, SPV proof generation and verification |
| [Multisig](multisig.md) | `src/multisig.rs` | M-of-N multi-party signature addresses and transaction construction |
| [Advanced Transactions](advanced-tx.md) | `src/advanced_tx.rs` | RBF replacement mechanism, timelocks, fee estimation |
| Mempool | `src/mempool.rs` | Unconfirmed transaction management and priority ordering |
| Script Engine | `src/script.rs` | Bitcoin Script subset interpretation and execution |
| SPV | `src/spv.rs` | Lightweight payment verification client |

---

## Merkle Tree

**Source file:** `src/merkle.rs` | **Documentation:** [Merkle API](merkle.md)

The Merkle tree is the foundation of blockchain data integrity. SimpleBTC uses SHA256 to build a binary hash tree, aggregating all transaction hashes in a block into a single 32-byte `merkle_root` stored in the block header.

The core value lies in supporting **SPV (Simplified Payment Verification)**: a light wallet does not need to download the full block (1–2 MB); it only needs to obtain the block header (80 bytes) and an O(log n) hash path to cryptographically prove that a transaction has been confirmed.

```rust
use simplebtc::merkle::MerkleTree;

let tx_ids = vec!["tx1_hash".to_string(), "tx2_hash".to_string()];
let tree = MerkleTree::new(&tx_ids);
let root = tree.get_root_hash();

// Generate and verify an SPV proof
let proof = tree.get_proof("tx1_hash").unwrap();
let valid = MerkleTree::verify_proof("tx1_hash", &proof, &root, 0);
```

The Merkle tree is called internally by `Block::new()` to compute `merkle_root`, and is also used by `Block::verify_transaction_inclusion()` for SPV verification.

---

## Multisig

**Source file:** `src/multisig.rs` | **Documentation:** [MultiSig API](multisig.md)

Multisig implements Bitcoin's M-of-N signature scheme: N participants each hold an ECDSA key pair, and at least M of them must sign to authorize fund movement. This is the core mechanism in the Bitcoin protocol for implementing distributed control and risk distribution.

Typical use cases include: 2-of-3 corporate fund management (preventing single-person embezzlement), 2-of-3 third-party escrow (buyer-seller-arbitrator), and personal multi-device backup (recovery is still possible even if the primary key is lost).

```rust
use simplebtc::multisig::{MultiSigAddress, MultiSigTxBuilder};
use simplebtc::wallet::Wallet;

// Create a 2-of-3 multisig address
let (w1, w2, w3) = (Wallet::new(), Wallet::new(), Wallet::new());
let pub_keys = vec![w1.public_key.clone(), w2.public_key.clone(), w3.public_key.clone()];
let ms_addr = MultiSigAddress::new(2, pub_keys).unwrap();

// Collect signatures (any two participants can sign)
let mut builder = MultiSigTxBuilder::new(ms_addr);
builder.add_signature(&w1, "transaction data").unwrap();
builder.add_signature(&w2, "transaction data").unwrap();
assert!(builder.is_complete());
```

Multisig addresses start with `"3"` (corresponding to Bitcoin's P2SH address format), generated via script hash, and support up to 15 participating keys.

---

## Advanced Transactions

**Source file:** `src/advanced_tx.rs` | **Documentation:** [Advanced TX API](advanced-tx.md)

The advanced transaction module provides three key features that address real engineering problems in the Bitcoin network:

**RBF (Replace-By-Fee):** Allows users to replace an unconfirmed transaction with a new one bearing a higher fee, thereby accelerating confirmation or canceling an erroneous transaction. `RBFManager` maintains a list of replaceable transactions and enforces replacement validation rules (same inputs, higher fee, increment meets minimum requirement).

**TimeLock:** Restricts a transaction from being mined before a specified time or block height. Supports two types: Unix timestamp-based (`new_time_based`) and block height-based (`new_height_based`). Commonly used for time deposits, inheritance, smart contracts, and similar scenarios.

**TxPriorityCalculator:** Recommends a reasonable fee based on transaction size and urgency (`Low`/`Medium`/`High`/`Urgent`), and calculates a composite priority score (70% fee rate weight + 30% priority weight).

```rust
use simplebtc::advanced_tx::{AdvancedTxBuilder, TimeLock, RBFManager, TxPriorityCalculator, FeeUrgency};

// Combined RBF + timelock usage
let timelock = TimeLock::new_height_based(850_000);
let builder = AdvancedTxBuilder::new()
    .with_rbf()
    .with_timelock(timelock);

// Recommended fee
let fee = TxPriorityCalculator::recommend_fee(250, FeeUrgency::High); // 250-byte transaction
println!("Recommended fee: {} satoshi", fee); // ~5000 satoshi
```

---

## Mempool

**Source file:** `src/mempool.rs`

The mempool (Memory Pool) stores all transactions that have been broadcast but not yet packed into a block. Miners select transactions from the mempool by priority to build new blocks.

Main responsibilities:
- Receive and temporarily store broadcast transactions
- Sort by fee rate, offering high-value transactions first to miners
- Detect and reject double-spend attempts
- Handle transaction replacement in conjunction with the RBF mechanism
- Remove packed transactions from the pool after block confirmation

```rust
use simplebtc::mempool::Mempool;

let mut pool = Mempool::new();
pool.add_transaction(tx);
let pending = pool.get_pending_transactions(10); // Get the 10 highest-fee transactions
```

---

## Script Engine

**Source file:** `src/script.rs`

Bitcoin Script is a simple stack-based scripting language used to define the spending conditions for transactions. SimpleBTC implements the core subset of Script, supporting the most common transaction types.

Supported script types:
- **P2PKH** (Pay-to-Public-Key-Hash): The most common ordinary address transaction; lock script format is `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`.
- **P2SH** (Pay-to-Script-Hash): The basis for multisig and complex contracts; addresses starting with `"3"`.
- **OP_RETURN**: Writes arbitrary non-spendable data on-chain (up to 80 bytes).

The script engine provides the underlying support for the multisig module: the `script` field of `MultiSigAddress` stores a simplified Script locking script.

---

## SPV (Simplified Payment Verification)

**Source file:** `src/spv.rs`

SPV simulates the operation of Bitcoin light wallets: validating the legitimacy of transactions without downloading the full blockchain. This is critical for resource-constrained devices (mobile phones, embedded systems).

SPV verification flow:
1. Download only block headers (each approximately 80 bytes, all headers approximately 60 MB)
2. Verify the proof of work (PoW) of the block headers
3. Request the Merkle proof for the target transaction (a few hundred bytes)
4. Execute `MerkleTree::verify_proof()` locally

```
Full node mode: Download full blockchain (~500 GB) → Full local verification
SPV mode:       Download block headers (~60 MB) + Merkle proof (few KB) → O(log n) verification
```

The SPV module is deeply integrated with the Merkle module, relying on `MerkleTree::get_proof()` and `MerkleTree::verify_proof()` for lightweight verification.

---

## Module Dependencies

```
Core Modules
├── Transaction (src/transaction.rs)
├── Wallet      (src/wallet.rs)
└── Block       (src/block.rs)
        │
        ▼
Advanced Modules (built on top of core modules)
├── Merkle      ← Used internally by Block (computing merkle_root and SPV proofs)
├── MultiSig    ← Depends on Wallet (ECDSA signing) + Script (locking scripts)
├── AdvancedTx  ← Depends on Transaction (RBF replacement validation)
├── Mempool     ← Depends on Transaction + AdvancedTx (RBF support)
├── Script      ← Foundation for MultiSig and SPV
└── SPV         ← Depends on Merkle (proof verification) + Block (block headers)
```

---

## Quick Navigation

- [Merkle API](merkle.md) — Merkle tree data structure and SPV proofs
- [MultiSig API](multisig.md) — M-of-N multisig
- [Advanced TX API](advanced-tx.md) — RBF, timelocks, fee calculation
- [Block API](block.md) — Block structure and mining
- [Transaction API](transaction.md) — Transaction construction and validation
- [Wallet API](wallet.md) — Wallet and key management
