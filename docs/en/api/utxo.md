# UTXO API

The UTXO module manages all unspent transaction outputs and is the core of Bitcoin's accounting model.

## Data Structures

### `UTXOSet`

```rust
pub struct UTXOSet {
    // key: txid (transaction ID)
    // value: Vec<(vout_index, TxOutput)>
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
}
```

**Internal structure**:
```
HashMap {
    "tx1": [(0, Output{value: 100, addr: "alice"}),
            (1, Output{value: 50, addr: "bob"})],
    "tx2": [(0, Output{value: 200, addr: "charlie"})],
}
```

---

## Core Concepts

### UTXO Model vs. Account Model

#### Account Model (Ethereum)
```
Account balances:
  Alice: 100 BTC
  Bob: 50 BTC

Transfer: Alice → Bob (30 BTC)
  Alice: 70 BTC  (-30)
  Bob: 80 BTC    (+30)
```

#### UTXO Model (Bitcoin)
```
UTXO set:
  tx1:0 → 100 BTC (Alice)
  tx1:1 → 50 BTC (Bob)

Transfer: Alice → Bob (30 BTC)
  Spend: tx1:0 (100 BTC)
  Create:
    tx2:0 → 30 BTC (Bob)
    tx2:1 → 70 BTC (Alice, change)

New UTXO set:
  tx1:1 → 50 BTC (Bob)
  tx2:0 → 30 BTC (Bob)
  tx2:1 → 70 BTC (Alice)
```

### Advantages of the UTXO Model

1. **Better privacy**
   - Can use a new address each time
   - Harder to trace fund flows

2. **Parallel processing**
   - Different UTXOs can be validated concurrently
   - No account locking issues

3. **Simplified validation**
   - Only need to check UTXO existence
   - No need for account history

---

## Methods

### Initialization

#### `new`

```rust
pub fn new() -> Self
```

Creates an empty UTXO set.

**Example**:
```rust
use bitcoin_simulation::utxo::UTXOSet;

let mut utxo_set = UTXOSet::new();
```

---

### UTXO Management

#### `add_transaction`

```rust
pub fn add_transaction(&mut self, tx: &Transaction)
```

Adds all outputs of a transaction to the UTXO set.

**Process**:
1. Iterate over all outputs of the transaction
2. Mark each output as unspent
3. Add to the UTXO set

**Parameters**:
- `tx` - The transaction to add

**Note**: Only adds outputs, does not process inputs

**Example**:
```rust
let tx = Transaction::new(...);
utxo_set.add_transaction(&tx);

// Now all of tx's outputs can be spent
```

---

#### `remove_utxo`

```rust
pub fn remove_utxo(&mut self, txid: &str, vout: usize)
```

Removes a spent UTXO.

**Double-spend protection**:
- A UTXO can only be spent once
- Removed from the set immediately after spending
- A second reference will fail

**Parameters**:
- `txid` - Transaction ID
- `vout` - Output index

**Example**:
```rust
// Spend a UTXO
utxo_set.remove_utxo("tx1", 0);

// Attempt to spend again (fails)
// UTXO does not exist
```

---

#### `process_transaction`

```rust
pub fn process_transaction(&mut self, tx: &Transaction) -> bool
```

Fully processes a transaction (removes inputs, adds outputs).

**ACID properties**:

**Atomicity**:
```rust
// Either fully succeeds or fully fails
if !tx.verify() {
    return false;  // No modifications made
}
// All processed
```

**Consistency**:
```rust
// Before and after processing: input_sum = output_sum + fee
assert_eq!(input_sum, output_sum + fee);
```

**Steps**:
1. Validate the transaction
2. Remove UTXOs referenced by inputs
3. Add newly created outputs

**Parameters**:
- `tx` - The transaction to process

**Return value**:
- `true` - Processing successful
- `false` - Transaction is invalid

