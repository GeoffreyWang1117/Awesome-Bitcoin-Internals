# Blockchain Operations

The blockchain is the core data structure of SimpleBTC — a cryptographically linked sequence of blocks, where each block contains a batch of validated transactions. This chapter covers the `Block` struct, the creation and management of `Blockchain`, the proof-of-work mining mechanism, and the chain validation and query interfaces.

---

## Block Structure

### Block Data Structure

```rust
// src/block.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,                     // Block height (genesis block is 0)
    pub timestamp: u64,                 // Unix timestamp (seconds)
    pub transactions: Vec<Transaction>, // Transaction list (first must be Coinbase)
    pub previous_hash: String,          // Parent block hash (64-character SHA-256 hex)
    pub hash: String,                   // This block's hash (valid value found by mining)
    pub nonce: u64,                     // Proof-of-work counter
    pub merkle_root: String,            // Merkle tree root hash of all transaction IDs
}
```

**Block header field details:**

| Field | Bitcoin Equivalent | Description |
|-------|-------------------|-------------|
| `index` | Block Height | Block height; genesis block is 0, incremented by 1 for each new block |
| `timestamp` | nTime | Block creation time (Unix timestamp) |
| `previous_hash` | hashPrevBlock | Parent block hash, forming the chain structure |
| `merkle_root` | hashMerkleRoot | Merkle tree root of all transactions; a fingerprint of the block's contents |
| `nonce` | nNonce | A random number continuously adjusted during mining |
| `hash` | Block Hash | SHA-256 hash of all block header fields |

### Chain Structure Diagram

```
Genesis Block (index=0)     Block 1               Block 2
┌──────────────────┐     ┌──────────────────┐  ┌──────────────────┐
│ prev: "0"        │◄────│ prev: abc...hash │◄─│ prev: def...hash │
│ hash: abc...     │     │ hash: def...     │  │ hash: ghi...     │
│ nonce: 38291     │     │ nonce: 72481     │  │ nonce: 19374     │
│ merkle: xyz...   │     │ merkle: pqr...   │  │ merkle: stu...   │
│ [Coinbase TX]    │     │ [Coinbase TX]    │  │ [Coinbase TX]    │
│                  │     │ [TX_1]           │  │ [TX_3]           │
│                  │     │ [TX_2]           │  │ [TX_4]           │
└──────────────────┘     └──────────────────┘  └──────────────────┘
```

**Why does the chain structure guarantee immutability?**

1. Modify any transaction in block 1 → `merkle_root` changes
2. `merkle_root` changes → block 1's `hash` is completely different
3. Block 2 recorded block 1's old `hash` → block 2's `previous_hash` no longer matches
4. Fixing block 2 requires re-mining (recalculating PoW), and the same applies to blocks 3, 4...
5. An attacker would need to control more than 51% of the total network hashrate to catch up with the honest chain

---

## Block Hash Calculation

The block hash is computed from the key fields of the block header (note: transactions are not hashed directly; the Merkle root is used instead):

```rust
// src/block.rs
pub fn calculate_hash(&self) -> String {
    use sha2::{Digest, Sha256};

    // Concatenate block header fields into a string
    let data = format!(
        "{}{}{}{}{}",
        self.index,
        self.timestamp,
        self.merkle_root,    // ← represents all transaction content
        self.previous_hash,
        self.nonce           // ← this value changes continuously during mining
    );

    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**The role of the Merkle root:**
- Any change to a single transaction will cause the Merkle root to change completely
- Verifying whether a transaction is included in a block requires only O(log n) hashes (a Merkle proof), rather than downloading all transactions

---

## Creating a Blockchain

### Blockchain Struct

```rust
// src/blockchain.rs
pub struct Blockchain {
    pub chain: Vec<Block>,           // Block list (the chain)
    pub difficulty: usize,           // Mining difficulty (number of leading zeros, default 3)
    pub mempool: Mempool,            // Memory pool (pending transactions, sorted by fee rate)
    pub utxo_set: UTXOSet,           // UTXO set (all unspent outputs)
    pub mining_reward: u64,          // Mining reward (default 50 satoshi)
    pub indexer: TransactionIndexer, // Transaction indexer (address → transaction, for fast queries)
    miner: ParallelMiner,            // Parallel PoW miner (private)
    pending_spent: HashSet<String>,  // UTXOs spent by pending transactions (double-spend prevention)
}
```

### Initializing the Blockchain

```rust
use bitcoin_simulation::blockchain::Blockchain;

// Create a blockchain (automatically includes the genesis block)
let mut blockchain = Blockchain::new();

