# Transaction API

The transaction module provides a complete implementation of the Bitcoin UTXO model.

## Data Structures

### `TxInput`

A transaction input that references a previously unspent output (UTXO).

```rust
pub struct TxInput {
    pub txid: String,        // ID of the referenced transaction
    pub vout: usize,         // Output index
    pub signature: String,   // Digital signature
    pub pub_key: String,     // Public key
}
```

**Methods**:

#### `new`

```rust
pub fn new(
    txid: String,
    vout: usize,
    signature: String,
    pub_key: String
) -> Self
```

Creates a new transaction input.

**Parameters**:
- `txid` - ID of the referenced transaction
- `vout` - Output index number
- `signature` - Signature generated with the private key
- `pub_key` - Corresponding public key

**Example**:
```rust
let input = TxInput::new(
    "abc123...".to_string(),
    0,
    wallet.sign("data"),
    wallet.public_key.clone()
);
```

---

### `TxOutput`

A transaction output representing an unspent amount (UTXO).

```rust
pub struct TxOutput {
    pub value: u64,              // Amount (satoshi)
    pub pub_key_hash: String,    // Recipient address
}
```

**Methods**:

#### `new`

```rust
pub fn new(value: u64, address: String) -> Self
```

Creates a new transaction output.

**Parameters**:
- `value` - Output amount (satoshi)
- `address` - Recipient address

**Example**:
```rust
let output = TxOutput::new(5000, bob_address);
```

#### `can_be_unlocked_with`

```rust
pub fn can_be_unlocked_with(&self, address: &str) -> bool
```

Checks whether this output can be unlocked by the specified address.

**Parameters**:
- `address` - The address to check

**Return value**:
- `true` - Address matches
- `false` - Address does not match

**Example**:
```rust
if output.can_be_unlocked_with(&alice.address) {
    println!("Alice can spend this output");
}
```

---

### `Transaction`

The complete transaction structure.

```rust
pub struct Transaction {
    pub id: String,                 // Transaction ID
    pub inputs: Vec<TxInput>,       // List of inputs
    pub outputs: Vec<TxOutput>,     // List of outputs
    pub timestamp: u64,             // Unix timestamp
    pub fee: u64,                   // Transaction fee
}
```

**Methods**:

#### `new`

```rust
pub fn new(
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    timestamp: u64,
    fee: u64
) -> Self
```

Creates a new transaction.

**Parameters**:
- `inputs` - List of transaction inputs
- `outputs` - List of transaction outputs
- `timestamp` - Unix timestamp
- `fee` - Transaction fee (satoshi)

**Return value**:
- A newly created transaction instance with an automatically computed ID

**Example**:
```rust
let tx = Transaction::new(
    vec![input1, input2],
    vec![output1, output2],
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    10
);
```

#### `new_coinbase`

```rust
pub fn new_coinbase(
    to: String,
    reward: u64,
    timestamp: u64,
    total_fees: u64
) -> Self
```

Creates a Coinbase transaction (mining reward).

**Parameters**:
- `to` - Miner address
- `reward` - Block reward (excluding fees)
- `timestamp` - Unix timestamp
- `total_fees` - Sum of all transaction fees in the block

**Return value**:
- Coinbase transaction instance

**Example**:
```rust
let coinbase = Transaction::new_coinbase(
    miner.address,
    50,
    timestamp,
    total_fees
);
```

#### `calculate_hash`

```rust
pub fn calculate_hash(&self) -> String
```

Computes the transaction hash (transaction ID).

**Return value**:
- 64-character hexadecimal hash string

**Notes**:
- Uses the SHA256 algorithm
- Includes all transaction data (inputs, outputs, timestamp, fee)
- Any change in data will produce a completely different hash

**Example**:
```rust
let tx_id = tx.calculate_hash();
println!("Transaction ID: {}", tx_id);
```

#### `is_coinbase`

```rust
pub fn is_coinbase(&self) -> bool
```

Checks whether this is a Coinbase transaction.

**Return value**:
- `true` - Coinbase transaction
- `false` - Regular transaction

**Criteria**:
- Exactly one input
- That input's txid is empty

**Example**:
```rust
if tx.is_coinbase() {
    println!("This is a mining reward transaction");
} else {
    println!("This is a regular transaction");
}
```

#### `verify`

```rust
pub fn verify(&self) -> bool
```

Validates the transaction (simplified version).

**Validation items**:
1. Coinbase transactions are always valid
2. Checks for at least one input and one output
3. Checks that the signature and public key are non-empty

**Return value**:
- `true` - Transaction is valid
- `false` - Transaction is invalid

**Note**:
Real Bitcoin also requires:
- ECDSA signature correctness
- UTXO existence
- Amount balance
- Script execution

**Example**:
```rust
if tx.verify() {
    blockchain.add_transaction(tx)?;
} else {
    return Err("Invalid transaction".to_string());
}
```

#### `size`

```rust
pub fn size(&self) -> usize
```

Calculates the transaction size in bytes.

**Return value**:
- Byte size of the transaction