**Example**:
```rust
let tx = blockchain.create_transaction(&alice, bob.address, 1000, 10)?;

if utxo_set.process_transaction(&tx) {
    println!("✓ UTXO updated successfully");
} else {
    println!("✗ Invalid transaction");
}
```

---

### Query Operations

#### `find_utxos`

```rust
pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)>
```

Finds all UTXOs for an address.

**Return format**: `Vec<(txid, vout, value)>`

**Parameters**:
- `address` - The address to query

**Return value**: List of UTXOs

**Example**:
```rust
let utxos = utxo_set.find_utxos(&alice.address);

println!("Alice's UTXOs:");
for (txid, vout, value) in utxos {
    println!("  {}:{} → {} sat", &txid[..8], vout, value);
}

// Output:
// Alice's UTXOs:
//   tx1:0 → 5000 sat
//   tx2:1 → 3000 sat
//   tx5:0 → 2000 sat
```

---

#### `find_spendable_outputs`

```rust
pub fn find_spendable_outputs(
    &self,
    address: &str,
    amount: u64
) -> Option<(u64, Vec<(String, usize)>)>
```

Finds a combination of UTXOs usable for payment.

**UTXO selection strategies**:

1. **Greedy algorithm** (current implementation):
   ```rust
   accumulated = 0
   for utxo in utxos:
       accumulated += utxo.value
       if accumulated >= amount:
           return utxos
   ```

2. **Optimal match** (possible optimization):
   - Select the combination closest in total to the target
   - Reduces change, saving on fees

3. **Smallest UTXO first**:
   - Prefer small UTXOs
   - Avoids UTXO fragmentation

**Parameters**:
- `address` - Sender address
- `amount` - Required amount (including fee)

**Return value**:
- `Some((accumulated, utxo_list))` - Sufficient UTXOs found
  - `accumulated`: Accumulated amount
  - `utxo_list`: Selected UTXO list
- `None` - Insufficient balance

**Example**:
```rust
// Need 1000 sat (including fee)
let result = utxo_set.find_spendable_outputs(&alice.address, 1000);

match result {
    Some((accumulated, utxos)) => {
        println!("✓ Found sufficient UTXOs");
        println!("  Accumulated amount: {} sat", accumulated);
        println!("  UTXOs used: {}", utxos.len());
        println!("  Change: {} sat", accumulated - 1000);
    }
    None => {
        println!("✗ Insufficient balance");
    }
}
```

---

#### `get_balance`

```rust
pub fn get_balance(&self, address: &str) -> u64
```

Calculates the total balance for an address.

**Computation method**:
```rust
balance = sum(all_utxos.value)
```

**Parameters**:
- `address` - The address to query

**Return value**: Balance (satoshi)

**Example**:
```rust
let balance = utxo_set.get_balance(&alice.address);
println!("Balance: {} satoshi", balance);
println!("Balance: {:.8} BTC", balance as f64 / 100_000_000.0);

// Batch query
let addresses = vec![alice.address, bob.address, charlie.address];
for addr in addresses {
    let bal = utxo_set.get_balance(&addr);
    println!("{}: {} sat", &addr[..10], bal);
}
```

---

## Use Cases

### Case 1: Selecting UTXOs When Creating a Transaction

```rust
fn create_payment(
    utxo_set: &UTXOSet,
    from: &Wallet,
    to: &str,
    amount: u64,
    fee: u64
) -> Result<Transaction, String> {
    let total_needed = amount + fee;

    // 1. Find available UTXOs
    let result = utxo_set.find_spendable_outputs(&from.address, total_needed);

    let (accumulated, utxo_refs) = result.ok_or("Insufficient balance")?;

    // 2. Build inputs
    let mut inputs = Vec::new();
    for (txid, vout) in utxo_refs {
        let signature = from.sign(&format!("{}{}", txid, vout));
        inputs.push(TxInput::new(txid, vout, signature, from.public_key.clone()));
    }

    // 3. Build outputs
    let mut outputs = vec![
        TxOutput::new(amount, to.to_string()),  // To recipient
    ];

    // 4. Change
    if accumulated > total_needed {
        outputs.push(TxOutput::new(
            accumulated - total_needed,
            from.address.clone()
        ));
    }

    // 5. Create transaction
    Ok(Transaction::new(inputs, outputs, current_timestamp(), fee))
}
```

