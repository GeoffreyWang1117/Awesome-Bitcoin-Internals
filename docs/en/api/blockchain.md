# Blockchain API

The blockchain module is the core of SimpleBTC, managing the entire state and operations of the blockchain.

## Data Structures

### `Blockchain`

```rust
pub struct Blockchain {
    pub chain: Vec<Block>,                      // The blockchain (list of blocks)
    pub difficulty: usize,                      // Mining difficulty
    pub pending_transactions: Vec<Transaction>, // Pending transaction pool
    pub utxo_set: UTXOSet,                     // UTXO set
    pub mining_reward: u64,                    // Mining reward
    pub indexer: TransactionIndexer,           // Transaction indexer
}
```

## Methods

### Initialization

#### `new`

```rust
pub fn new() -> Blockchain
```

Creates a new blockchain, automatically creating the genesis block.

**Initial parameters**:
- `difficulty: 3` - Mining difficulty (3 leading zeros)
- `mining_reward: 50` - Block reward (50 satoshi)
- Genesis block contains 100 satoshi sent to `genesis_address`

**Return value**: New blockchain instance

**Example**:
```rust
let mut blockchain = Blockchain::new();
println!("Blockchain initialized, current height: {}", blockchain.chain.len());
```

---

### Transaction Management

#### `create_transaction`

```rust
pub fn create_transaction(
    &self,
    from_wallet: &Wallet,
    to_address: String,
    amount: u64,
    fee: u64,
) -> Result<Transaction, String>
```

Creates a new transaction. Automatically selects UTXOs, builds inputs/outputs, and adds signatures.

**Parameters**:
- `from_wallet` - Sender's wallet (requires private key for signing)
- `to_address` - Recipient address
- `amount` - Transfer amount (satoshi)
- `fee` - Transaction fee (satoshi)

**Return value**:
- `Ok(Transaction)` - Transaction created successfully
- `Err(String)` - Error message

**Error cases**:
- `"Insufficient balance (including fee)"` - Not enough UTXOs
- `"UTXO does not exist"` - Referenced UTXO has already been spent
- `"Referenced transaction does not exist"` - Data inconsistency

**Workflow**:
1. Find available UTXOs for the sender
2. Select sufficient UTXOs (greedy algorithm)
3. Create transaction inputs (including signatures)
4. Create transaction outputs (recipient + change)
5. Compute transaction ID

**Example**:
```rust
// Basic usage
let tx = blockchain.create_transaction(
    &alice,
    bob.address.clone(),
    5000,  // transfer 5000 satoshi
    10,    // fee 10 satoshi
)?;

blockchain.add_transaction(tx)?;

// Check balance
let balance = blockchain.get_balance(&alice.address);
if balance < amount + fee {
    return Err("Insufficient balance".to_string());
}

// Batch creation
for i in 1..=10 {
    let tx = blockchain.create_transaction(
        &alice,
        recipients[i].clone(),
        1000,
        i as u64,  // different fees
    )?;
    blockchain.add_transaction(tx)?;
}
```

#### `add_transaction`

```rust
pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String>
```

Adds a transaction to the pending pool, waiting to be mined.

**Parameters**:
- `transaction` - The transaction to add

**Validation items**:
1. ✅ Transaction format is correct (`verify()`)
2. ✅ UTXOs referenced by inputs exist
3. ✅ Signatures are valid
4. ✅ Total inputs ≥ total outputs

**Return value**:
- `Ok(())` - Added successfully
- `Err(String)` - Reason for validation failure

**Example**:
```rust
let tx = blockchain.create_transaction(&alice, bob.address, 1000, 5)?;

match blockchain.add_transaction(tx) {
    Ok(_) => println!("✓ Transaction added to pending pool"),
    Err(e) => eprintln!("✗ Invalid transaction: {}", e),
}

// View number of pending transactions
println!("Pending: {} transactions", blockchain.pending_transactions.len());
```

---

### Mining

#### `mine_pending_transactions`

```rust
pub fn mine_pending_transactions(
    &mut self,
    miner_address: String
) -> Result<(), String>
```

Mining: packs pending transactions into a new block.

**Parameters**:
- `miner_address` - Miner address (receives reward)

**Mining flow**:
1. Check if there are pending transactions
2. Sort by fee rate, highest to lowest
3. Compute total fees
4. Create Coinbase transaction (reward + fees)
5. Build Merkle tree
6. Proof of work (adjust nonce to find a valid hash)
7. Validate all transactions in the block
8. Update UTXO set (atomic operation)
9. Add block to the chain
10. Clear the pending pool

**Return value**:
- `Ok(())` - Mining successful
- `Err(String)` - Error message

**Error cases**:
- `"No pending transactions"` - Pending pool is empty
- `"Block contains invalid transactions"` - Transaction validation failed
- `"UTXO update failed"` - Data inconsistency

