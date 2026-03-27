# UTXO Management

UTXO (Unspent Transaction Output) is the core of Bitcoin's ledger model. Understanding UTXOs is a key step toward mastering how Bitcoin works. This chapter introduces the design and usage of `UTXOSet` in SimpleBTC.

---

## What is a UTXO?

In the traditional banking system (and in Ethereum's account model), the system directly records the balance of each account. Bitcoin chose a fundamentally different approach: **instead of storing balances, it only stores "unspent outputs"**.

Every Bitcoin transaction:
1. **Consumes** some previously existing UTXOs (as inputs)
2. **Creates** some new UTXOs (as outputs)

Think of Bitcoin like physical cash: you have a 100-unit bill and a 50-unit bill. You want to buy something worth 80 units, so you hand over the entire 100-unit bill and receive 20 units in change. You "spent" the 100-unit UTXO and "created" two new UTXOs — one worth 80 units for the merchant and one worth 20 units as change for yourself.

### UTXO Lifecycle

```
Creation              Existence                  Spending              Destruction
 |                     |                          |                     |
 v                     v                          v                     v
Tx output ──────► UTXO set ──────────────► Referenced by new tx ──────► Removed from set
(packed into block)  (queryable and spendable)   (as input)           (cannot be reused)
```

---

## UTXO Model vs. Account Model

| Feature | UTXO Model (Bitcoin) | Account Model (Ethereum) |
|---------|---------------------|--------------------------|
| State storage | Records all unspent outputs | Records the balance of each account |
| Balance calculation | Sum all UTXOs belonging to the address | Read the account field directly |
| Privacy | Better (each transaction can use a new address) | Weaker (fixed address) |
| Parallel processing | Naturally supported (different UTXOs are independent) | Requires extra concurrency control |
| Double-spend prevention | A UTXO can only be spent once | Controlled via nonce sequence numbers |
| Complex contracts | Harder to implement | Natively supported |
| Intuitiveness | Requires understanding the UTXO concept | Similar to a bank account; intuitive |

The source code comments (`src/utxo.rs`, lines 8–19) provide a concise summary:

```rust
// Account model (Ethereum, etc.):
// - Records the balance of each account
// - Transfer: account A -100, account B +100
// - Simple and intuitive, but hard to process in parallel
//
// UTXO model (Bitcoin):
// - No concept of account balance
// - Only records unspent transaction outputs
// - Transfer: spend A's UTXO, create a new UTXO for B
// - Better privacy and parallelism
```

---

## UTXOSet Data Structure

SimpleBTC uses `UTXOSet` to manage the set of all unspent outputs across the entire blockchain.

```rust
#[derive(Debug, Clone)]
pub struct UTXOSet {
    // key: txid (transaction ID)
    // value: list of all unspent outputs for that transaction [(output index, output details)]
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
}
```

- **Key**: Transaction ID (txid), a hexadecimal hash string
- **Value**: The list of outputs under that transaction that have not yet been spent; each item contains the output's index within the transaction (`vout`) and the output's detailed information (`TxOutput`)

This data structure makes it very efficient (O(1) hash lookup) to find all available outputs for a given txid, and also makes it easy to remove a specific individual output.

---

## Core API Details

### Creating a UTXO Set

```rust
let mut utxo_set = UTXOSet::new();
```

Initializes an empty UTXO set. When the blockchain starts, it processes all transactions block by block from the genesis block to populate it.

---

### Adding Transaction Outputs: `add_transaction`

```rust
pub fn add_transaction(&mut self, tx: &Transaction)
```

When a new transaction is packed into a block and confirmed, this method is called to add **all outputs** of that transaction to the UTXO set.

```rust
// Example: a genesis block is mined, and the coinbase reward enters the UTXO set
let coinbase_tx = Transaction::new_coinbase("miner_address".to_string(), 50, 0, 0);
utxo_set.add_transaction(&coinbase_tx);
// Now coinbase_tx.id -> [(0, TxOutput { value: 50, ... })] is in the set
```

> **Note**: `add_transaction` only adds outputs; it does not process inputs (it does not remove spent UTXOs). For complete transaction processing, use `process_transaction`.

---

### Removing Spent Outputs: `remove_utxo`

```rust
pub fn remove_utxo(&mut self, txid: &str, vout: usize)
```

When a UTXO is referenced by an input of a transaction (i.e., it is spent), it must be removed from the set. This is the core mechanism for preventing double spending.

```rust
// The user spent output #0 of txid="abc123"
utxo_set.remove_utxo("abc123", 0);
// Attempting to spend the same UTXO again will fail validation because it is no longer in the set
```

In the implementation, `remove_utxo` uses `retain` to keep other unaffected outputs; if all outputs of a transaction have been spent, the entire txid entry is also deleted:

```rust
pub fn remove_utxo(&mut self, txid: &str, vout: usize) {
    if let Some(outputs) = self.utxos.get_mut(txid) {
        outputs.retain(|(index, _)| *index != vout);
        if outputs.is_empty() {
            self.utxos.remove(txid);
        }
    }
}
```

---

### Querying All UTXOs for an Address: `find_utxos`

```rust
pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)>
```

Iterates over the entire UTXO set and returns all unspent outputs belonging to the specified address. The result format is `(txid, vout, value)`.

```rust
let utxos = utxo_set.find_utxos("alice_address");
for (txid, vout, value) in &utxos {
    println!("UTXO: {}:{} = {} satoshis", txid, vout, value);
}
```

---

### Finding Spendable Outputs: `find_spendable_outputs`

```rust
pub fn find_spendable_outputs(
    &self,
    address: &str,
    amount: u64,
) -> Option<(u64, Vec<(String, usize)>)>
```

This is the **most important API when creating a new transaction**. It uses a greedy coin selection algorithm, accumulating UTXOs from the address one by one until the total meets `amount`.

```rust
// Need to pay 30 satoshis (including fees)
match utxo_set.find_spendable_outputs("alice", 30) {
    Some((accumulated, inputs)) => {
        // accumulated: the actual total selected (may be > 30; the difference is the change)
        // inputs: list of selected UTXOs, each item is (txid, vout)
        let change = accumulated - 30;
        println!("Selected {} UTXOs, change: {} satoshis", inputs.len(), change);
    }
    None => {
        println!("Insufficient balance");
    }
}
```

**Change mechanism**: If `accumulated > amount`, the difference must be returned to the sender as a change output. For example, to pay 3 BTC, you select a 5 BTC UTXO, and you need to create a 2 BTC change output (with the fee deducted from it).

**UTXO selection strategy comparison:**

| Strategy | Description | This Implementation |
|----------|-------------|---------------------|
| Greedy algorithm | Accumulate sequentially until the amount is met | Used here |
| Best match | Combination closest to the target amount | Reduces change |
| Smallest UTXO first | Prefer small-denomination UTXOs | Reduces fragmentation |
| Largest UTXO first | Prefer large-denomination UTXOs | Reduces number of inputs |

---

### Excluding Already-Pending UTXOs: `find_spendable_outputs_excluding`

```rust
pub fn find_spendable_outputs_excluding(
    &self,
    address: &str,
    amount: u64,
    excluded: &HashSet<String>,
) -> Option<(u64, Vec<(String, usize)>)>
```

This is an extended version of `find_spendable_outputs`. When the same wallet initiates multiple transactions in quick succession, the UTXOs already selected by prior transactions have not yet been confirmed (they are still in the mempool) and cannot be reused. By passing an `excluded` set (formatted as `"txid:vout"` strings), these UTXOs already occupied by pending transactions can be skipped.

```rust
// The first transaction used "abc:0"
let mut pending_spent: HashSet<String> = HashSet::new();
pending_spent.insert("abc:0".to_string());

// The second transaction automatically skips "abc:0"
let result = utxo_set.find_spendable_outputs_excluding("alice", 20, &pending_spent);
```

---

### Querying Balance: `get_balance`

```rust
pub fn get_balance(&self, address: &str) -> u64
```

A Bitcoin "balance" is a **computed value**, not a stored one. This method internally calls `find_utxos` and sums up the amounts of all UTXOs belonging to the address.

```rust
let balance = utxo_set.get_balance("alice");
println!("Alice's balance: {} satoshis", balance);
```

> Key insight: Querying a balance requires scanning the entire UTXO set (time complexity O(n)). Actual Bitcoin nodes use address indexes to optimize this operation.

---

### Complete Transaction Processing: `process_transaction`

```rust
pub fn process_transaction(&mut self, tx: &Transaction) -> bool
```

This is the **core function** for updating UTXO state, and it executes the following steps in order:
1. Calls `tx.verify()` to verify the transaction signature
2. If it is not a coinbase transaction, removes all UTXOs referenced by the inputs
3. Adds all of the transaction's outputs to the UTXO set

```rust
// Process a regular transaction
let success = utxo_set.process_transaction(&transfer_tx);
if !success {
    eprintln!("Transaction verification failed; UTXO set was not modified");
}
```

This function has atomic semantics — if verification fails, the UTXO set is not modified, ensuring state consistency.

---

## Double-Spend Prevention Mechanism

Double spending is the core security problem that blockchain must solve. The UTXO model naturally defends against double spending:

```
Attack flow:
1. Attacker has a UTXO worth 10 BTC (txid="xyz", vout=0)
2. Creates transaction A: spend xyz:0, pay merchant 10 BTC
3. Merchant accepts; transaction A enters the mempool
4. Attacker creates transaction B: also spend xyz:0, pay themselves 10 BTC
5. Attempts to broadcast transaction B

Defense result:
- After transaction A is confirmed, xyz:0 is deleted from the UTXO set
- When transaction B is validated, xyz:0 is not found and is rejected by the node
- Even if transaction A is unconfirmed, the mempool's double-spend detection rejects transaction B at step 3
```

In SimpleBTC's `Mempool`, `utxo_index` (`HashMap<"txid:vout", spending_txid>`) records which UTXOs have already been claimed by transactions in the mempool, enabling double-spend detection and rejection of transaction B at step 3.

---

## Complete Usage Example

```rust
use bitcoin_simulation::utxo::UTXOSet;
use bitcoin_simulation::transaction::Transaction;

fn main() {
    let mut utxo_set = UTXOSet::new();

    // Step 1: Mine a block, create a coinbase transaction (bitcoin created from scratch)
    let coinbase = Transaction::new_coinbase("alice".to_string(), 50, 0, 0);
    utxo_set.process_transaction(&coinbase);

    // Step 2: Query Alice's balance
    let alice_balance = utxo_set.get_balance("alice");
    println!("Alice balance: {} satoshis", alice_balance); // Output: 50

    // Step 3: Alice transfers 20 satoshis to Bob (fee: 2 satoshis)
    let needed = 22; // 20 for Bob + 2 fee
    if let Some((accumulated, inputs)) = utxo_set.find_spendable_outputs("alice", needed) {
        let change = accumulated - needed;
        println!("Selected {} UTXOs, total: {}, change: {}", inputs.len(), accumulated, change);

        // Build and broadcast the transaction (signature details omitted here)
        // let tx = build_transaction(inputs, "bob", 20, "alice", change, 2);
        // utxo_set.process_transaction(&tx);
    }

    // Step 4: Query Bob's balance
    // println!("Bob balance: {}", utxo_set.get_balance("bob"));
}
```

---

## Summary

The UTXO model is the cornerstone of Bitcoin's architecture. `UTXOSet` ensures ledger security through the following mechanisms:

- **`process_transaction`**: Atomically updates UTXO state (removes inputs first, then adds outputs)
- **`remove_utxo`**: Ensures each UTXO can only be spent once, preventing double spending
- **`find_spendable_outputs_excluding`**: Resolves UTXO conflicts for consecutive transactions by tracking via `pending_spent`
- **`get_balance`**: Balance is a computed value — the sum of all UTXOs belonging to that address

The next chapter will cover the specific structure of transactions and the signature verification mechanism.