### Case 2: Detailed Balance Query

```rust
fn balance_breakdown(utxo_set: &UTXOSet, address: &str) {
    let utxos = utxo_set.find_utxos(address);
    let total = utxo_set.get_balance(address);

    println!("=== Balance Details ===");
    println!("Address: {}", &address[..20]);
    println!("Total balance: {} sat ({:.8} BTC)", total, total as f64 / 1e8);
    println!("UTXO count: {}", utxos.len());
    println!("\nUTXO list:");

    for (i, (txid, vout, value)) in utxos.iter().enumerate() {
        println!("  #{}: {}:{} → {} sat",
            i + 1, &txid[..8], vout, value);
    }

    // Statistics
    if !utxos.is_empty() {
        let avg = total / utxos.len() as u64;
        let max = utxos.iter().map(|(_, _, v)| v).max().unwrap();
        let min = utxos.iter().map(|(_, _, v)| v).min().unwrap();

        println!("\nStatistics:");
        println!("  Average: {} sat", avg);
        println!("  Maximum: {} sat", max);
        println!("  Minimum: {} sat", min);
    }
}
```

### Case 3: UTXO Consolidation

```rust
fn consolidate_utxos(
    blockchain: &mut Blockchain,
    wallet: &Wallet
) -> Result<(), String> {
    let utxos = blockchain.utxo_set.find_utxos(&wallet.address);

    // If too many UTXOs (>50), consolidate into 1
    if utxos.len() > 50 {
        println!("Starting UTXO consolidation...");
        println!("  Current UTXO count: {}", utxos.len());

        // Create a self-to-self transaction consolidating all UTXOs
        let total = blockchain.get_balance(&wallet.address);
        let fee = 100;  // Fixed fee

        let tx = blockchain.create_transaction(
            wallet,
            wallet.address.clone(),
            total - fee,
            fee,
        )?;

        blockchain.add_transaction(tx)?;
        blockchain.mine_pending_transactions(wallet.address.clone())?;

        let new_utxos = blockchain.utxo_set.find_utxos(&wallet.address);
        println!("✓ Consolidation complete");
        println!("  New UTXO count: {}", new_utxos.len());
    }

    Ok(())
}
```

### Case 4: UTXO Audit

```rust
fn audit_utxo_set(utxo_set: &UTXOSet, blockchain: &Blockchain) -> bool {
    println!("=== UTXO Audit ===");

    // 1. Count total UTXOs
    let total_utxos: usize = utxo_set.utxos.values()
        .map(|v| v.len())
        .sum();
    println!("Total UTXO count: {}", total_utxos);

    // 2. Sum total value
    let mut total_value = 0u64;
    for outputs in utxo_set.utxos.values() {
        for (_, output) in outputs {
            total_value += output.value;
        }
    }
    println!("Total value: {} sat", total_value);

    // 3. Validate each UTXO
    let mut valid = true;
    for (txid, outputs) in &utxo_set.utxos {
        // Verify the transaction exists on the blockchain
        let tx_exists = blockchain.chain.iter()
            .any(|block| block.transactions.iter()
                .any(|tx| &tx.id == txid));

        if !tx_exists {
            println!("✗ Warning: UTXO references non-existent transaction {}", txid);
            valid = false;
        }
    }

    if valid {
        println!("✓ UTXO set is complete");
    }

    valid
}
```

---

## Performance Optimization

### 1. Index Optimization