**Performance**:
- Difficulty 3: ~0.001–0.1 seconds
- Difficulty 4: ~0.01–1 second
- Difficulty 5: ~0.1–10 seconds
- Difficulty 6+: Several seconds to several minutes

**Example**:
```rust
// Basic mining
blockchain.mine_pending_transactions(miner.address.clone())?;

// Mining loop (similar to a real miner)
loop {
    if blockchain.pending_transactions.is_empty() {
        println!("Waiting for new transactions...");
        std::thread::sleep(Duration::from_secs(1));
        continue;
    }

    println!("Starting mining...");
    let start = Instant::now();

    blockchain.mine_pending_transactions(miner.address.clone())?;

    let duration = start.elapsed();
    println!("✓ Mining successful! Time elapsed: {:?}", duration);

    // Check reward
    let reward = blockchain.get_balance(&miner.address);
    println!("Miner balance: {} satoshi", reward);
}
```

---

### Query Operations

#### `get_balance`

```rust
pub fn get_balance(&self, address: &str) -> u64
```

Queries the balance of an address.

**Parameters**:
- `address` - The address to query

**Return value**: Balance (satoshi)

**Computation method**: Traverse the UTXO set and sum all UTXOs for the address

**Example**:
```rust
let balance = blockchain.get_balance(&alice.address);
println!("Balance: {} satoshi", balance);
println!("Balance: {:.8} BTC", balance as f64 / 100_000_000.0);

// Batch query
let addresses = vec![alice.address, bob.address, charlie.address];
for addr in addresses {
    let bal = blockchain.get_balance(&addr);
    println!("{}: {}", &addr[..10], bal);
}
```

#### `is_valid`

```rust
pub fn is_valid(&self) -> bool
```

Validates the integrity of the entire blockchain.

**Validation items**:
1. ✅ Each block's hash is correct
2. ✅ Forward references are correct (`previous_hash` links)
3. ✅ Proof of work is valid (hash satisfies difficulty)
4. ✅ All transactions are valid

**Return value**:
- `true` - Blockchain is complete and valid
- `false` - Tampering or error detected

**Use cases**:
- Periodic integrity checks
- Validation after syncing nodes
- Detection of tampering attacks

**Example**:
```rust
// Periodic validation
if !blockchain.is_valid() {
    panic!("❌ Blockchain has been tampered with!");
}

// Detailed validation log
for (i, block) in blockchain.chain.iter().enumerate() {
    if block.hash != block.calculate_hash() {
        eprintln!("Block {} has invalid hash", i);
    }
    if !block.validate_transactions() {
        eprintln!("Block {} contains invalid transactions", i);
    }
}

if blockchain.is_valid() {
    println!("✅ Blockchain validation passed");
}
```

#### `print_chain`

```rust
pub fn print_chain(&self)
```

Prints detailed blockchain information (for debugging).

**Output includes**:
- Block index, timestamp, hash
- Previous block hash
- Nonce value
- Transaction list (ID, type, fee, inputs/outputs)

**Example**:
```rust
blockchain.print_chain();

// Example output:
// ========== Blockchain Info ==========
//
// --- Block #0 ---
// Timestamp: 1703001234
// Hash: 0003ab4f9c2d...
// Previous hash: 0
// Nonce: 1247
// Transaction count: 1
//   Transaction #0: abc123...
//     Type: Coinbase (mining reward)
//     Inputs: 1
//     Outputs: 1
//       Output 0: 100 -> genesis_address
// ...
```

---

## Usage Examples

