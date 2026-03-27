# Merkle Tree and SPV Verification

Merkle trees (hash trees) and Simplified Payment Verification (SPV) are the technical foundation for Bitcoin's lightweight clients. They allow a mobile wallet to securely verify transactions with only a few MB of storage, without needing to download the full blockchain of over 500 GB.

---

## What is a Merkle Tree?

A Merkle tree is a **binary hash tree** invented by computer scientist Ralph Merkle in 1979. Its core idea is: by recursively hashing data, any quantity of data can be compressed into a fixed-length "fingerprint" (the root hash).

In Bitcoin, all the transactions included in each block are organized into a Merkle tree. The tree's root hash (Merkle Root) is stored in the block header, protected by proof of work (PoW). Any tampering with the transaction data will cause the root hash to change, invalidating that block and all subsequent blocks.

### Tree Structure Diagram (4 transactions)

```
                    ┌─────────────┐
                    │  Root Hash  │
                    │ hash(H12+H34)│
                    └──────┬──────┘
                   ┌───────┴───────┐
            ┌──────┴──────┐  ┌─────┴──────┐
            │     H12     │  │     H34    │
            │ hash(H1+H2) │  │ hash(H3+H4)│
            └──────┬──────┘  └─────┬──────┘
          ┌────────┴───┐     ┌─────┴───┐
       ┌──┴──┐     ┌───┴──┐ ┌───┴──┐ ┌──┴───┐
       │ H1  │     │  H2  │ │  H3  │ │  H4  │
       │hash │     │ hash │ │ hash │ │ hash │
       │(tx1)│     │(tx2) │ │(tx3) │ │(tx4) │
       └──┬──┘     └──┬───┘ └──┬───┘ └──┬───┘
          │           │        │         │
         tx1         tx2      tx3       tx4
      (Transaction 1) (Transaction 2) (Transaction 3) (Transaction 4)
```

---

## Construction Process

Construction follows a **bottom-up** approach in two phases:

### Phase 1: Build the Leaf Layer

The raw data of each transaction is SHA-256 hashed to become a leaf node:

```
H1 = SHA256(tx1_data)
H2 = SHA256(tx2_data)
H3 = SHA256(tx3_data)
H4 = SHA256(tx4_data)
```

**Odd number handling**: If the number of transactions is odd, the last transaction is duplicated to make the layer count even. This is the standard practice specified by the Bitcoin protocol.

### Phase 2: Merge Layer by Layer to the Root

Each pair of adjacent nodes' hashes is concatenated and then hashed to produce the parent node:

```
H12 = SHA256(H1 + H2)
H34 = SHA256(H3 + H4)
Root = SHA256(H12 + H34)
```

This process is repeated until only one node remains, which is the **Merkle root**.

---

## Implementation in SimpleBTC

### Node Structure: `MerkleNode`

```rust
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,                   // The node's hash value
    pub left: Option<Box<MerkleNode>>,  // Left child node (only for internal nodes)
    pub right: Option<Box<MerkleNode>>, // Right child node (only for internal nodes)
}
```

- **Leaf nodes**: Both `left` and `right` are `None`; `hash` is the SHA-256 value of the transaction data
- **Internal nodes**: Have left and right child nodes; `hash` is `SHA256(left.hash + right.hash)`
- **Root node**: The top node of the tree; the final Merkle Root

Two factory methods for creating nodes:

```rust
// Leaf node: hash the raw data directly
let leaf = MerkleNode::new_leaf("tx_data_string");

// Internal node: merge two child nodes
let parent = MerkleNode::new_internal(left_node, right_node);
```

### Tree Structure: `MerkleTree`

```rust
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>, // Tree root node
    pub leaves: Vec<String>,      // List of original transaction hashes
}
```

---

### Building the Merkle Tree: `MerkleTree::new`

```rust
pub fn new(transactions: &[String]) -> Self
```

Accepts a list of transaction IDs (strings) and automatically builds the complete Merkle tree:

```rust
use bitcoin_simulation::merkle::MerkleTree;

let txs = vec![
    "tx1_hash".to_string(),
    "tx2_hash".to_string(),
    "tx3_hash".to_string(),
    "tx4_hash".to_string(),
];

let tree = MerkleTree::new(&txs);
let root = tree.get_root_hash();
println!("Merkle Root: {}", root);
// Output: a 64-character hexadecimal hash string
```

Key steps in the internal implementation:

```rust
// 1. Pad odd count
if !leaves.len().is_multiple_of(2) {
    leaves.push(leaves.last().unwrap().clone());
}

// 2. Build the leaf node layer
let mut nodes: Vec<MerkleNode> = leaves.iter()
    .map(|tx| MerkleNode::new_leaf(tx))
    .collect();

// 3. Merge layer by layer from bottom to top
while nodes.len() > 1 {
    let mut next_level = Vec::new();
    for i in (0..nodes.len()).step_by(2) {
        let left = nodes[i].clone();
        let right = nodes[i + 1].clone(); // Even count already guaranteed
        next_level.push(MerkleNode::new_internal(left, right));
    }
    nodes = next_level;
}
```

---

### Generating a Merkle Proof: `get_proof`

```rust
pub fn get_proof(&self, tx_hash: &str) -> Option<Vec<String>>
```

Generates a **Merkle proof** (also called a "Merkle path") for the specified transaction. This proof contains the hashes of all **sibling nodes** along the path from that transaction's leaf node to the root node.

```rust
// Generate a proof for tx1
let proof = tree.get_proof("tx1_hash").unwrap();
// proof = [H2, H34]  ← list of sibling hashes needed for verification
```

**Diagram: proof needed to verify tx1**

```
                    ┌────────────┐
                    │    Root    │ ← known (stored in block header)
                    └─────┬──────┘
               ┌──────────┴──────────┐
        ┌──────┴──────┐       ┌──────┴──────┐
        │     H12     │       │ ★ H34 ★    │ ← proof element[1]
        └──────┬──────┘       └─────────────┘
       ┌───────┴───────┐
    ┌──┴──┐       ┌────┴──┐
    │  H1 │       │★ H2 ★│ ← proof element[0]
    └──┬──┘       └───────┘
       │
     [tx1]  ← the transaction to verify (known)
```

The verifier only needs two hashes `[H2, H34]` (log₂4 = 2 steps), without needing to know the contents of tx2, tx3, or tx4.

---

### Verifying a Merkle Proof: `verify_proof`

```rust
pub fn verify_proof(
    tx_hash: &str,      // Hash of the transaction to verify
    proof: &[String],   // Merkle proof (list of sibling hashes)
    root_hash: &str,    // Merkle root from the block header
    index: usize,       // The transaction's index position in the block
) -> bool
```

This is a **static method**; verification is possible without holding the complete Merkle tree. SPV clients use exactly this method to verify transactions.

```rust
// Known: tx1 is in the block at index 0; Merkle Root comes from the block header
let is_valid = MerkleTree::verify_proof(
    "tx1_hash",
    &proof,      // [H2, H34]
    &root_hash,  // from block header, protected by PoW
    0,           // tx1 is transaction #0
);
println!("Transaction verification result: {}", is_valid); // true
```

**Verification algorithm steps** (using tx1, index=0 as an example):

```
Step 1: current_hash = SHA256("tx1_hash")      → get H1
        index=0 (even), H1 is on the left
        combined = H1 + proof[0] (H2)
        current_hash = SHA256(H1 + H2)         → get H12
        index = 0 / 2 = 0

Step 2: index=0 (even), H12 is on the left
        combined = H12 + proof[1] (H34)
        current_hash = SHA256(H12 + H34)       → get the computed Root

Verify: computed Root == merkle_root in the block header?
```

Source code implementation:

