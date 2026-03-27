# Advanced TX API

The advanced transaction module is implemented in `src/advanced_tx.rs` and provides three key mechanisms addressing real engineering problems in the Bitcoin network: **RBF (Replace-By-Fee)** (allowing acceleration or cancellation of unconfirmed transactions), **TimeLock** (restricting transactions from being confirmed before a specified time or block), and **TxPriorityCalculator** (recommending reasonable fees and calculating transaction priority).

---

## RBFManager — Replace-By-Fee Manager

RBF (Replace-By-Fee), defined in BIP125, allows users to replace an unconfirmed transaction in the mempool with a new one bearing a higher fee, thereby accelerating confirmation or canceling an erroneous transaction.

### Struct Definition

```rust
pub struct RBFManager {
    // replaceable_txs: Vec<String>  // Private field, stores list of replaceable transaction IDs
}
```

### Methods

#### `RBFManager::new`

Creates a new RBF manager instance (replaceable transaction list is empty).

```rust
pub fn new() -> Self
```

#### `RBFManager::mark_replaceable`

Marks the specified transaction as supporting RBF replacement. Idempotent operation; marking the same transaction multiple times has no side effects.

```rust
pub fn mark_replaceable(&mut self, tx_id: &str)
```

**Parameters:**
- `tx_id` — Transaction ID to mark as replaceable.

In the Bitcoin protocol, a transaction indicates RBF support by setting its `nSequence` field to a value less than `0xFFFFFFFE`. `AdvancedTxBuilder::with_rbf()` automatically sets `sequence` to `0xFFFFFFFD`.

#### `RBFManager::is_replaceable`

Checks whether a transaction has been marked as replaceable.

```rust
pub fn is_replaceable(&self, tx_id: &str) -> bool
```

#### `RBFManager::can_replace`

Validates whether a new transaction can legitimately replace the old one. Performs full RBF rule checks:

```rust
pub fn can_replace(
    &self,
    old_tx: &Transaction,
    new_tx: &Transaction,
) -> Result<(), String>
```

**Validation rules (in order):**

