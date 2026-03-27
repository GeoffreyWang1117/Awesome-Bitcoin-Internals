# Transaction Priority

When the network is congested, thousands of pending transactions may accumulate in the mempool. Miners can only pack approximately 1 MB of data into each block, so a priority mechanism is needed to decide which transactions are confirmed first. This chapter covers how transaction priority is calculated in SimpleBTC, the mempool sorting logic, and the fee recommendation strategy.

---

## Core Concept: Fee Rate

**Fee rate** is the most important metric for measuring transaction priority:

```
Fee Rate (sat/byte) = Fee (satoshi) / Transaction Size (bytes)
```

Miners preferentially select high-fee-rate transactions to pack into blocks, because doing so maximizes fee income for the same block space.

### Why Use Fee Rate Instead of Absolute Fee?

A complex transaction with 10 inputs may pay a fee of 1000 sat but occupies 900 bytes, giving a fee rate of about 1.1 sat/byte. A simple transaction with only 1 input pays 200 sat but occupies only 192 bytes, giving a fee rate of about 1.04 sat/byte. From the miner's perspective, both are roughly equivalent. If only the absolute fee is considered, the former would be incorrectly prioritized, wasting block space.

---

## Mempool Structure

SimpleBTC's `Mempool` uses a **dual-index structure** to support efficient priority sorting:

```rust
pub struct Mempool {
    // Primary storage: txid → mempool entry
    transactions: HashMap<String, MempoolEntry>,

    // Fee rate index: fee_rate → set of txids (BTreeMap is automatically sorted; efficient iteration)
    fee_index: BTreeMap<ordered_float::NotNan<f64>, HashSet<String>>,

    // UTXO index: used for double-spend detection
    utxo_index: HashMap<String, String>,

    // Capacity control
    max_size: usize,       // Maximum bytes (default 300 MB)
    current_size: usize,   // Currently used bytes
    min_fee_rate: f64,     // Minimum accepted fee rate (default 1.0 sat/byte)
    max_age: u64,          // Maximum retention time (default 72 hours)
}
```

`BTreeMap` (a balanced binary search tree) is the key: it automatically sorts by fee rate, making the operation "get the top N transactions by fee rate" a simple reverse iteration from the tail, with time complexity O(N).

### Mempool Entry: `MempoolEntry`

```rust
pub struct MempoolEntry {
    pub transaction: Transaction,  // Full transaction data
    pub added_time: u64,           // Time added (Unix timestamp)
    pub size: usize,               // Estimated byte size
    pub fee_rate: f64,             // Calculated fee rate (sat/byte)
    pub replaceable: bool,         // Whether RBF replacement is supported
}
```

The fee rate is calculated immediately when a `MempoolEntry` is created and cached, avoiding repeated calculations:

```rust
impl MempoolEntry {
    pub fn new(transaction: Transaction, size: usize) -> Self {
        let fee_rate = if size > 0 {
            transaction.fee as f64 / size as f64
        } else {
            0.0
        };
        // ...
    }
}
```

---

## Transaction Size Estimation

SimpleBTC uses a simplified formula to estimate transaction byte size:

```rust
fn estimate_tx_size(&self, tx: &Transaction) -> usize {
    let base = 10;               // Fixed overhead (version number, lock time, etc.)
    let inputs_size = tx.inputs.len() * 148;   // About 148 bytes per input
    let outputs_size = tx.outputs.len() * 34;  // About 34 bytes per output
    base + inputs_size + outputs_size
}
```

**Real Bitcoin transaction size reference** (native SegWit, P2WPKH format):

| Transaction Type | Inputs | Outputs | Estimated Size |
|-----------------|--------|---------|----------------|
| Simple transfer | 1 | 2 | ~192 bytes |
| Consolidate multiple UTXOs | 5 | 2 | ~898 bytes |
| Batch payment | 1 | 10 | ~388 bytes |

---

## Transaction Addition and Validation Flow

When `mempool.add_transaction(tx)` is called, the following checks are executed in order internally:

```
Transaction arrives at mempool
      │
      ▼
① Already exists? ──Yes──► Reject (duplicate transaction)
      │No
      ▼
② Basic security validation (format, signature, etc.)
      │
      ▼
③ Double-spend detection: has any input already been spent by another mempool transaction?
      │
      ├─ Yes, and old transaction supports RBF and new fee is higher ──► Trigger RBF replacement; continue
      │
      └─ Yes, but RBF conditions not met ──► Reject (double-spend attack)
      │No
      ▼
④ Estimate size; calculate fee rate
      │
      ▼
⑤ Fee rate ≥ min_fee_rate? ──No──► Reject (fee rate too low)
      │Yes
      ▼
⑥ Mempool full? ──Yes──► Trigger eviction of low-fee-rate transactions
      │
      ▼
⑦ Add to transactions, fee_index, utxo_index
      │
      ▼
     Success
```

```rust
// Example: add a transaction to the mempool
let mut mempool = Mempool::default(); // 300 MB limit, 1 sat/byte minimum fee rate

let tx = Transaction::new(inputs, outputs, 0, 200); // 200 sat fee
match mempool.add_transaction(tx) {
    Ok(()) => println!("Transaction entered mempool"),
    Err(e) => println!("Rejected reason: {}", e),
}
```

---

## Priority Sorting and Block Packing

### Get Top N by Fee Rate: `get_top_transactions`

```rust
pub fn get_top_transactions(&self, max_count: usize) -> Vec<Transaction>
```

By reverse-iterating `fee_index` (BTreeMap from high to low), the highest-fee-rate transactions are quickly retrieved:

```rust
// Iterate from high fee rate to low
for (_fee_rate, txids) in self.fee_index.iter().rev() {
    for txid in txids {
        if let Some(entry) = self.transactions.get(txid) {
            result.push(entry.transaction.clone());
            if result.len() >= max_count {
                return result;
            }
        }
    }
}
```

```rust
// Usage: miner wants to preview the top 10 best transactions
let top_txs = mempool.get_top_transactions(10);
for tx in &top_txs {
    println!("txid: {}, fee: {} sat", tx.id, tx.fee);
}
```

### Pack by Block Size Limit: `get_transactions_for_block`

```rust
pub fn get_transactions_for_block(&self, max_size: usize) -> Vec<Transaction>
```

A more practical block-packing function. It also selects transactions from high fee rate to low, but additionally checks that the accumulated size does not exceed `max_size` bytes:

```rust
pub fn get_transactions_for_block(&self, max_size: usize) -> Vec<Transaction> {
    let mut result = Vec::new();
    let mut total_size = 0;

    for (_fee_rate, txids) in self.fee_index.iter().rev() {
        for txid in txids {
            if let Some(entry) = self.transactions.get(txid) {
                if total_size + entry.size <= max_size {
                    result.push(entry.transaction.clone());
                    total_size += entry.size;
                }
            }
        }
    }
    result
}
```

```rust
// Usage: pack transactions for a new block (Bitcoin block limit is about 1 MB = 1,000,000 bytes)
let block_txs = mempool.get_transactions_for_block(1_000_000);
println!("Selected {} transactions for packing", block_txs.len());
```

---

## Composite Priority Score

SimpleBTC provides `TxPriorityCalculator` in `src/advanced_tx.rs`, implementing more refined priority calculation.

### Basic Fee Rate Calculation

```rust
pub fn calculate_fee_rate(fee: u64, size: usize) -> f64 {
    if size == 0 { return 0.0; }
    fee as f64 / size as f64
}
```

```rust
// 200 sat fee, transaction size 192 bytes
let fee_rate = TxPriorityCalculator::calculate_fee_rate(200, 192);
println!("Fee rate: {:.2} sat/byte", fee_rate); // ~1.04 sat/byte
```

### Coin Age Priority

In early Bitcoin (before SegWit), "coin age" was also considered: the UTXO's value multiplied by the number of blocks it has been waiting, divided by the transaction size:

```rust
/// Priority = (input value × input confirmations) / transaction size
pub fn calculate_priority(
    input_value: u64,  // Total value of inputs (satoshi)
    input_age: u32,    // Number of blocks the input UTXO has been confirmed
    tx_size: usize,
) -> f64 {
    (input_value as f64 * input_age as f64) / tx_size as f64
}
```