```rust
pub fn verify_proof(tx_hash: &str, proof: &[String], root_hash: &str, index: usize) -> bool {
    let mut current_hash = MerkleNode::hash_data(tx_hash);
    let mut current_index = index;

    for sibling_hash in proof {
        let combined = if current_index.is_multiple_of(2) {
            // Current node is on the left, sibling is on the right
            format!("{}{}", current_hash, sibling_hash)
        } else {
            // Current node is on the right, sibling is on the left
            format!("{}{}", sibling_hash, current_hash)
        };
        current_hash = MerkleNode::hash_data(&combined);
        current_index /= 2;
    }

    current_hash == root_hash
}
```

---

## SPV Light Clients

### SPV Concept

SPV (Simplified Payment Verification) was proposed by Satoshi Nakamoto in [Section 8 of the Bitcoin whitepaper](https://bitcoin.org/bitcoin.pdf). Its core idea is: **a light client does not need to verify all transactions; it only needs to trust the longest proof-of-work chain and use Merkle proofs to verify transactions relevant to itself**.

| Feature | Full Node | SPV Node |
|---------|-----------|----------|
| Storage requirement | 400+ GB (full blockchain) | ~5 MB (block headers only) |
| Bandwidth consumption | Full blocks (1–4 MB/block) | Block headers only (80 bytes/block) |
| Verification scope | All transactions | Only transactions relevant to itself |
| Security level | Highest (fully self-verified) | Relies on PoW; trusts miner honesty |
| Suitable for | Mining pools, exchanges, full nodes | Mobile wallets, embedded devices |

### SPV Implementation in SimpleBTC

#### Block Header Structure: `BlockHeader`

SPV clients only download and store block headers, not the transaction body:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u32,           // Block height
    pub hash: String,          // Block hash
    pub previous_hash: String, // Previous block hash (ensures chain structure)
    pub merkle_root: String,   // Merkle root (32 bytes, used to verify transactions)
    pub timestamp: u64,        // Timestamp
    pub bits: u32,             // Difficulty target
    pub nonce: u64,            // Proof-of-work nonce
}
```

Each block header is a fixed 80 bytes. Bitcoin currently has approximately 830,000 blocks, so the total size of all block headers is about 66 MB — a tremendous saving compared to the full blockchain of 600+ GB.

#### SPV Client: `SPVClient`

```rust
pub struct SPVClient {
    headers: Vec<BlockHeader>,               // Block header chain
    header_index: HashMap<String, BlockHeader>, // hash → header for fast lookup
    verified_transactions: HashMap<String, (String, bool)>, // txid → (block_hash, verified result)
    chain_tip: Option<String>,               // Current latest block hash
    total_work: u64,                         // Accumulated work
}
```

---

### SPV Workflow

#### Step 1: Sync Block Headers

```rust
use bitcoin_simulation::spv::SPVClient;

let mut client = SPVClient::new();

// Fetch blocks from a full node and extract their headers
let blocks = /* fetched from the P2P network */;
client.sync_from_blocks(&blocks).unwrap();

println!("Synced {} block headers", client.get_height());
println!("Storage used: {} bytes", client.estimate_storage_size());
// 1000 block headers require only 80,000 bytes (about 78 KB)
```

Headers can also be added one at a time:

```rust
use bitcoin_simulation::spv::BlockHeader;

let header = BlockHeader {
    height: 0,
    hash: "genesis_hash".to_string(),
    previous_hash: "0000...".to_string(),
    merkle_root: "merkle_root_hash".to_string(),
    timestamp: 1231006505,
    bits: 0x1d00ffff,
    nonce: 2083236893,
};

client.add_block_header(header).unwrap();
```

The **continuity** of the block header chain is automatically verified by `add_block_header`: the new header's `previous_hash` must match the `hash` of the previous header; otherwise it is rejected:

```rust
// Attempting to add a non-continuous block header returns an error
let bad_header = BlockHeader {
    height: 1,
    hash: "block_1".to_string(),
    previous_hash: "wrong_hash".to_string(), // does not match!
    // ...
};
let result = client.add_block_header(bad_header);
assert!(result.is_err()); // rejected
```

#### Step 2: Verify Transaction Inclusion

When a user receives a payment, they need to verify that this transaction has indeed been packed into a block:

```rust
// Suppose a merchant receives a payment notification: tx_id is at position 0 in block_hash
let tx_id = "payment_tx_hash";
let block_hash = "some_block_hash";