1. **Replaceability check:** `old_tx.id` must be in the replaceable list; otherwise returns `"Original transaction does not support RBF"`.
2. **Same inputs:** Both transactions' input lists have the same length, and corresponding inputs have identical `txid` and `vout` (must spend the same UTXOs); otherwise returns `"Must spend the same UTXOs"`.
3. **Higher fee:** `new_tx.fee > old_tx.fee`; otherwise returns `"New transaction fee({}) must be higher than old transaction({})"`.
4. **Sufficient increment:** `fee_increase >= old_tx.size()` (simplified rule: fee increment must be at least as many satoshi as the old transaction's byte count), preventing low-cost spam replacement attacks.

**Return value:**
- `Ok(())` — Replacement is legitimate; the new transaction can be broadcast.
- `Err(String)` — Specific validation failure reason.

#### `RBFManager::remove_confirmed`

Removes a confirmed transaction from the replaceable list.

```rust
pub fn remove_confirmed(&mut self, tx_id: &str)
```

### RBFManager Usage Example

```rust
use simplebtc::advanced_tx::RBFManager;

let mut rbf = RBFManager::new();

// Mark original transaction as supporting RBF
rbf.mark_replaceable("original_tx_001");
assert!(rbf.is_replaceable("original_tx_001"));
assert!(!rbf.is_replaceable("other_tx_002"));

// Validate whether replacement is legitimate
match rbf.can_replace(&old_tx, &new_tx) {
    Ok(()) => println!("Replacement legitimate, broadcast new transaction"),
    Err(e) => println!("Replacement rejected: {}", e),
}

// Remove record after transaction is confirmed
rbf.remove_confirmed("original_tx_001");
assert!(!rbf.is_replaceable("original_tx_001"));
```

---

## TimeLock — Timelock

Timelocks restrict a transaction from being mined by miners before a specific time or block height. They are a foundational primitive for implementing advanced scenarios such as time deposits, inheritance, smart contracts, and more.

### Struct Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLock {
    pub locktime: u64,         // Lock value: Unix timestamp (seconds) or block height
    pub is_block_height: bool, // true: block-height-based; false: timestamp-based
}
```

| Field | Type | Description |
|-------|------|-------------|
| `locktime` | `u64` | Lock value. When `is_block_height = true`, this is the block height; otherwise it is a Unix timestamp (seconds). |
| `is_block_height` | `bool` | Lock type flag. Corresponds to Bitcoin protocol: `locktime < 500_000_000` = block height; `>= 500_000_000` = timestamp. |

### Methods

#### `TimeLock::new_time_based`

Creates a Unix timestamp-based timelock. The transaction cannot be confirmed before `timestamp` seconds.

```rust
pub fn new_time_based(timestamp: u64) -> Self
```

**Parameters:**
- `timestamp` — Unix timestamp (seconds) for unlock time. For example, `1767225600` represents 2026-01-01 00:00:00 UTC.

#### `TimeLock::new_height_based`

Creates a block-height-based timelock. The transaction cannot be confirmed until the blockchain reaches the specified height.

```rust
pub fn new_height_based(height: u64) -> Self
```

**Parameters:**
- `height` — Block height required for unlock. For example, `900_000` represents approximately mid-2027 (based on approximately 10 minutes per block).

#### `TimeLock::is_mature`

Checks whether the timelock has expired (is ready to be spent).

```rust
pub fn is_mature(&self, current_time: u64, current_height: u32) -> bool
```

**Parameters:**
- `current_time` — Current Unix timestamp (seconds).
- `current_height` — Current blockchain height.

**Return value:**
- Time-based: `current_time >= self.locktime`
- Block-height-based: `current_height as u64 >= self.locktime`

#### `TimeLock::remaining`

Gets how much time (seconds) or how many blocks remain until unlock.

```rust
pub fn remaining(&self, current_time: u64, current_height: u32) -> i64
```

**Return value:** Remaining seconds or block count. A negative value indicates the lock time has already passed (expired).

### TimeLock Usage Example

```rust
use simplebtc::advanced_tx::TimeLock;

// Timestamp-based: lock until 2026-01-01 00:00:00 UTC
let time_lock = TimeLock::new_time_based(1_767_225_600);
let now = 1_740_000_000u64; // Current time (2025)
println!("Timelock expired: {}", time_lock.is_mature(now, 0)); // false
println!("Remaining until unlock: {} seconds", time_lock.remaining(now, 0));

// Block-height-based: lock until block 900,000
let block_lock = TimeLock::new_height_based(900_000);
let current_height = 850_000u32; // Current block height
println!("Block lock expired: {}", block_lock.is_mature(0, current_height)); // false
println!("Remaining until unlock: {} blocks", block_lock.remaining(0, current_height)); // 50000