```rust
// Create an index for addresses
pub struct IndexedUTXOSet {
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
    // New: address index
    address_index: HashMap<String, Vec<(String, usize)>>,
}

impl IndexedUTXOSet {
    pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)> {
        // O(1) lookup instead of O(n)
        if let Some(refs) = self.address_index.get(address) {
            refs.iter()
                .filter_map(|(txid, vout)| {
                    self.utxos.get(txid)
                        .and_then(|outputs| outputs.iter()
                            .find(|(idx, _)| idx == vout)
                            .map(|(_, output)| (txid.clone(), *vout, output.value))
                        )
                })
                .collect()
        } else {
            vec![]
        }
    }
}
```

### 2. Batch Operations

```rust
// Process transactions in batch
pub fn process_transactions(&mut self, txs: &[Transaction]) -> bool {
    // 1. Validate all transactions
    for tx in txs {
        if !tx.verify() {
            return false;
        }
    }

    // 2. Batch update UTXOs
    for tx in txs {
        // Remove inputs
        if !tx.is_coinbase() {
            for input in &tx.inputs {
                self.remove_utxo(&input.txid, input.vout);
            }
        }

        // Add outputs
        self.add_transaction(tx);
    }

    true
}
```

### 3. Balance Caching

```rust
pub struct CachedUTXOSet {
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
    balance_cache: HashMap<String, u64>,  // Balance cache
}

impl CachedUTXOSet {
    pub fn get_balance(&mut self, address: &str) -> u64 {
        // Check cache
        if let Some(balance) = self.balance_cache.get(address) {
            return *balance;
        }

        // Compute and cache
        let balance = self.calculate_balance(address);
        self.balance_cache.insert(address.to_string(), balance);
        balance
    }

    fn invalidate_cache(&mut self, address: &str) {
        self.balance_cache.remove(address);
    }
}
```

---

## Comparison with Ethereum Account Model

| Feature | UTXO Model (Bitcoin) | Account Model (Ethereum) |
|---------|---------------------|--------------------------|
| State | Stateless (only UTXO set) | Stateful (account balances, nonce) |
| Balance | Computed value (UTXO sum) | Stored value (directly stored) |
| Transfer | Spend UTXOs, create new UTXOs | Account balance increases/decreases |
| Privacy | Better (can use new address) | Worse (reuse of addresses) |
| Parallelism | Easy to validate in parallel | Requires sequential processing (nonce) |
| Complexity | Complex transaction construction | Simple transactions |
| Smart contracts | Limited (Script) | Flexible (EVM) |

---

## Frequently Asked Questions

### Q: Why use the UTXO model?

**A:**
- ✅ Better privacy (new address each time)
- ✅ Parallel validation (different UTXOs are independent)
- ✅ Simplified validation logic
- ✅ Natural double-spend prevention

### Q: Will UTXOs keep accumulating?

**A:** Yes. Solutions:
- UTXO consolidation (merge multiple small UTXOs)
- Higher transaction fees (limit junk UTXOs)
- UTXO commitments (reduce storage)

### Q: How do I prevent UTXO fragmentation?

**A:**
```rust
// Consolidate periodically
if utxos.len() > threshold {
    consolidate_utxos();
}

// Prioritize small UTXOs
utxos.sort_by_key(|u| u.value);  // Smallest first
```

### Q: What if UTXOs are lost?

**A:** As long as you have the private key, you can rebuild the UTXO set from the blockchain:
```rust
fn rebuild_utxo_set(blockchain: &Blockchain, address: &str) -> UTXOSet {
    let mut utxo_set = UTXOSet::new();

    for block in &blockchain.chain {
        for tx in &block.transactions {
            utxo_set.process_transaction(tx);
        }
    }

    utxo_set
}
```

---

## References

- [Transaction API](./transaction.md) - Creating and spending UTXOs
- [Blockchain API](./blockchain.md) - UTXO set management
- [Basic Concepts - UTXO Model](../guide/concepts.md#utxo-model)
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)

---

[Back to API Index](./core.md)
