# Block API

`Block` is the fundamental unit of the blockchain, defined in `src/block.rs`. Each block contains a batch of confirmed transactions and is linked to the previous block via a hash chain, together forming an immutable ledger.

---

## Block Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,                     // Block height (index); genesis block is 0
    pub timestamp: u64,                 // Unix timestamp (seconds)
    pub transactions: Vec<Transaction>, // Transaction list (first must be a Coinbase transaction)
    pub previous_hash: String,          // Parent block hash (SHA256, 64-character hex)
    pub hash: String,                   // Current block hash (found through mining)
    pub nonce: u64,                     // Proof-of-work nonce (adjusted during mining)
    pub merkle_root: String,            // Merkle tree root hash of all transactions
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `index` | `u32` | Block height. Genesis block is `0`; each subsequent block increments by 1. |
| `timestamp` | `u64` | Unix timestamp (seconds) when the block was created. Automatically set by `SystemTime::now()`. |
| `transactions` | `Vec<Transaction>` | List of transactions in the block. The first **must** be a Coinbase transaction (miner reward). |
| `previous_hash` | `String` | SHA256 hash (64-character hex) of the parent block. For the genesis block, this field is `"0"`. |
| `hash` | `String` | SHA256 hash of the current block. Computed by `calculate_hash()`, continuously updated during mining until the difficulty requirement is met. |
| `nonce` | `u64` | Proof-of-work nonce. Miners increment `nonce` to find a hash satisfying the difficulty target. |
| `merkle_root` | `String` | Merkle tree root hash of all transactions in the block. Any tampered transaction will change this value. |

### Chain Structure

```
Genesis Block (index=0)  ->   Block 1         ->   Block 2
prev: "0"                     prev: abc123...       prev: def456...
hash: abc123...               hash: def456...       hash: ghi789...
```

Because each block's `hash` depends on `previous_hash` and all transactions (via `merkle_root`), modifying any historical block requires recomputing the hash of all subsequent blocks — computationally infeasible.

---

## Methods

### `Block::new`

Creates a new block. Automatically sets the timestamp and computes the Merkle root, but `nonce` starts at `0` and `hash` is the initial computed value (does not yet satisfy mining difficulty).

```rust
pub fn new(
    index: u32,
    transactions: Vec<Transaction>,
    previous_hash: String,
) -> Block
```

**Parameters:**
- `index` — Height of the new block.
- `transactions` — List of transactions to include in the block (first should be a Coinbase transaction).
- `previous_hash` — Hash string of the parent block.

**Return value:** An initialized `Block` instance (mining not yet complete).

**Internal flow:**
1. Obtain the current Unix timestamp.
2. Build a `MerkleTree` from the `id` list of `transactions` and compute `merkle_root`.
3. Construct the block with `nonce = 0` and call `calculate_hash()` to get the initial hash.

```rust
use simplebtc::block::Block;
use simplebtc::transaction::Transaction;

let coinbase = Transaction::new_coinbase("miner_address", 3125000); // 3.125 BTC (satoshi)
let block = Block::new(1, vec![coinbase], "abc123...".to_string());
println!("Block #{}: {}", block.index, block.hash);
```

---

### `Block::calculate_hash`

Computes the SHA256 hash of the block. The hash input includes `index`, `timestamp`, `merkle_root`, `previous_hash`, and `nonce`.

```rust
pub fn calculate_hash(&self) -> String
```

**Return value:** 64-character lowercase hexadecimal SHA256 hash string.

**Hash input format:**
```
"{index}{timestamp}{merkle_root}{previous_hash}{nonce}"
```

Using `merkle_root` rather than the full transaction data keeps the block header lightweight (approximately 80 bytes) while ensuring the integrity of all transaction content.

```rust
let mut block = Block::new(1, transactions, prev_hash);
// Recompute hash after modifying nonce (core mining logic)
block.nonce += 1;
block.hash = block.calculate_hash();
println!("New hash: {}", block.hash);
```

---

### `Block::validate_transactions`

Validates the signature validity of all transactions in the block. Calls each transaction's `verify()` method in sequence.

```rust
pub fn validate_transactions(&self) -> bool
```

**Return value:**
- `true` — All transaction signatures are valid.
- `false` — At least one invalid transaction exists.

```rust
let block = Block::new(1, transactions, prev_hash);

if block.validate_transactions() {
    println!("All transactions valid, can be added to chain");
} else {
    println!("Block contains invalid transactions, rejected");
}
```