// Request a Merkle proof from a full node (in practice, this is done via P2P protocol)
let proof = vec!["sibling_hash_1".to_string(), "sibling_hash_2".to_string()];
let tx_index = 0; // Position of the transaction in the block

let is_valid = client.verify_transaction(tx_id, &proof, block_hash, tx_index).unwrap();
if is_valid {
    println!("Payment confirmed! Transaction {} is in the block", tx_id);
} else {
    println!("Verification failed; the transaction may not be in that block");
}
```

#### Step 3: Check Historical Verification Results

```rust
// Check whether a transaction has already passed SPV verification
if let Some(verified) = client.is_transaction_verified(tx_id) {
    if verified {
        println!("This transaction has been verified");
    }
}

// Get SPV statistics
let stats = client.get_stats();
println!("Block header count: {}", stats.header_count);
println!("Storage size: {} bytes", stats.storage_size);
println!("Verified transaction count: {}", stats.verified_tx_count);
```

---

## Complete Example: Build a Tree and Perform SPV Verification

```rust
use bitcoin_simulation::merkle::MerkleTree;
use bitcoin_simulation::spv::{SPVClient, BlockHeader};

fn main() {
    // 1. Assume a block contains 4 transactions
    let transactions = vec![
        "tx1".to_string(),
        "tx2".to_string(),
        "tx3".to_string(),
        "tx4".to_string(),
    ];

    // 2. Build the Merkle tree (what a full node does)
    let tree = MerkleTree::new(&transactions);
    let merkle_root = tree.get_root_hash();
    println!("Merkle Root: {}", merkle_root);

    // 3. Generate a proof for tx1 (full node generates it at the SPV client's request)
    let proof = tree.get_proof("tx1").unwrap();
    println!("Merkle proof for tx1 contains {} hashes", proof.len());

    // 4. SPV client verification (knows only the block header and proof, not the other transactions)
    let mut spv = SPVClient::new();
    let header = BlockHeader {
        height: 0,
        hash: "block_0".to_string(),
        previous_hash: "0".to_string(),
        merkle_root: merkle_root.clone(),
        timestamp: 1700000000,
        bits: 0,
        nonce: 42,
    };
    spv.add_block_header(header).unwrap();

    let valid = spv.verify_transaction("tx1", &proof, "block_0", 0).unwrap();
    println!("SPV verification result: {}", valid); // true

    // 5. Verify directly using the static method (no SPVClient needed)
    let valid2 = MerkleTree::verify_proof("tx1", &proof, &merkle_root, 0);
    println!("Static verification result: {}", valid2); // true
}
```

---

## Why is SPV Verification Secure?

An attacker cannot forge a Merkle proof for two reasons:

1. **SHA-256 collision resistance**: Finding two different inputs that produce the same hash is computationally infeasible (requires approximately 2¹²⁸ hash operations).
2. **PoW protection**: The `merkle_root` is stored in the block header, which is protected by proof of work. To forge a block header containing a fake `merkle_root`, an attacker would need to redo the mining work for that block and all subsequent blocks, which is computationally extremely difficult (the "longest chain rule").

The only trust assumption of SPV is: **honest miners control more than 51% of the hashrate**. As long as this assumption holds, an attacker cannot deceive an SPV client at any practical cost.

---

## Summary

| Component | Role |
|-----------|------|
| `MerkleNode` | Basic unit of the Merkle tree; stores the hash value and references to child nodes |
| `MerkleTree::new` | Builds the complete Merkle tree bottom-up from a list of transactions |
| `MerkleTree::get_proof` | Generates an O(log n)-sized Merkle proof for a specified transaction |
| `MerkleTree::verify_proof` | Verifies a transaction using a proof + root hash; O(log n) time complexity |
| `BlockHeader` | Block header; 80 bytes; contains the Merkle Root |
| `SPVClient` | Light client; downloads only block headers and verifies transactions using Merkle proofs |