println!("Chain length: {}", blockchain.chain.len());     // 1 (genesis block only)
println!("Mining difficulty: {}", blockchain.difficulty);     // 3
println!("Mining reward: {} satoshi", blockchain.mining_reward); // 50
```

Internal flow of `Blockchain::new()`:

```rust
pub fn new() -> Blockchain {
    let mempool = Mempool::new_permissive();

    let mut blockchain = Blockchain {
        chain: vec![],
        difficulty: 3,
        mempool,
        utxo_set: UTXOSet::new(),
        mining_reward: 50,
        indexer: TransactionIndexer::new(),
        miner: ParallelMiner::default(),
        pending_spent: HashSet::new(),
    };

    // Create and add the genesis block
    let genesis_block = blockchain.create_genesis_block();
    blockchain.indexer.index_block(&genesis_block);
    blockchain.chain.push(genesis_block);

    blockchain
}
```

---

## Genesis Block

The genesis block is the first block in the blockchain (index = 0). What makes it special:

- `previous_hash` = `"0"` (does not reference any parent block)
- Contains a Coinbase transaction that issues 10,000,000 satoshi of initial funds to the **genesis wallet**
- Uses a **deterministic genesis wallet** (fixed private key `0x01`), ensuring the address is the same every time the system starts

```rust
// src/blockchain.rs
fn create_genesis_block(&mut self) -> Block {
    let timestamp = /* current Unix time */;

    // Deterministic genesis wallet (fixed private key, can be spent with a signature)
    let genesis_wallet = Wallet::genesis();
    let coinbase_tx = Transaction::new_coinbase(
        genesis_wallet.address,
        10_000_000,  // Genesis block reward: 10M satoshi
        timestamp,
        0,           // No fee
    );

    // Add the genesis UTXO to the UTXO set
    self.utxo_set.add_transaction(&coinbase_tx);

    // The genesis block's previous_hash is fixed as "0"
    Block::new(0, vec![coinbase_tx], "0".to_string())
}
```

Two equivalent ways to obtain the genesis wallet:

```rust
let genesis = Blockchain::genesis_wallet();  // Static method of Blockchain
let genesis2 = Wallet::genesis();            // Obtained directly from the wallet module
assert_eq!(genesis.address, genesis2.address);
```

---

## Proof of Work (PoW) Mining

### Principle

Proof of work requires miners to find a `nonce` value such that the block hash satisfies the condition "the first N digits are 0":

```
difficulty = 3, target hash format: 000xxxxxxxxx...
```

Since the output of SHA-256 is completely unpredictable, miners can only brute-force the nonce:

```
nonce=0: hash = "a7f3b2..." → not satisfied (does not start with "000")
nonce=1: hash = "2c91d4..." → not satisfied
...
nonce=38291: hash = "000a4b7c9..." → satisfied! Block mined
```

On average, 16³ = 4096 attempts are needed (difficulty 3). Real Bitcoin's difficulty is equivalent to about 20 leading zeros, requiring approximately 2⁸⁰ attempts.

### Block::mine_block() (single-threaded)

```rust
// src/block.rs
pub fn mine_block(&mut self, difficulty: usize) {
    let target = "0".repeat(difficulty);

    while self.hash[..difficulty] != target {
        self.nonce += 1;
        self.hash = self.calculate_hash();
    }

    println!("✓ Block mined: {}", self.hash);
}
```

### ParallelMiner (multi-threaded)

`Blockchain::mine_pending_transactions()` uses `ParallelMiner` instead of the single-threaded `mine_block()`, making full use of multi-core CPUs:

```rust
// src/blockchain.rs excerpt
self.miner
    .mine_block(&mut block, self.difficulty)
    .map_err(|e| format!("Mining failed: {}", e))?;
```

`ParallelMiner` splits the nonce space among multiple threads to search in parallel; the first thread to find a valid hash wins.

### Difficulty and Adjustment

| Difficulty | Leading Zeros | Average Attempts | Use Case |
|------------|--------------|-----------------|----------|
| 1 | 1 zero | 16 | Very fast testing |
| 2 | 2 zeros | 256 | Quick demo |
| 3 | 3 zeros | 4,096 | Default config |
| 4 | 4 zeros | 65,536 | Performance testing |
| 6 | 6 zeros | 16,777,216 | Close to real-world |

---

## Adding Transactions and Mining

### Complete Flow

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

let mut blockchain = Blockchain::new();
let genesis = Blockchain::genesis_wallet();
let alice = Wallet::new();

// 1. Create a transaction
let tx = blockchain.create_transaction(
    &genesis,
    alice.address.clone(),
    5000,   // Transfer 5000 satoshi
    50,     // Fee 50 satoshi
)?;

// 2. Add to mempool (verify signature + UTXO)
blockchain.add_transaction(tx)?;

println!("Mempool transaction count: {}", blockchain.mempool.len()); // 1

// 3. Mine (alice receives the reward as miner)
blockchain.mine_pending_transactions(alice.address.clone())?;

println!("Chain length: {}", blockchain.chain.len()); // 2 (genesis + new block)
println!("Mempool transaction count: {}", blockchain.mempool.len()); // 0 (cleared)
```

