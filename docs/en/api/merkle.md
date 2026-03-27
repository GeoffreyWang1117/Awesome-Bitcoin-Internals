# Merkle API

The Merkle tree (hash tree) is implemented in `src/merkle.rs` and is the core data structure for blockchain data integrity verification. SimpleBTC uses SHA256 to build a binary Merkle tree, aggregating all transaction hashes in a block into a single root hash (`merkle_root`) stored in the block header.

---

## Data Structures

### MerkleNode Struct

A single node in the Merkle tree, which can be a leaf node (corresponding to one transaction) or an internal node (corresponding to the hash of its child node hashes).

```rust
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,                   // SHA256 hash of the node (64-character hex)
    pub left: Option<Box<MerkleNode>>,  // Left child node (None for leaf nodes)
    pub right: Option<Box<MerkleNode>>, // Right child node (None for leaf nodes)
}
```

| Field | Type | Description |
|-------|------|-------------|
| `hash` | `String` | Node hash. For leaf nodes: `SHA256(transaction ID)`; for internal nodes: `SHA256(left_hash + right_hash)`. |
| `left` | `Option<Box<MerkleNode>>` | Left child node. `None` for leaf nodes. |
| `right` | `Option<Box<MerkleNode>>` | Right child node. `None` for leaf nodes. |

#### MerkleNode Methods

```rust
// Create a leaf node from raw data (computes SHA256 hash)
pub fn new_leaf(data: &str) -> Self

// Create an internal node from two child nodes (hash = SHA256(left_hash + right_hash))
pub fn new_internal(left: MerkleNode, right: MerkleNode) -> Self
```

---

### MerkleTree Struct

The complete Merkle tree, holding the root node and the original leaf data list.

```rust
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>, // Root node (None when transaction list is empty)
    pub leaves: Vec<String>,      // Original leaf data list (transaction ID list)
}
```

| Field | Type | Description |
|-------|------|-------------|
| `root` | `Option<MerkleNode>` | The root node of the tree. `None` when input is empty. |
| `leaves` | `Vec<String>` | The original transaction ID list passed in during construction (unhashed). |

---

## Tree Structure Illustration

Example with 4 transactions:

```
              Root
             /    \
           H12    H34
          /  \   /  \
        H1  H2  H3  H4
        │    │   │   │
       tx1  tx2 tx3 tx4

Where:
  H1  = SHA256(tx1)
  H2  = SHA256(tx2)
  H12 = SHA256(H1 + H2)
  H34 = SHA256(H3 + H4)
  Root = SHA256(H12 + H34)
```