### Complete Blockchain Demo

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn main() -> Result<(), String> {
    println!("=== SimpleBTC Blockchain Demo ===\n");

    // 1. Initialize
    let mut blockchain = Blockchain::new();
    println!("✓ Blockchain created (genesis block)\n");

    // 2. Create participants
    let alice = Wallet::new();
    let bob = Wallet::new();
    let miner = Wallet::new();

    println!("✓ Created 3 wallets");
    println!("  Alice: {}", &alice.address[..16]);
    println!("  Bob:   {}", &bob.address[..16]);
    println!("  Miner: {}\n", &miner.address[..16]);

    // 3. Alice receives initial funds
    let init_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        alice.address.clone(),
        10000,
        0,
    )?;
    blockchain.add_transaction(init_tx)?;
    blockchain.mine_pending_transactions(miner.address.clone())?;

    println!("✓ Block #1 mined");
    println!("  Alice balance: {}\n", blockchain.get_balance(&alice.address));

    // 4. Multiple transactions
    println!("Creating 5 transactions (different fees)...");
    for i in 1..=5 {
        let tx = blockchain.create_transaction(
            &alice,
            bob.address.clone(),
            100 * i,
            i as u64,  // incrementing fees
        )?;
        blockchain.add_transaction(tx)?;
        println!("  Transaction #{}: {} sat, fee rate: {} sat/byte",
            i, 100 * i, i);
    }

    // 5. Mine (transactions sorted by fee rate)
    println!("\nStarting mining...");
    blockchain.mine_pending_transactions(miner.address.clone())?;
    println!("✓ Block #2 mined\n");

    // 6. Final balances
    println!("=== Final Balances ===");
    println!("Alice: {} satoshi", blockchain.get_balance(&alice.address));
    println!("Bob:   {} satoshi", blockchain.get_balance(&bob.address));
    println!("Miner: {} satoshi", blockchain.get_balance(&miner.address));

    // 7. Validate blockchain
    println!("\n=== Validate Blockchain ===");
    if blockchain.is_valid() {
        println!("✅ Blockchain integrity validation passed");
    } else {
        println!("❌ Blockchain validation failed");
    }

    // 8. Print details
    println!("\n=== Blockchain Details ===");
    blockchain.print_chain();

    Ok(())
}
```

### Fee Priority Demo

```rust
fn fee_priority_demo() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let alice = Wallet::new();
    let recipients: Vec<_> = (0..3).map(|_| Wallet::new()).collect();

    // Initialize Alice's balance
    setup_balance(&mut blockchain, &alice, 10000)?;

    // Create transactions with different fee rates
    let txs = vec![
        ("Slow", 1000, 1),   // 1 sat/byte
        ("Fast", 1000, 50),  // 50 sat/byte
        ("Medium", 1000, 10),  // 10 sat/byte
    ];

    println!("Adding transactions:");
    for (i, (name, amount, fee)) in txs.iter().enumerate() {
        let tx = blockchain.create_transaction(
            &alice,
            recipients[i].address.clone(),
            *amount,
            *fee,
        )?;
        println!("  {}: {} sat, fee rate {} sat/byte", name, amount, fee);
        blockchain.add_transaction(tx)?;
    }

    println!("\nMining (auto-sorted by fee rate)...");
    blockchain.mine_pending_transactions(recipients[0].address.clone())?;

    // View transaction order in the latest block
    let latest_block = blockchain.chain.last().unwrap();
    println!("\nTransaction order in block:");
    for (i, tx) in latest_block.transactions.iter().skip(1).enumerate() {
        println!("  #{}: fee rate {:.2} sat/byte",
            i + 1, tx.fee_rate());
    }

    Ok(())
}
```

### Monitoring Blockchain State

```rust
fn blockchain_monitor(blockchain: &Blockchain) {
    println!("=== Blockchain State ===");
    println!("Block height: {}", blockchain.chain.len());
    println!("Difficulty: {} ({} leading zeros)",
        blockchain.difficulty, blockchain.difficulty);
    println!("Mining reward: {} satoshi", blockchain.mining_reward);
    println!("Pending transactions: {}", blockchain.pending_transactions.len());

    // UTXO statistics
    let total_utxos = blockchain.utxo_set.utxos
        .values()
        .map(|v| v.len())
        .sum::<usize>();
    println!("Total UTXOs: {}", total_utxos);

    // Latest block info
    if let Some(latest) = blockchain.chain.last() {
        println!("\nLatest block:");
        println!("  Hash: {}", &latest.hash[..16]);
        println!("  Transactions: {}", latest.transactions.len());
        println!("  Merkle root: {}", &latest.merkle_root[..16]);
    }
}
```

## Configuration Recommendations

### Mining Difficulty

```rust
// Demo environment
blockchain.difficulty = 3;  // Fast (milliseconds)

// Test environment
blockchain.difficulty = 4;  // Moderate (seconds)

// Production environment
blockchain.difficulty = 6;  // Secure (minutes)
```

### Block Reward

```rust
// Bitcoin-style (gradual halving)
let halving_interval = 210000;
let halvings = blockchain.chain.len() / halving_interval;
blockchain.mining_reward = 50 >> halvings;  // 50, 25, 12.5, ...
```

## Performance Optimization

### 1. UTXO Indexing

Use `indexer` to speed up queries:

```rust
// Find all transactions for an address
let txs = blockchain.indexer.get_transactions_by_address(&address);

// Find a specific transaction
let tx = blockchain.indexer.get_transaction(&txid);
```

### 2. Batch Operations

```rust
// Batch add transactions
for tx in transactions {
    blockchain.add_transaction(tx)?;
}
// Mine all at once
blockchain.mine_pending_transactions(miner.address)?;
```

### 3. Parallel Validation

```rust
use rayon::prelude::*;

// Parallel validation of all transactions (requires rayon dependency)
let all_valid = blockchain.pending_transactions
    .par_iter()
    .all(|tx| tx.verify());
```

## References

- [Transaction API](./transaction.md)
- [Block API](./block.md)
- [UTXO API](./utxo.md)
- [Real-World Examples](../examples/enterprise-multisig.md)

---

[Back to API Index](./core.md)