### Detailed flow of mine_pending_transactions()

```rust
pub fn mine_pending_transactions(&mut self, miner_address: String) -> Result<(), String> {
    if self.mempool.is_empty() {
        return Err("No pending transactions".to_string());
    }

    // 1. Fetch high-fee-rate transactions from the mempool (already sorted)
    let pending_txs = self.mempool.get_top_transactions(usize::MAX);

    // 2. Calculate total fees
    let total_fees: u64 = pending_txs.iter().map(|tx| tx.fee).sum();

    // 3. Create Coinbase transaction (miner reward = block reward + total fees)
    let coinbase_tx = Transaction::new_coinbase(
        miner_address,
        self.mining_reward,  // 50 satoshi
        timestamp,
        total_fees,
    );

    // 4. Assemble the block (Coinbase must be the first transaction)
    let mut transactions = vec![coinbase_tx];
    transactions.extend(pending_txs.iter().cloned());

    let previous_hash = self.chain.last().unwrap().hash.clone();
    let mut block = Block::new(self.chain.len() as u32, transactions, previous_hash);

    // 5. Parallel PoW mining
    self.miner.mine_block(&mut block, self.difficulty)?;

    // 6. Verify all transaction signatures in the block
    if !block.validate_transactions() {
        return Err("Block contains invalid transactions".to_string());
    }

    // 7. Update UTXO set (consume input UTXOs, create output UTXOs)
    for tx in &block.transactions {
        if !self.utxo_set.process_transaction(tx) {
            return Err("UTXO update failed".to_string());
        }
    }

    // 8. Add block to chain + build index
    self.indexer.index_block(&block);
    self.chain.push(block);

    // 9. Clear mempool and pending_spent
    for tx in &pending_txs {
        let _ = self.mempool.remove_transaction(&tx.id);
    }
    self.pending_spent.clear();

    Ok(())
}
```

---

## Merkle Tree and Transaction Verification

`Block::new()` automatically builds the Merkle tree and calculates the Merkle root upon creation:

```rust
pub fn new(index: u32, transactions: Vec<Transaction>, previous_hash: String) -> Block {
    // Collect all transaction IDs
    let tx_ids: Vec<String> = transactions.iter().map(|tx| tx.id.clone()).collect();

    // Build Merkle tree, calculate root hash
    let merkle_tree = MerkleTree::new(&tx_ids);
    let merkle_root = merkle_tree.get_root_hash();

    let mut block = Block {
        index, timestamp, transactions, previous_hash,
        hash: String::new(), nonce: 0, merkle_root,
    };
    block.hash = block.calculate_hash();
    block
}
```

Verifying whether a transaction is included in a block (SPV use case):

```rust
// src/block.rs
pub fn verify_transaction_inclusion(&self, tx_id: &str, index: usize) -> bool {
    let tx_ids: Vec<String> = self.transactions.iter().map(|tx| tx.id.clone()).collect();
    let merkle_tree = MerkleTree::new(&tx_ids);

    if let Some(proof) = merkle_tree.get_proof(tx_id) {
        MerkleTree::verify_proof(tx_id, &proof, &self.merkle_root, index)
    } else {
        false
    }
}
```

---

## Chain Validation

`Blockchain::is_valid()` validates the integrity of the chain block by block, starting from block 1 (skipping the genesis block):

```rust
pub fn is_valid(&self) -> bool {
    for i in 1..self.chain.len() {
        let current = &self.chain[i];
        let previous = &self.chain[i - 1];

        // 1. Verify the block's own hash (prevent silent data tampering)
        if current.hash != current.calculate_hash() {
            println!("Block {} has invalid hash", i);
            return false;
        }

        // 2. Verify forward reference (chain link integrity)
        if current.previous_hash != previous.hash {
            println!("Block {} has invalid forward reference", i);
            return false;
        }

        // 3. Verify proof of work (hash leading zeros satisfy difficulty requirement)
        let target = "0".repeat(self.difficulty);
        if current.hash[..self.difficulty] != target {
            println!("Block {} has invalid proof of work", i);
            return false;
        }

        // 4. Verify ECDSA signatures of all transactions in the block
        if !current.validate_transactions() {
            println!("Block {} contains invalid transactions", i);
            return false;
        }
    }
    true
}
```