// An already-expired timelock
let expired_lock = TimeLock::new_height_based(800_000);
println!("Expired: {}", expired_lock.is_mature(0, 850_000)); // true
println!("Remaining (negative = already past): {}", expired_lock.remaining(0, 850_000)); // -50000
```

---

## AdvancedTxBuilder — Advanced Transaction Builder

`AdvancedTxBuilder` is a builder (Builder Pattern) for configuring advanced transaction options (RBF support and timelocks) and generating the corresponding `sequence` field value.

### Struct Definition

```rust
pub struct AdvancedTxBuilder {
    pub enable_rbf: bool,
    pub timelock: Option<TimeLock>,
    pub sequence: u32,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `enable_rbf` | `bool` | Whether RBF support is enabled. `true` after `with_rbf()`. |
| `timelock` | `Option<TimeLock>` | Associated timelock configuration. `Some(TimeLock)` after `with_timelock()`. |
| `sequence` | `u32` | Transaction input sequence number, encoding RBF and timelock state: `0xFFFFFFFF` (default/no feature), `0xFFFFFFFD` (RBF), `0x00000000` (timelock). |

### Methods

#### `AdvancedTxBuilder::new`

Creates a default builder. RBF and timelock are disabled by default; `sequence = 0xFFFFFFFF`.

```rust
pub fn new() -> Self
```

#### `AdvancedTxBuilder::with_rbf`

Enables RBF support. Sets `enable_rbf` to `true` and `sequence` to `0xFFFFFFFD` (less than `0xFFFFFFFE`, compliant with BIP125).

```rust
pub fn with_rbf(mut self) -> Self
```

**Return value:** `Self` (supports method chaining).

#### `AdvancedTxBuilder::with_timelock`

Sets a timelock. Sets `timelock` to `Some(timelock)` and `sequence` to `0` (enables `nLockTime` mechanism).

```rust
pub fn with_timelock(mut self, timelock: TimeLock) -> Self
```

**Parameters:**
- `timelock` — The `TimeLock` instance to associate.

**Return value:** `Self` (supports method chaining).

#### `AdvancedTxBuilder::get_sequence`

Gets the final `sequence` field value, which should be written to the transaction input's `nSequence` field.

```rust
pub fn get_sequence(&self) -> u32
```

#### `AdvancedTxBuilder::supports_rbf`

Checks whether the current configuration supports RBF (`sequence < 0xFFFFFFFE`).

```rust
pub fn supports_rbf(&self) -> bool
```

### AdvancedTxBuilder Usage Example

```rust
use simplebtc::advanced_tx::{AdvancedTxBuilder, TimeLock};

// Enable RBF only
let rbf_builder = AdvancedTxBuilder::new()
    .with_rbf();
println!("sequence: 0x{:08X}", rbf_builder.get_sequence()); // 0xFFFFFFFD
println!("Supports RBF: {}", rbf_builder.supports_rbf()); // true

// Enable timelock only (lock until block 900,000)
let timelock = TimeLock::new_height_based(900_000);
let timelock_builder = AdvancedTxBuilder::new()
    .with_timelock(timelock);
println!("sequence: 0x{:08X}", timelock_builder.get_sequence()); // 0x00000000

// RBF + timelock combined (with_timelock overrides sequence to 0)
let combined = AdvancedTxBuilder::new()
    .with_rbf()
    .with_timelock(TimeLock::new_time_based(1_800_000_000));
println!("Timelock: {:?}", combined.timelock);
println!("sequence: 0x{:08X}", combined.get_sequence()); // 0x00000000

// Default builder (no advanced features)
let default_builder = AdvancedTxBuilder::new();
println!("sequence: 0x{:08X}", default_builder.get_sequence()); // 0xFFFFFFFF
println!("Supports RBF: {}", default_builder.supports_rbf()); // false
```

---

## TxPriorityCalculator — Transaction Priority Calculator

`TxPriorityCalculator` is a stateless utility class (all methods are associated functions) for calculating transaction priority scores and recommending reasonable fees. Miners use priority scores to decide which mempool transactions to pack first.

### Struct Definition

```rust
pub struct TxPriorityCalculator;
```

### FeeUrgency — Fee Urgency Level

```rust
#[derive(Debug, Clone, Copy)]
pub enum FeeUrgency {
    Low,    // Low priority: 1 sat/byte, confirmation within hours
    Medium, // Medium priority: 5 sat/byte, 30–60 minute confirmation
    High,   // High priority: 20 sat/byte, 10–20 minutes (approximately 1–2 blocks)
    Urgent, // Urgent: 50 sat/byte, next block (approximately 10 minutes)
}
```

| Enum Value | Fee Rate | Expected Confirmation Time | Typical Use Case |
|-----------|----------|--------------------------|-----------------|
| `Low` | 1 sat/byte | Hours to days | Non-urgent transfers, low network fee periods |
| `Medium` | 5 sat/byte | 30–60 minutes | Everyday transactions, normal confirmation speed |
| `High` | 20 sat/byte | 10–20 minutes | Time-sensitive transactions (Lightning Network channel opening) |
| `Urgent` | 50 sat/byte | ~10 minutes (next block) | Urgent payments, exchange withdrawals |

### Methods

#### `TxPriorityCalculator::calculate_priority`

Calculates the traditional priority score based on UTXO value and age.

**Formula:** `priority = (input_value × input_age) / tx_size`

```rust
pub fn calculate_priority(
    input_value: u64, // Total value of input UTXOs (satoshi)
    input_age: u32,   // Age of input UTXOs (number of confirmations)
    tx_size: usize,   // Transaction size (bytes)
) -> f64
```

Older (larger `age`) and higher-value UTXOs have higher priority. Returns `0.0` when `tx_size = 0` (prevents division by zero).

#### `TxPriorityCalculator::calculate_fee_rate`

Calculates the transaction fee rate (sat/byte).

**Formula:** `fee_rate = fee / size`

```rust
pub fn calculate_fee_rate(
    fee: u64,    // Fee (satoshi)
    size: usize, // Transaction size (bytes)
) -> f64
```

Returns `0.0` when `size = 0`.

#### `TxPriorityCalculator::calculate_score`

Calculates a composite score (miner sorting basis).

**Formula:** `score = fee_rate × 0.7 + priority × 0.001 × 0.3`

```rust
pub fn calculate_score(fee_rate: f64, priority: f64) -> f64
```

Weight distribution: 70% fee-rate-based, 30% UTXO-priority-based. Transactions with higher fee rates get higher composite scores and are more likely to be selected by miners.

#### `TxPriorityCalculator::recommend_fee`

Recommends a fee (satoshi) based on transaction size and urgency.

```rust
pub fn recommend_fee(
    tx_size: usize,    // Transaction size (bytes)
    urgency: FeeUrgency, // Fee urgency level
) -> u64
```

**Return value:** `(tx_size × sat_per_byte) as u64` truncated.

### TxPriorityCalculator Usage Example

```rust
use simplebtc::advanced_tx::{TxPriorityCalculator, FeeUrgency};

// A standard Bitcoin transaction is approximately 250 bytes (1 input + 2 outputs)
let tx_size = 250usize;

// Recommended fee for each urgency level
println!("Low priority:  {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Low));
// 250 sat
println!("Medium priority:  {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Medium));
// 1250 sat
println!("High priority:  {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::High));
// 5000 sat
println!("Urgent:      {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Urgent));
// 12500 sat

// Calculate fee rate for an existing transaction
let actual_fee = 2000u64; // Actual fee
let fee_rate = TxPriorityCalculator::calculate_fee_rate(actual_fee, tx_size);
println!("Actual fee rate: {:.1} sat/byte", fee_rate); // 8.0 sat/byte

// Calculate UTXO priority (hold 1 BTC, confirmed for 100 blocks, 250-byte transaction)
let priority = TxPriorityCalculator::calculate_priority(
    100_000_000, // 1 BTC = 100,000,000 satoshi
    100,         // 100 blocks of age
    tx_size,
);
println!("Priority score: {:.0}", priority); // 40,000,000

// Composite score (miner sorting basis)
let score = TxPriorityCalculator::calculate_score(fee_rate, priority);
println!("Composite score: {:.2}", score);
```

---

## Complete Usage Examples

### Scenario 1: Accelerating an Unconfirmed Transaction with RBF

```rust
use simplebtc::advanced_tx::{RBFManager, AdvancedTxBuilder, TxPriorityCalculator, FeeUrgency};

fn accelerate_tx_example() {
    let mut rbf = RBFManager::new();

    // 1. Send the original transaction (low fee, supports RBF)
    let builder = AdvancedTxBuilder::new().with_rbf();
    println!("RBF sequence: 0x{:08X}", builder.get_sequence()); // 0xFFFFFFFD

    // Simulate transaction sent but still unconfirmed after 30 minutes
    // ... create and broadcast original transaction original_tx ...
    rbf.mark_replaceable("original_tx_id_001");

    // 2. Network congested, need to increase fee
    let tx_size = 250usize;
    let old_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Low);
    let new_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::High);
    println!("Original fee: {} sat -> New fee: {} sat", old_fee, new_fee);

    // 3. Validate replacement rules
    // can_replace checks: same inputs, higher fee, sufficient increment
    // match rbf.can_replace(&old_tx, &new_tx) {
    //     Ok(()) => { /* broadcast new transaction */ }
    //     Err(e) => println!("Replacement rejected: {}", e),
    // }

    // 4. Clean up after old transaction is confirmed (or replaced)
    rbf.remove_confirmed("original_tx_id_001");
}
```

### Scenario 2: Time Deposit with Timelock

```rust
use simplebtc::advanced_tx::{AdvancedTxBuilder, TimeLock};

fn savings_timelock() {
    // Lock until block height 950,000 (approximately 2028)
    let unlock_height = 950_000u64;
    let timelock = TimeLock::new_height_based(unlock_height);

    let builder = AdvancedTxBuilder::new()
        .with_timelock(timelock.clone());

    println!("Transaction sequence: 0x{:08X}", builder.get_sequence()); // 0x00000000
    println!("Timelock enabled: {}", builder.timelock.is_some());

    // Check if funds can currently be used
    let current_height = 870_000u32;
    if timelock.is_mature(0, current_height) {
        println!("Funds unlocked, can be used");
    } else {
        let remaining = timelock.remaining(0, current_height);
        println!("Need to wait {} more blocks (approximately {} days)",
            remaining,
            remaining * 10 / 60 / 24); // Approximate days
    }
}
```

### Scenario 3: Fee Strategy Analysis

```rust
use simplebtc::advanced_tx::{TxPriorityCalculator, FeeUrgency};

fn fee_strategy_analysis() {
    let tx_sizes = vec![
        (125,  "Simple payment (1 input, 1 output)"),
        (250,  "Standard transaction (1 input, 2 outputs)"),
        (500,  "Batch payment (multiple inputs and outputs)"),
        (1000, "Large transaction (common before SegWit)"),
    ];

    println!("{:<40} {:>10} {:>10} {:>10} {:>10}",
        "Transaction Type", "Low", "Medium", "High", "Urgent");
    println!("{}", "-".repeat(80));

    for (size, desc) in &tx_sizes {
        println!("{:<40} {:>10} {:>10} {:>10} {:>10}",
            desc,
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::Low),
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::Medium),
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::High),
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::Urgent),
        );
    }

    // Composite score comparison
    let fee_rate_a = TxPriorityCalculator::calculate_fee_rate(500, 250); // 2 sat/byte
    let fee_rate_b = TxPriorityCalculator::calculate_fee_rate(5000, 250); // 20 sat/byte
    let priority_a = TxPriorityCalculator::calculate_priority(10_000_000, 50, 250);
    let priority_b = TxPriorityCalculator::calculate_priority(100_000, 1, 250);

    println!("\nTransaction A (old UTXO, low fee rate) composite score: {:.2}", TxPriorityCalculator::calculate_score(fee_rate_a, priority_a));
    println!("Transaction B (new UTXO, high fee rate) composite score: {:.2}", TxPriorityCalculator::calculate_score(fee_rate_b, priority_b));
}
```

---

## sequence Field Value Reference

| sequence Value | Meaning |
|---------------|---------|
| `0xFFFFFFFF` | Default value; RBF and timelock not enabled |
| `0xFFFFFFFE` | Does not support RBF, but allows nLockTime |
| `0xFFFFFFFD` | Supports RBF (BIP125 standard value) |
| `0x00000000` | Enables timelock (nLockTime takes effect) |

---

## Related Modules

- [`Mempool`](advanced.md#mempoolmempool) — The mempool uses `RBFManager` for transaction replacement and `TxPriorityCalculator` to sort pending transactions.
- [`MultiSig`](multisig.md) — Multisig and timelocks can be combined to implement scenarios such as "M-of-N required before expiry, reduced to 1-of-N after."
- [Advanced Modules Overview](advanced.md) — View the full advanced module dependency graph.