**Handling odd number of transactions:** If a layer has an odd number of nodes, the last node is duplicated to form a pair (e.g., with 3 transactions, tx3 is duplicated as tx3').

---

## Methods

### `MerkleTree::new`

Builds a complete Merkle tree from a list of transaction IDs. Uses a bottom-up approach to build layer by layer; time complexity O(n).

```rust
pub fn new(transactions: &[String]) -> Self
```

**Parameters:**
- `transactions` — A slice of transaction IDs (or arbitrary strings). Can be empty, in which case `root` is `None`.

**Return value:** A fully constructed `MerkleTree` instance.

```rust
use simplebtc::merkle::MerkleTree;

// Build tree from transaction ID list
let tx_ids = vec![
    "tx_hash_1".to_string(),
    "tx_hash_2".to_string(),
    "tx_hash_3".to_string(),
    "tx_hash_4".to_string(),
];
let tree = MerkleTree::new(&tx_ids);

// Handle empty transaction list
let empty_tree = MerkleTree::new(&[]);
assert!(empty_tree.root.is_none());
```

---

### `MerkleTree::get_root_hash`

Gets the root hash string of the Merkle tree. This value is stored in the `merkle_root` field of the block header.

```rust
pub fn get_root_hash(&self) -> String
```

**Return value:**
- 64-character lowercase hexadecimal SHA256 hash string (when tree is non-empty).
- Empty string `""` (when tree is empty, i.e., `root` is `None`).

```rust
let tree = MerkleTree::new(&tx_ids);
let root_hash = tree.get_root_hash();
println!("Merkle root: {}", root_hash);
// Output: a3f7c2e1b4d9...（64-character hex）

// Compare with the value stored in the block
assert_eq!(root_hash, block.merkle_root);
```

---

### `MerkleTree::get_proof`

Generates a Merkle proof (SPV proof) for a specified transaction. The proof is a set of sibling node hashes; an SPV client uses these hashes to reconstruct the root hash layer by layer from the leaf, without accessing the full block.

```rust
pub fn get_proof(&self, tx_hash: &str) -> Option<Vec<String>>
```

**Parameters:**
- `tx_hash` — The transaction ID to generate a proof for (must exist in `self.leaves`).

**Return value:**
- `Some(Vec<String>)` — List of sibling node hashes needed for the proof, ordered from the leaf layer to the root layer.
- `None` — Transaction ID does not exist in this Merkle tree.

**Proof size:** For a block with n transactions, the proof contains `ceil(log2(n))` hashes, each 32 bytes. For example, a block with 2000 transactions requires only approximately 352 bytes of proof (11 hashes).

```rust
let tree = MerkleTree::new(&tx_ids);

match tree.get_proof("tx_hash_1") {
    Some(proof) => {
        println!("Proof contains {} sibling hashes", proof.len());
        for (i, hash) in proof.iter().enumerate() {
            println!("  Layer {}: {}", i, &hash[..16]);
        }
    }
    None => println!("Transaction does not exist in this Merkle tree"),
}
```

---

### `MerkleTree::verify_proof`

Verifies a Merkle proof (static method). This is the core function of SPV lightweight verification: using the sibling hashes in the proof, it computes upward layer by layer from the leaf node, verifying whether the final result matches the `merkle_root` in the block header.

```rust
pub fn verify_proof(
    tx_hash: &str,
    proof: &[String],
    root_hash: &str,
    index: usize,
) -> bool
```

**Parameters:**
- `tx_hash` — The transaction ID string to verify (raw value, not hashed).
- `proof` — List of sibling node hashes generated by `get_proof()`.
- `root_hash` — The `merkle_root` value stored in the block header.
- `index` — The position index of the transaction in the block's transaction list (starting from 0), used to determine left/right merge order.

**Return value:**
- `true` — Proof is valid; the transaction is indeed included in the corresponding block.
- `false` — Proof is invalid; the transaction is not in this block, or data has been tampered with.

**Verification algorithm:**

```
Input: tx_hash, proof = [sibling_0, sibling_1, ...], root_hash, index

Steps:
  current = SHA256(tx_hash)
  For each sibling_hash in proof:
    If index is even (current node is on the left):
      current = SHA256(current + sibling_hash)
    If index is odd (current node is on the right):
      current = SHA256(sibling_hash + current)
    index = index / 2

Final: current == root_hash → verification passes
```

```rust
use simplebtc::merkle::MerkleTree;

let tx_ids = vec![
    "tx1".to_string(),
    "tx2".to_string(),
    "tx3".to_string(),
    "tx4".to_string(),
];

let tree = MerkleTree::new(&tx_ids);
let root = tree.get_root_hash();

// Generate proof
let proof = tree.get_proof("tx1").expect("Transaction exists");

// Verify proof (index=0, tx1 is the first transaction)
let is_valid = MerkleTree::verify_proof("tx1", &proof, &root, 0);
assert!(is_valid, "SPV proof verification failed");
println!("Transaction tx1 confirmed included in block");

// Tamper test: proof becomes invalid after modifying transaction content
let tampered = MerkleTree::verify_proof("tx1_TAMPERED", &proof, &root, 0);
assert!(!tampered, "Proof should be invalid after tampering");
```

---

## Complete Usage Examples

### Example 1: Integration with a Block

```rust
use simplebtc::block::Block;
use simplebtc::merkle::MerkleTree;
use simplebtc::transaction::Transaction;
use simplebtc::wallet::Wallet;

fn main() {
    // Simulate packing 4 transactions
    let miner = Wallet::new();
    let alice = Wallet::new();
    let bob = Wallet::new();

    let transactions = vec![
        Transaction::new_coinbase(&miner.address, 3_125_000),
        Transaction::new(&alice, &bob.address, 100_000, 1_000),
        Transaction::new(&alice, &miner.address, 50_000, 500),
        Transaction::new(&bob, &alice.address, 20_000, 200),
    ];

    // Block::new internally builds a MerkleTree and computes merkle_root
    let block = Block::new(1, transactions, "000000abc...".to_string());
    println!("Merkle root: {}", block.merkle_root);

    // SPV verification: is tx[2] in this block?
    let tx_id = block.transactions[2].id.clone();
    let included = block.verify_transaction_inclusion(&tx_id, 2);
    println!("Transaction included in block: {}", included);
}
```

### Example 2: Using MerkleTree Standalone

```rust
use simplebtc::merkle::MerkleTree;

fn spv_demo() {
    // Full node builds complete Merkle tree
    let tx_ids: Vec<String> = (1..=8)
        .map(|i| format!("transaction_{:04}", i))
        .collect();

    let tree = MerkleTree::new(&tx_ids);
    let root = tree.get_root_hash();
    println!("Merkle root for 8 transactions: {}", root);

    // Generate SPV proof for tx #5 (index=4)
    let target_tx = "transaction_0005";
    let proof = tree.get_proof(target_tx).expect("Transaction exists");
    println!("Proof size: {} hashes (log2(8)=3 layers)", proof.len());

    // SPV client verification (only needs root + proof, not full transaction list)
    let verified = MerkleTree::verify_proof(target_tx, &proof, &root, 4);
    println!("SPV verification result: {}", verified);
}
```

### Example 3: Detecting Data Tampering

```rust
use simplebtc::merkle::MerkleTree;

fn tamper_detection() {
    let original = vec!["tx_a".to_string(), "tx_b".to_string(), "tx_c".to_string()];
    let tree = MerkleTree::new(&original);
    let original_root = tree.get_root_hash();

    // Simulate attacker modifying tx_b
    let mut tampered = original.clone();
    tampered[1] = "tx_b_MALICIOUS".to_string();
    let tampered_tree = MerkleTree::new(&tampered);
    let tampered_root = tampered_tree.get_root_hash();

    // Merkle roots are completely different; tampering is immediately detected
    assert_ne!(original_root, tampered_root);
    println!("Original root:  {}", &original_root[..16]);
    println!("Tampered root:  {}", &tampered_root[..16]);
    println!("Tamper detected: root hash has changed");
}
```

---

## Security Notes

**Why can Merkle proofs be trusted?**

For an attacker to forge a valid Merkle proof, they would need to:
1. Find a SHA256 hash collision (computational complexity approximately 2^128 — currently infeasible); or
2. Re-mine the block (changing `merkle_root` changes the block hash, requiring redo of the proof of work).

Therefore, as long as an SPV client can obtain block headers protected by honest proof of work, the security of a Merkle proof is equivalent to that of a full node.

**Block timestamp variance:** Miners' block timestamps are allowed to differ by approximately 2 hours, but this does not affect the security of Merkle verification (the Merkle tree does not depend on timestamps).

---

## Related Modules

- [`Block`](block.md) — `Block::new()` internally calls `MerkleTree::new()` to compute `merkle_root`; `Block::verify_transaction_inclusion()` uses `get_proof()` and `verify_proof()`.
- [Advanced Modules](advanced.md) — The SPV module (`src/spv.rs`) is built on the Merkle API to implement a lightweight client.