> **Note:** This method only validates signatures, not UTXO balances. Balance validation is handled at the `Blockchain` layer.

---

### `Block::verify_transaction_inclusion`

Verifies whether a specific transaction is included in this block using a Merkle proof. This is the core functionality of SPV (Simplified Payment Verification), requiring no traversal of all transactions — time complexity is O(log n).

```rust
pub fn verify_transaction_inclusion(
    &self,
    tx_id: &str,
    index: usize,
) -> bool
```

**Parameters:**
- `tx_id` — The transaction ID (hash string) to verify.
- `index` — The position index of the transaction in the block's transaction list (starting from 0).

**Return value:**
- `true` — The transaction is indeed included in this block and the Merkle proof is valid.
- `false` — The transaction is not in this block, or the proof is invalid.

**Internal flow:**
1. Reconstruct the block's `MerkleTree`.
2. Call `get_proof(tx_id)` to generate a Merkle proof.
3. Call `MerkleTree::verify_proof()` to verify the proof against `merkle_root`.

```rust
let tx_id = "abc123def456...";
let tx_index = 2; // Position of the transaction in the block

if block.verify_transaction_inclusion(tx_id, tx_index) {
    println!("Transaction confirmed in block #{}", block.index);
} else {
    println!("Transaction is not in this block");
}
```

---

### `Block::mine_block`

Proof-of-Work mining. Continuously increments `nonce` and recomputes the hash until the hash prefix satisfies the difficulty requirement (i.e., starts with `difficulty` number of `'0'` characters).

```rust
pub fn mine_block(&mut self, difficulty: usize)
```

**Parameters:**
- `difficulty` — Mining difficulty: the number of leading `'0'` characters required in the hash.

**Side effects:** Modifies `self.nonce` and `self.hash` until a valid hash is found.

```rust
let mut block = Block::new(1, transactions, prev_hash);
println!("Starting mining, difficulty: 4");
block.mine_block(4); // Hash must start with "0000"
println!("Mining complete: {}", block.hash);
println!("Nonce used: {}", block.nonce);
// Example output: 0000a3f7c2...
```

> **About difficulty:** Bitcoin mainnet's current difficulty is roughly equivalent to about 20 leading `'0'` characters in the hash (requiring approximately 2^80 hash calculations). This project uses smaller difficulty values (such as 2–4) for demonstration purposes.

---

## Complete Usage Example

```rust
use simplebtc::block::Block;
use simplebtc::transaction::Transaction;
use simplebtc::wallet::Wallet;

fn main() {
    // 1. Create miner wallet
    let miner = Wallet::new();

    // 2. Create Coinbase transaction (miner reward)
    let coinbase = Transaction::new_coinbase(&miner.address, 3_125_000);

    // 3. Create a regular transfer transaction
    let alice = Wallet::new();
    let bob = Wallet::new();
    let transfer = Transaction::new(&alice, &bob.address, 50_000, 500);

    // 4. Pack into a block (assuming parent hash is known)
    let prev_hash = "0000abc123...".to_string();
    let mut block = Block::new(1, vec![coinbase, transfer], prev_hash);

    // 5. Mine (proof of work)
    block.mine_block(3); // Difficulty 3: hash starts with "000"

    // 6. Validate the block
    assert!(block.validate_transactions(), "Block transactions invalid");
    println!("Block hash: {}", block.hash);
    println!("Merkle root: {}", block.merkle_root);
    println!("Nonce: {}", block.nonce);

    // 7. SPV verification: is a transaction included in this block?
    let included = block.verify_transaction_inclusion(&block.transactions[0].id.clone(), 0);
    println!("Coinbase transaction included: {}", included);
}
```

---

## Immutability Principle

```
Attacker attempts to modify a transaction in Block 1:

  Modify transaction
      ↓
  Transaction hash changes
      ↓
  Merkle Root changes
      ↓
  Block 1's Hash changes
      ↓
  Block 2's previous_hash no longer matches
      ↓
  Hashes of Blocks 2, 3, 4... all become invalid
      ↓
  Attacker must re-mine all subsequent blocks (computationally infeasible)
```

This is the mathematical guarantee of blockchain "immutability."

---

## Related Modules

- [`MerkleTree`](merkle.md) — Merkle tree implementation, used to compute `merkle_root` and generate SPV proofs.
- [`Transaction`](transaction.md) — Transaction struct, the core data of a Block.
- [`Blockchain`](blockchain.md) — Manages the blockchain, calls `mine_block()`, and maintains chain state.