**Use cases**:
- Computing fee rate (sat/byte)
- Estimating block space usage
- Fee estimation

**Example**:
```rust
let size = tx.size();
println!("Transaction size: {} bytes", size);
```

#### `fee_rate`

```rust
pub fn fee_rate(&self) -> f64
```

Calculates the transaction fee rate (satoshi/byte).

**Return value**:
- Fee rate (sat/byte)

**Formula**:
```
fee_rate = fee / size
```

**Fee rate reference**:
- 1–5 sat/byte: Low priority
- 5–20 sat/byte: Medium priority
- 20–50 sat/byte: High priority
- 50+ sat/byte: Urgent

**Example**:
```rust
let rate = tx.fee_rate();
println!("Fee rate: {:.2} sat/byte", rate);

if rate < 5.0 {
    println!("Warning: Low fee rate, confirmation may be slow");
}
```

#### `output_sum`

```rust
pub fn output_sum(&self) -> u64
```

Gets the total amount of all outputs.

**Return value**:
- Total output amount (satoshi)

**Use cases**:
- Verifying transaction balance
- Computing the actual fee

**Formula**:
```
fee = input_sum - output_sum
```

**Example**:
```rust
let output_total = tx.output_sum();
let fee = input_total - output_total;
println!("Fee: {}", fee);
```

## Usage Examples

### Creating a Simple Transaction

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn main() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let alice = Wallet::new();
    let bob = Wallet::new();

    // Alice receives initial funds
    let init_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis".to_string()),
        alice.address.clone(),
        10000,
        0,
    )?;
    blockchain.add_transaction(init_tx)?;
    blockchain.mine_pending_transactions(alice.address.clone())?;

    // Alice sends to Bob
    let tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        3000,  // amount
        10,    // fee
    )?;

    // View transaction details
    println!("Transaction ID: {}", tx.id);
    println!("Number of inputs: {}", tx.inputs.len());
    println!("Number of outputs: {}", tx.outputs.len());
    println!("Fee: {}", tx.fee);
    println!("Fee rate: {:.2} sat/byte", tx.fee_rate());

    // Add to the blockchain
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(bob.address)?;

    Ok(())
}
```

### Batch Transactions

```rust
// Create multiple transactions to test fee priority
let transactions = vec![
    (bob.address.clone(), 1000, 1),   // Low fee
    (charlie.address.clone(), 2000, 50), // High fee
    (david.address.clone(), 3000, 5), // Medium fee
];

for (to, amount, fee) in transactions {
    let tx = blockchain.create_transaction(&alice, to, amount, fee)?;
    blockchain.add_transaction(tx)?;
}

// Mining sorts transactions by fee rate, highest first
blockchain.mine_pending_transactions(miner.address)?;
```

### Manually Building a Transaction

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use bitcoin_simulation::transaction::{Transaction, TxInput, TxOutput};

// 1. Create input (requires knowledge of the prior UTXO)
let input = TxInput::new(
    "previous_tx_id".to_string(),
    0,  // vout
    alice.sign("tx_data"),
    alice.public_key.clone(),
);

// 2. Create outputs
let output1 = TxOutput::new(3000, bob.address);     // To Bob
let output2 = TxOutput::new(6990, alice.address);   // Change

// 3. Assemble transaction
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

let tx = Transaction::new(
    vec![input],
    vec![output1, output2],
    timestamp,
    10,  // fee
);

// 4. Verify and add
if tx.verify() {
    blockchain.add_transaction(tx)?;
}
```

## Error Handling

```rust
match blockchain.create_transaction(&alice, bob.address, 1000, 10) {
    Ok(tx) => {
        println!("✓ Transaction created successfully");
        blockchain.add_transaction(tx)?;
    }
    Err(e) => {
        eprintln!("❌ Transaction creation failed: {}", e);
        // Common errors:
        // - "Insufficient balance (including fee)"
        // - "UTXO does not exist"
        // - "Referenced transaction does not exist"
    }
}
```

## Best Practices

### 1. Setting Fees

```rust
// Set fee based on urgency
let size = estimate_tx_size(inputs_count, outputs_count);

let fee = match urgency {
    Urgency::Low => size * 1,      // 1 sat/byte
    Urgency::Medium => size * 10,  // 10 sat/byte
    Urgency::High => size * 50,    // 50 sat/byte
};
```

### 2. UTXO Selection

```rust
// Prefer small UTXOs to avoid fragmentation
let utxos = blockchain.utxo_set.find_spendable_outputs(&address, amount)?;
println!("Used {} UTXOs", utxos.1.len());
```

### 3. Transaction Validation

```rust
// Validate immediately after creating a transaction
let tx = Transaction::new(...);
assert!(tx.verify(), "Transaction validation failed");
assert!(tx.fee_rate() >= 1.0, "Fee rate too low");
```

## References

- [Blockchain API](./blockchain.md) - Blockchain transaction management
- [UTXO API](./utxo.md) - UTXO set operations
- [Wallet API](./wallet.md) - Wallet and signing

---

[Back to API Index](./core.md)