Validation example:

```rust
let mut blockchain = Blockchain::new();
// ... add transactions, mine ...

// Normal case: should pass
assert!(blockchain.is_valid());

// Simulate tampering (for educational purposes; Rust's borrow rules constrain direct access in practice)
// If someone modifies a transaction in a historical block, is_valid() will return false
```

---

## Balance Query

Balances are calculated via the UTXO set, avoiding a scan of all historical blocks:

```rust
pub fn get_balance(&self, address: &str) -> u64 {
    self.utxo_set.get_balance(address)
}
```

Usage example:

```rust
let balance = blockchain.get_balance(&alice.address);
println!("Alice balance: {} satoshi", balance);
println!("Alice balance: {:.8} BTC", balance as f64 / 1e8);
```

**Performance advantage of UTXOSet:**

Without a UTXO set, querying a balance requires scanning all transactions in all blocks (O(n), where n = total number of transactions). The UTXO set caches all current unspent outputs in memory, making a query an O(1) hash table lookup.

---

## Printing Blockchain Information

`Blockchain::print_chain()` provides formatted debug output:

```rust
blockchain.print_chain();
```

Sample output:

```
========== Blockchain Info ==========

--- Block #0 ---
Timestamp: 1711497600
Hash: 000a4b7c9d2e1f3a...
Previous hash: 0
Nonce: 38291
Transaction count: 1
  Transaction #0: f3a1b2c4...
    Type: Coinbase (mining reward)
    Input count: 1
    Output count: 1
      Output 0: 10000000 -> 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2

--- Block #1 ---
Timestamp: 1711497615
Hash: 000d2f8a1b9e4c7f...
Previous hash: 000a4b7c9d2e1f3a...
Nonce: 72481
Transaction count: 2
  Transaction #0: a1b2c3d4...
    Type: Coinbase (mining reward)
    ...
  Transaction #1: e5f6a7b8...
    Fee: 50 satoshi
    Fee rate: 0.23 sat/byte
    Input count: 1
    Output count: 2
      Output 0: 5000 -> 1AliceAddress...
      Output 1: 4994950 -> 1GenesisAddress...

================================
```

---

## Complete Operation Example

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn main() -> Result<(), String> {
    // Initialize
    let mut blockchain = Blockchain::new();
    let genesis = Blockchain::genesis_wallet();
    let alice = Wallet::new();
    let bob = Wallet::new();
    let miner = Wallet::new();

    // Round 1: genesis → alice
    let tx1 = blockchain.create_transaction(&genesis, alice.address.clone(), 100_000, 100)?;
    blockchain.add_transaction(tx1)?;
    blockchain.mine_pending_transactions(miner.address.clone())?;

    println!("Block 1 mined");
    println!("Alice balance: {} sat", blockchain.get_balance(&alice.address));
    println!("Miner balance: {} sat", blockchain.get_balance(&miner.address));

    // Round 2: alice → bob (two transactions in the same block)
    let tx2 = blockchain.create_transaction(&alice, bob.address.clone(), 30_000, 200)?;
    let tx3 = blockchain.create_transaction(&alice, miner.address.clone(), 20_000, 150)?;
    blockchain.add_transaction(tx2)?;
    blockchain.add_transaction(tx3)?;
    blockchain.mine_pending_transactions(miner.address.clone())?;

    println!("\nBlock 2 mined (contains 2 transactions)");
    println!("Chain length: {}", blockchain.chain.len()); // 3
    println!("Alice balance: {} sat", blockchain.get_balance(&alice.address));
    println!("Bob balance:   {} sat", blockchain.get_balance(&bob.address));
    println!("Miner balance: {} sat", blockchain.get_balance(&miner.address));

    // Verify chain integrity
    assert!(blockchain.is_valid(), "Chain validation should pass");
    println!("\nChain validation passed!");

    // Print full chain information
    blockchain.print_chain();

    Ok(())
}
```

---

## Key Parameter Reference

| Parameter | Default | Description |
|-----------|---------|-------------|
| `difficulty` | 3 | Mining difficulty (number of leading zeros) |
| `mining_reward` | 50 | Base block reward (satoshi) |
| Genesis block reward | 10,000,000 | Genesis Coinbase amount (satoshi) |
| Real Bitcoin initial reward | 5,000,000,000 | 50 BTC (in satoshi) |
| Real Bitcoin halving interval | 210,000 blocks | Approximately every 4 years |
| Real Bitcoin target block time | 10 minutes | Difficulty adjusted approximately every 2016 blocks |
