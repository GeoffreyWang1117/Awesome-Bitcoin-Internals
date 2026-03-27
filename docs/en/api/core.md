# Core Module API

The SimpleBTC core module provides fundamental Bitcoin blockchain functionality.

## Module List

### Transaction Module
- [Transaction API](./transaction.md) - Transaction structure, UTXO model, fee calculation

### Block Module
- [Block API](./block.md) - Block structure, proof of work, Merkle root

### Blockchain Module
- [Blockchain API](./blockchain.md) - Blockchain management, mining, validation

### Wallet Module
- [Wallet API](./wallet.md) - Key generation, signing, addresses

### UTXO Module
- [UTXO API](./utxo.md) - UTXO set, balance queries, double-spend protection

## Quick Index

### Common Functions

#### Create a Wallet
```rust
use bitcoin_simulation::wallet::Wallet;
let wallet = Wallet::new();
```

#### Create a Transaction
```rust
let tx = blockchain.create_transaction(
    &from_wallet,
    to_address,
    amount,
    fee
)?;
```

#### Mine
```rust
blockchain.mine_pending_transactions(miner_address)?;
```

#### Query Balance
```rust
let balance = blockchain.get_balance(&address);
```

## Data Flow

```
1. Create Wallet
   Wallet::new() → generate key pair → obtain address

2. Create Transaction
   Select UTXOs → build inputs/outputs → sign → verify

3. Add Transaction
   Validate transaction → add to pending pool → wait to be mined

4. Mine
   Collect transactions → create Coinbase → compute Merkle root → PoW → update UTXOs

5. Query
   Traverse UTXO set → accumulate balance
```

## Type Definitions

### Core Types

```rust
// Amount unit: satoshi
type Amount = u64;  // 1 BTC = 100,000,000 satoshi

// Address: 40-character hexadecimal
type Address = String;

// Hash: 64-character hexadecimal
type Hash = String;

// Unix timestamp (seconds)
type Timestamp = u64;
```

### Error Types

```rust
// All APIs return Result<T, String>
type ApiResult<T> = Result<T, String>;

// Common error messages
"Insufficient balance (including fee)"
"UTXO does not exist"
"Transaction validation failed"
"Referenced transaction does not exist"
"No pending transactions"
```

## Usage Patterns

### Basic Pattern

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn main() -> Result<(), String> {
    // 1. Initialize
    let mut blockchain = Blockchain::new();
    let wallet = Wallet::new();

    // 2. Operate
    let tx = blockchain.create_transaction(...)?;
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(...)?;

    // 3. Query
    let balance = blockchain.get_balance(&wallet.address);

    Ok(())
}
```

### Error Handling Pattern

```rust
match blockchain.create_transaction(&alice, bob_addr, 1000, 10) {
    Ok(tx) => {
        blockchain.add_transaction(tx)?;
        println!("✓ Transaction successful");
    }
    Err(e) => {
        eprintln!("✗ Error: {}", e);
        // Handle error...
    }
}
```

## Performance Considerations

### UTXO Queries
- Time complexity: O(n), where n is the total number of UTXOs
- Recommendation: Use index optimization (see `indexer.rs`)

### Mining
- Time complexity: O(2^difficulty)
- Recommendation: Difficulty 3–4 is suitable for demos; real applications require higher values

### Blockchain Validation
- Time complexity: O(n*m), where n is the block count and m is the average transaction count
- Recommendation: Validate periodically rather than after every operation

## Thread Safety

⚠️ **Note**: The current implementation is not thread-safe.

For concurrent access:

```rust
use std::sync::{Arc, Mutex};

let blockchain = Arc::new(Mutex::new(Blockchain::new()));

// In different threads
let blockchain = blockchain.clone();
let mut bc = blockchain.lock().unwrap();
bc.create_transaction(...)?;
```

## Next Steps

- View detailed API documentation for specific modules
- Read the [Advanced Module API](./advanced.md)
- Refer to [Real-World Examples](../examples/enterprise-multisig.md)

---

[Back to Documentation Home](../introduction/README.md)
