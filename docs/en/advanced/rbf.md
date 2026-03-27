# Replace-By-Fee (RBF) Mechanism

RBF (Replace-By-Fee) is a mechanism proposed in BIP125 that allows users to replace unconfirmed transactions.

## Overview

### What is RBF?

RBF allows a sender to replace an unconfirmed transaction with a new transaction carrying a higher fee.

**Scenario**:
```
1. Alice sends transaction A: 1 BTC → Bob, fee 1 sat/byte
2. Network congestion; transaction A remains unconfirmed for a long time
3. Alice sends transaction B: 1 BTC → Bob, fee 50 sat/byte
4. Miners prioritize packing transaction B (higher fee)
5. Transaction A is discarded
```

### Why is RBF Needed?

1. **Speed up confirmation**
   - Initial fee estimate was inaccurate
   - Network suddenly became congested
   - Urgent transaction needs fast confirmation

2. **Cancel a transaction**
   - Sent to the wrong address
   - Changed your mind
   - "Cancel" by sending to yourself

3. **Batch optimization**
   - Initial transaction includes some recipients
   - Add more recipients later
   - Save on total fees

---

## Technical Implementation

### RBF Signaling

**nSequence field**:
```rust
// Enable RBF
input.sequence = 0xFFFFFFFD;  // < 0xFFFFFFFE

// Disable RBF (final transaction)
input.sequence = 0xFFFFFFFF;
```

### BIP125 Rules

A replacement transaction must satisfy:

1. **Higher fee**
   ```rust
   new_tx.fee > original_tx.fee
   ```

2. **Spends the same UTXOs**
   ```rust
   new_tx.inputs == original_tx.inputs
   ```

3. **Fee increment**
   ```rust
   new_tx.fee >= original_tx.fee + min_relay_fee
   ```

4. **Does not introduce new unconfirmed UTXOs**

---

## RBFManager Implementation

### Data Structure

```rust
pub struct RBFManager {
    replaceable_txs: Vec<String>,  // List of replaceable transaction IDs
}
```

### Methods

#### `new`

```rust
pub fn new() -> Self
```

Creates a new RBF manager.

#### `mark_replaceable`

```rust
pub fn mark_replaceable(&mut self, txid: String)
```

Marks a transaction as replaceable.

**Example**:
```rust
let mut rbf = RBFManager::new();
rbf.mark_replaceable(tx.id.clone());
```

#### `is_replaceable`

```rust
pub fn is_replaceable(&self, txid: &str) -> bool
```

Checks whether a transaction is replaceable.

#### `replace_transaction`

```rust
pub fn replace_transaction(
    &mut self,
    original_txid: &str,
    new_tx: Transaction
) -> Result<(), String>
```

Replaces the original transaction with a new one.

**Validation**:
1. The original transaction must be replaceable
2. The new transaction has a higher fee
3. The new transaction is valid

---

## Use Cases

### Use Case 1: Speeding Up Confirmation

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    advanced_tx::RBFManager,
};

fn speed_up_transaction() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let bob = Wallet::new();

    // Initialize balance
    setup_balance(&mut blockchain, &alice, 10000)?;

    println!("=== RBF Transaction Acceleration Demo ===\n");

    // 1. Create a low-fee transaction
    println!("--- Step 1: Send a low-fee transaction ---");
    let slow_tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        1000,
        1,  // Low fee: 1 sat
    )?;

    println!("Original transaction:");
    println!("  ID: {}", &slow_tx.id[..16]);
    println!("  Amount: 1000 sat");
    println!("  Fee: 1 sat");
    println!("  Fee rate: {:.2} sat/byte\n", slow_tx.fee_rate());

    blockchain.add_transaction(slow_tx.clone())?;
    rbf.mark_replaceable(slow_tx.id.clone());

    // 2. Network congestion; transaction remains unconfirmed for a long time
    println!("--- Step 2: Network congestion ---");
    println!("⏰ Waiting for confirmation...");
    println!("⏰ Still unconfirmed after 10 minutes");
    println!("⚠️  Fee too low; need to accelerate\n");

    // 3. Create a high-fee replacement transaction
    println!("--- Step 3: Create replacement transaction (higher fee) ---");
    let fast_tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        1000,
        50,  // High fee: 50 sat
    )?;

    println!("Replacement transaction:");
    println!("  ID: {}", &fast_tx.id[..16]);
    println!("  Amount: 1000 sat");
    println!("  Fee: 50 sat (50x)");
    println!("  Fee rate: {:.2} sat/byte\n", fast_tx.fee_rate());

    // 4. Validate and replace
    if rbf.is_replaceable(&slow_tx.id) {
        if fast_tx.fee > slow_tx.fee {
            println!("✓ RBF conditions met:");
            println!("  New fee ({}) > Original fee ({})", fast_tx.fee, slow_tx.fee);

            // Remove the original transaction from the pending pool
            blockchain.pending_transactions.retain(|tx| tx.id != slow_tx.id);

            // Add the new transaction
            blockchain.add_transaction(fast_tx)?;

            println!("✓ Transaction replaced\n");
        }
    }

    // 5. Mine and confirm
    println!("--- Step 4: Miner packs (prioritizes high fee rate) ---");
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("✓ Transaction confirmed");
    println!("  Bob balance: {} sat", blockchain.get_balance(&bob.address));

    Ok(())
}
```

**Output**:
```
=== RBF Transaction Acceleration Demo ===