```rust
// Example: input value 1 BTC = 100,000,000 sat, confirmed 100 blocks, transaction size 200 bytes
let priority = TxPriorityCalculator::calculate_priority(100_000_000, 100, 200);
println!("Coin age priority: {:.0}", priority); // 50,000,000
```

> **Historical context**: Bitcoin Core removed coin-age-based free transaction priority in version 0.12 (2016), because low-fee transactions significantly slowed down block packing. In the modern network, fee rate is the only priority metric that actually matters.

### Composite Score Formula: 70% Fee Rate + 30% Coin Age

```rust
/// Composite score = fee_rate × 0.7 + priority × 0.001 × 0.3
pub fn calculate_score(fee_rate: f64, priority: f64) -> f64 {
    fee_rate * 0.7 + priority * 0.001 * 0.3
}
```

Design rationale of this weighted formula:
- **70% weight for fee rate**: ensures miner revenue maximization; high-fee-rate transactions still take priority
- **30% weight for coin age** (scaled by 0.001): gives a "bonus" to long-waiting transactions, preventing low-fee-rate old UTXOs from never being confirmed

```rust
// Full scoring example
let fee_rate = TxPriorityCalculator::calculate_fee_rate(500, 200); // 2.5 sat/byte
let priority = TxPriorityCalculator::calculate_priority(50_000_000, 10, 200); // 2,500,000
let score = TxPriorityCalculator::calculate_score(fee_rate, priority);
println!("Composite score: {:.4}", score);
// score = 2.5 * 0.7 + 2_500_000 * 0.001 * 0.3 = 1.75 + 750 = 751.75
```

---

## Fee Recommendations

`TxPriorityCalculator::recommend_fee` returns a suggested fee based on urgency:

```rust
pub enum FeeUrgency {
    Low,    // Low priority: confirmed within a few hours
    Medium, // Medium priority: confirmed within 30–60 minutes
    High,   // High priority: 10–20 minutes (approximately 1–2 blocks)
    Urgent, // Urgent: next block (highest priority)
}

pub fn recommend_fee(tx_size: usize, urgency: FeeUrgency) -> u64 {
    let sat_per_byte = match urgency {
        FeeUrgency::Low    => 1.0,   // 1 sat/byte
        FeeUrgency::Medium => 5.0,   // 5 sat/byte
        FeeUrgency::High   => 20.0,  // 20 sat/byte
        FeeUrgency::Urgent => 50.0,  // 50 sat/byte
    };
    (tx_size as f64 * sat_per_byte) as u64
}
```

```rust
use bitcoin_simulation::advanced_tx::{TxPriorityCalculator, FeeUrgency};

// Estimate the suggested fee for a standard transaction (1 input, 2 outputs)
let tx_size = 10 + 1 * 148 + 2 * 34; // = 226 bytes

let low_fee    = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Low);
let medium_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Medium);
let high_fee   = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::High);
let urgent_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Urgent);

println!("Low priority:    {} sat ({} sat/byte)", low_fee,    1);  // 226 sat
println!("Medium priority: {} sat ({} sat/byte)", medium_fee, 5);  // 1130 sat
println!("High priority:   {} sat ({} sat/byte)", high_fee,   20); // 4520 sat
println!("Urgent:          {} sat ({} sat/byte)", urgent_fee, 50); // 11300 sat
```

**Real Bitcoin network fee rate reference** (2024 data, BTC/USD = $60,000):

| Urgency | Typical Fee Rate | Approximate USD (226-byte transaction) |
|---------|-----------------|----------------------------------------|
| Low | 1–3 sat/byte | $0.14 – $0.41 |
| Medium | 5–15 sat/byte | $0.68 – $2.03 |
| High | 20–50 sat/byte | $2.71 – $6.78 |
| Urgent | 50–200 sat/byte | $6.78 – $27.1 |

> **Note**: Actual fee rates are heavily influenced by network congestion. At the peak of the 2017 bull market, some users paid fees exceeding $50 for fast confirmation.

---