--- Step 1: Send a low-fee transaction ---
Original transaction:
  ID: abc123...
  Amount: 1000 sat
  Fee: 1 sat
  Fee rate: 0.01 sat/byte

--- Step 2: Network congestion ---
⏰ Waiting for confirmation...
⏰ Still unconfirmed after 10 minutes
⚠️  Fee too low; need to accelerate

--- Step 3: Create replacement transaction (higher fee) ---
Replacement transaction:
  ID: def456...
  Amount: 1000 sat
  Fee: 50 sat (50x)
  Fee rate: 0.50 sat/byte

✓ RBF conditions met:
  New fee (50) > Original fee (1)
✓ Transaction replaced

--- Step 4: Miner packs (prioritizes high fee rate) ---
✓ Transaction confirmed
  Bob balance: 1000 sat
```

---

### Use Case 2: Canceling a Transaction

```rust
fn cancel_transaction() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let wrong_addr = Wallet::new().address;  // Wrong address

    setup_balance(&mut blockchain, &alice, 10000)?;

    println!("=== RBF Transaction Cancellation Demo ===\n");

    // 1. Sent to the wrong address
    println!("--- Error: sent to wrong address ---");
    let wrong_tx = blockchain.create_transaction(
        &alice,
        wrong_addr.clone(),
        5000,
        10,
    )?;

    println!("Erroneous transaction:");
    println!("  Recipient: {} (wrong!)", &wrong_addr[..16]);
    println!("  Amount: 5000 sat\n");

    blockchain.add_transaction(wrong_tx.clone())?;
    rbf.mark_replaceable(wrong_tx.id.clone());

    // 2. Error discovered; attempt to cancel
    println!("--- Error discovered; attempting to cancel ---");
    println!("Strategy: send to yourself with a higher fee\n");

    // 3. Create a "cancel" transaction (send to yourself)
    let cancel_tx = blockchain.create_transaction(
        &alice,
        alice.address.clone(),  // Send to yourself
        4950,  // Slightly less (fee deducted)
        50,    // Higher fee
    )?;

    println!("Cancellation transaction:");
    println!("  Recipient: {} (yourself)", &alice.address[..16]);
    println!("  Amount: 4950 sat");
    println!("  Fee: 50 sat (5x)\n");

    // 4. Replace
    if cancel_tx.fee > wrong_tx.fee {
        blockchain.pending_transactions.retain(|tx| tx.id != wrong_tx.id);
        blockchain.add_transaction(cancel_tx)?;
        println!("✓ Transaction cancelled (actually replaced)\n");
    }

    // 5. Confirm
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("✓ Funds returned");
    println!("  Alice balance: {} sat", blockchain.get_balance(&alice.address));
    println!("  Wrong address balance: {} sat", blockchain.get_balance(&wrong_addr));

    Ok(())
}
```

---

### Use Case 3: Batch Payment Optimization

```rust
fn batch_payment_optimization() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let recipients: Vec<_> = (0..5).map(|_| Wallet::new()).collect();

    setup_balance(&mut blockchain, &alice, 100000)?;

    println!("=== RBF Batch Payment Optimization ===\n");

    // 1. Initial payment (2 recipients)
    println!("--- Initial batch payment (2 recipients) ---");
    let mut outputs = vec![
        TxOutput::new(1000, recipients[0].address.clone()),
        TxOutput::new(2000, recipients[1].address.clone()),
    ];

    // Create transaction... (simplified)
    println!("Payment:");
    println!("  Recipient 1: 1000 sat");
    println!("  Recipient 2: 2000 sat");
    println!("  Fee: 10 sat\n");

    // 2. Add more recipients
    println!("--- Add more recipients (RBF extension) ---");
    outputs.push(TxOutput::new(3000, recipients[2].address.clone()));
    outputs.push(TxOutput::new(4000, recipients[3].address.clone()));

    println!("Newly added:");
    println!("  Recipient 3: 3000 sat");
    println!("  Recipient 4: 4000 sat");
    println!("  Fee: 15 sat (only 5 sat more!)\n");

    println!("Advantages:");
    println!("  ✓ 4 transactions merged into 1");
    println!("  ✓ Fee savings (4×10 - 15 = 25 sat)");
    println!("  ✓ Block space saved");

    Ok(())
}
```

---

## Security Considerations

### ⚠️ Zero-Confirmation Transaction Risk

**Problem**: RBF makes zero-confirmation transactions insecure

```rust
// Attack scenario
// 1. Attacker: Alice → merchant Bob (1 BTC, low fee)
//    Merchant sees the transaction and ships the goods