## Low-Fee-Rate Transaction Eviction

When the mempool reaches its capacity limit, it automatically evicts the lowest-fee-rate transactions to free up space:

```rust
fn evict_low_fee_transactions(&mut self, needed_size: usize) -> Result<()> {
    let mut freed_size = 0;
    let mut to_remove = Vec::new();

    // Iterate from [low fee rate to high fee rate] (BTreeMap forward iteration)
    for (_fee_rate, txids) in self.fee_index.iter() {
        for txid in txids {
            if let Some(entry) = self.transactions.get(txid) {
                to_remove.push(txid.clone());
                freed_size += entry.size;
                if freed_size >= needed_size {
                    break;
                }
            }
        }
        if freed_size >= needed_size { break; }
    }

    // Execute eviction
    for txid in &to_remove {
        self.remove_transaction(txid)?;
    }
    Ok(())
}
```

This design ensures the mempool always maintains the subset of "highest-fee-rate" transactions; low-fee-rate transactions are naturally eliminated through competition.

```rust
// Create a very small mempool to demonstrate eviction behavior
let mut mempool = Mempool::new(1000, 1.0); // Only 1 KB capacity

// Add multiple transactions; when over 1 KB, low-fee-rate ones are evicted
for i in 1..=10 {
    let tx = create_tx_with_fee(i * 100); // fee: 100, 200, ..., 1000
    let _ = mempool.add_transaction(tx); // Low-fee-rate ones may be evicted
}
```

---

## Expired Transaction Cleanup

By default, transactions that have been waiting in the mempool for more than 72 hours are cleared:

```rust
// Call periodically (e.g., once per hour)
let expired_count = mempool.clear_expired();
if expired_count > 0 {
    println!("Cleared {} expired transactions", expired_count);
}
```

---

## Mempool Statistics

```rust
let stats = mempool.get_stats();
println!("Pending transaction count: {}", stats.tx_count);
println!("Mempool size:              {} / {} bytes", stats.total_size, stats.max_size);
println!("Total pending fees:        {} sat", stats.total_fees);
println!("Average fee rate:          {:.2} sat/byte", stats.avg_fee_rate);
println!("Minimum accepted fee rate: {:.2} sat/byte", stats.min_fee_rate);
```

---

## Replace-By-Fee (RBF)

RBF (BIP125) allows users to replace an old transaction in the mempool with a new one carrying a higher fee. In SimpleBTC, `RBFManager` in `advanced_tx.rs` manages replaceable transactions:

```rust
use bitcoin_simulation::advanced_tx::RBFManager;

let mut rbf = RBFManager::new();

// Mark a transaction as replaceable (set sequence < 0xFFFFFFFE when sending)
rbf.mark_replaceable("original_tx_id");

// Later, replace the old transaction with a new one bearing a higher fee
let can_replace = rbf.can_replace(&old_tx, &new_tx);
match can_replace {
    Ok(()) => println!("RBF replacement successful"),
    Err(reason) => println!("Replacement rejected: {}", reason),
}
```

RBF replacement conditions (verified by `can_replace`):
1. The old transaction must have been marked as replaceable (`replaceable = true`)
2. The number of inputs in the new and old transactions must be the same, referencing the same UTXOs
3. The new transaction's fee must be strictly higher than the old transaction's
4. The fee increment must be at least the size of the old transaction (approximately 1 sat/byte)

---

## Summary

SimpleBTC's transaction priority system consists of three layers:

| Layer | Component | Role |
|-------|-----------|------|
| Mempool sorting | `Mempool` + `BTreeMap<fee_rate>` | Automatically maintains a sorted queue by fee rate |
| Priority calculation | `TxPriorityCalculator` | Fee rate, coin age, composite score |
| Fee recommendation | `FeeUrgency` + `recommend_fee` | Recommends a reasonable fee by urgency level |

Core formula recap:
```
Fee Rate (sat/byte)  = fee / transaction size
Composite Score      = fee_rate × 0.7 + coin_age_priority × 0.001 × 0.3
Recommended Fee      = transaction size × sat_per_byte (select 1/5/20/50 by urgency)
```