// 2. Attacker replaces: Alice → Alice (1 BTC, high fee)
//    Funds return to attacker; merchant suffers a loss

// Defense: wait for confirmation
if confirmations < 1 {
    println!("⚠️ Warning: zero-confirmation transactions are unsafe (RBF risk)");
    println!("Recommendation: wait for at least 1 confirmation");
}
```

### Merchant Recommendations

```rust
fn accept_payment(tx: &Transaction) -> bool {
    // 1. Check whether RBF is enabled
    if is_rbf_enabled(tx) {
        println!("⚠️ Transaction has RBF enabled");

        // Option A: Reject zero-confirmation
        println!("Waiting for confirmation...");
        return false;

        // Option B: Require a higher fee
        if tx.fee_rate() < 50.0 {
            println!("Fee too low; requires >= 50 sat/byte");
            return false;
        }
    }

    // 2. Wait for sufficient confirmations
    let confirmations = get_confirmations(tx);
    if confirmations < 1 {
        return false;
    }

    true
}
```

---

## RBF vs CPFP

### Child-Pays-For-Parent (CPFP)

**CPFP**: A child transaction pays for the parent transaction's fee

```
Parent transaction: Alice → Bob (low fee)
  ↓
Child transaction: Bob → Charlie (high fee)

Miners will pack them together to collect the higher fee
```

### Comparison

| Feature | RBF | CPFP |
|---------|-----|------|
| Operator | Sender | Receiver |
| Mechanism | Replace transaction | Child transaction pulls parent |
| Fee paid by | Sender | Receiver |
| Complexity | Simple | Slightly more complex |
| Use case | Sender accelerates | Receiver accelerates |

---

## Best Practices

### 1. When to Use RBF

```rust
// ✅ Suitable scenarios for RBF
if network_congested && !urgent {
    // Send with a low fee first; accelerate when needed
    create_rbf_transaction(fee_low);
}

// ❌ Unsuitable scenarios for RBF
if urgent || large_amount {
    // Send with a high fee directly
    create_transaction(fee_high);
}
```

### 2. Fee Strategy

```rust
fn calculate_replacement_fee(original_fee: u64) -> u64 {
    // Increase by at least 50% of the original fee
    let min_increase = original_fee / 2;

    // Or reach the current recommended fee rate
    let recommended = get_recommended_fee_rate() * tx_size;

    max(original_fee + min_increase, recommended)
}
```

### 3. User Notification

```rust
fn notify_replacement(original_tx: &Transaction, new_tx: &Transaction) {
    println!("📢 Transaction has been replaced:");
    println!("  Original transaction: {}", &original_tx.id[..16]);
    println!("  New transaction: {}", &new_tx.id[..16]);
    println!("  Original fee: {} sat", original_tx.fee);
    println!("  New fee: {} sat", new_tx.fee);
    println!("  Increase: +{} sat", new_tx.fee - original_tx.fee);
}
```

---

## Implementation Example

### Complete RBF Transaction Flow

```rust
use bitcoin_simulation::advanced_tx::RBFManager;

fn rbf_complete_example() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let bob = Wallet::new();

    // 1. Initialize
    setup_balance(&mut blockchain, &alice, 10000)?;

    // 2. Create a replaceable transaction
    let tx1 = blockchain.create_transaction(&alice, bob.address.clone(), 1000, 5)?;
    blockchain.add_transaction(tx1.clone())?;
    rbf.mark_replaceable(tx1.id.clone());

    println!("✓ Original transaction created (fee: 5 sat)");

    // 3. Monitor transaction status
    std::thread::sleep(std::time::Duration::from_secs(30));

    if !is_confirmed(&blockchain, &tx1.id) {
        println!("⚠️ Still unconfirmed after 30 seconds; preparing to accelerate...");

        // 4. Create replacement transaction
        let tx2 = blockchain.create_transaction(&alice, bob.address, 1000, 50)?;

        // 5. Validate RBF rules
        if rbf.can_replace(&tx1, &tx2) {
            // 6. Execute replacement
            blockchain.pending_transactions.retain(|tx| tx.id != tx1.id);
            blockchain.add_transaction(tx2.clone())?;

            println!("✓ Transaction replaced (fee: 50 sat)");

            // 7. Confirm
            blockchain.mine_pending_transactions(alice.address)?;
            println!("✓ New transaction confirmed");
        }
    }

    Ok(())
}
```

---

## References

- [BIP125 - Opt-in RBF](https://github.com/bitcoin/bips/blob/master/bip-0125.mediawiki)
- [Transaction API](../api/transaction.md)
- [Fee Optimization](./priority.md)

---

**Summary**: RBF is a powerful tool, but be mindful of zero-confirmation transaction risks. Merchants should wait for confirmation; users should use it judiciously.

[Back to Advanced Features](./README.md)
