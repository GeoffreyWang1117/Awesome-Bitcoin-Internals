# Quick Start

This is a 5-minute tutorial that walks you through the core features of SimpleBTC.

## Your First Blockchain Program

Create a new Rust project and add the SimpleBTC dependency:

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn main() {
    println!("🚀 SimpleBTC Quick Start\n");

    // 1. Create a blockchain
    let mut blockchain = Blockchain::new();
    println!("✓ Blockchain initialized");

    // 2. Create wallets
    let alice = Wallet::new();
    let bob = Wallet::new();
    println!("✓ Two wallets created");
    println!("  Alice: {}", alice.address);
    println!("  Bob:   {}\n", bob.address);

    // 3. Alice receives initial funds (from the genesis block)
    let tx1 = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        alice.address.clone(),
        10000,  // 10,000 satoshi
        0,      // No fee (genesis transaction)
    ).unwrap();
    blockchain.add_transaction(tx1).unwrap();
    blockchain.mine_pending_transactions(alice.address.clone()).unwrap();

    println!("💰 Alice's balance: {} satoshi", blockchain.get_balance(&alice.address));

    // 4. Alice sends to Bob
    let tx2 = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        3000,   // Transfer 3000
        10,     // Fee 10
    ).unwrap();
    blockchain.add_transaction(tx2).unwrap();
    blockchain.mine_pending_transactions(bob.address.clone()).unwrap();

    // 5. View final balances
    println!("\n💼 Final balances:");
    println!("  Alice: {} satoshi", blockchain.get_balance(&alice.address));
    println!("  Bob:   {} satoshi\n", blockchain.get_balance(&bob.address));

    // 6. Validate the blockchain
    if blockchain.is_valid() {
        println!("✅ Blockchain validation passed!");
    }

    // 7. Print blockchain information
    blockchain.print_chain();
}
```

## Expected Output

```
🚀 SimpleBTC Quick Start

✓ Blockchain initialized
✓ Two wallets created
  Alice: a3f2d8c9e4b7...
  Bob:   b9e4c7d2a3f1...

Block mined: 0003ab4f9c2d...
💰 Alice's balance: 10050 satoshi

Block mined: 0007c3e8d1a9...

💼 Final balances:
  Alice: 6990 satoshi
  Bob:   3060 satoshi

✅ Blockchain validation passed!
```

## Core Concepts at a Glance

### 1. Blockchain

A blockchain is a chain-linked data structure of blocks, where each block contains multiple transactions.

```rust
let mut blockchain = Blockchain::new();
```

**Key methods**:
- `create_transaction()` - Create a transaction
- `add_transaction()` - Add to the pending pool
- `mine_pending_transactions()` - Mine and package transactions
- `get_balance()` - Query balance
- `is_valid()` - Validate the blockchain

### 2. Wallet

A wallet manages public keys, private keys, and addresses.

```rust
let wallet = Wallet::new();
println!("Address: {}", wallet.address);
println!("Public key: {}", wallet.public_key);
// Keep the private key secret!
```

**Key methods**:
- `new()` - Create a new wallet
- `sign()` - Sign data
- `verify_signature()` - Verify a signature

### 3. Transaction

A transaction is the basic unit of value transfer, using the UTXO model.

```rust
let tx = blockchain.create_transaction(
    &sender,         // Sender's wallet
    receiver_addr,   // Receiver's address
    amount,          // Amount (satoshi)
    fee,             // Fee (satoshi)
)?;
```

**A transaction contains**:
- Inputs: the UTXOs being spent
- Outputs: the new UTXOs being created
- Fee: total inputs - total outputs

### 4. Mining

Mining is the process of packaging transactions into a block via Proof of Work (PoW).

```rust
blockchain.mine_pending_transactions(miner_address)?;
```

**Mining process**:
1. Collect pending transactions
2. Create a Coinbase transaction (reward + fees)
3. Compute the Merkle root
4. Find a hash satisfying the difficulty target (adjust nonce)
5. Add the block to the chain
6. Update the UTXO set

## Advanced Examples

### Multiple Transactions

```rust
// Create multiple transactions
for i in 1..=5 {
    let tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        100 * i,
        i,  // Different fees
    )?;
    blockchain.add_transaction(tx)?;
}

// Package all transactions at once
blockchain.mine_pending_transactions(miner.address.clone())?;
```

### Fee-Rate Priority

```rust
// Low-fee transaction
let slow_tx = blockchain.create_transaction(&alice, bob.address.clone(), 1000, 1)?;

// High-fee transaction
let fast_tx = blockchain.create_transaction(&alice, charlie.address.clone(), 1000, 50)?;

blockchain.add_transaction(slow_tx)?;
blockchain.add_transaction(fast_tx)?;

// Miners will prioritize fast_tx (higher fee rate)
blockchain.mine_pending_transactions(miner.address)?;
```

### Balance Query

```rust
let balance = blockchain.get_balance(&alice.address);
println!("Balance: {} satoshi ({:.8} BTC)", balance, balance as f64 / 100_000_000.0);
```

## REST API Usage

Start the API server:

```bash
cargo run --bin btc-server
```

### Create a Wallet

```bash
curl -X POST http://localhost:3000/api/wallet/create
```

Response:
```json
{
  "address": "a3f2d8c9e4b7...",
  "public_key": "04f9a...",
  "private_key": "keep your private key safe"
}
```

### Create a Transaction

```bash
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice_address",
    "to": "bob_address",
    "amount": 5000,
    "fee": 10
  }'
```

### Query Balance

```bash
curl http://localhost:3000/api/balance/alice_address
```

### Mine a Block

```bash
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{
    "miner_address": "miner_address"
  }'
```

### Get Blockchain Information

```bash
curl http://localhost:3000/api/blockchain/info
```

Response:
```json
{
  "chain_length": 3,
  "difficulty": 3,
  "pending_transactions": 2,
  "latest_block": {
    "index": 2,
    "hash": "0003ab4f...",
    "timestamp": 1703001234,
    "transaction_count": 5
  }
}
```

## Electron GUI

Launch the graphical interface:

```bash
cd frontend
npm install
npm start
```

GUI features:
- 📊 **Blockchain Explorer**: Visualize all blocks
- 👛 **Wallet Management**: Create and import wallets
- 💸 **Send Transactions**: Create transactions graphically
- ⛏️ **Mining**: Click a button to start mining
- 🎮 **Demo Mode**: Run the full demo with one click

## Practical Examples

SimpleBTC provides three complete practical examples:

### 1. Enterprise Multi-Signature Wallet (2-of-3)

```bash
cargo run --example enterprise_multisig
```

What you will learn:
- Creating a multi-signature address
- Collecting signatures
- Enterprise fund management

### 2. Escrow Service

```bash
cargo run --example escrow_service
```

What you will learn:
- Buyer/seller transactions
- Arbitration mechanism
- Dispute resolution

### 3. Time-Deposit Savings

```bash
cargo run --example timelock_savings
```

What you will learn:
- Setting up time locks
- Expiry checks
- Enforced saving

## Next Steps

Now that you have mastered the basics, you can dive deeper:

1. **Understand the Principles** - [Core Concepts](./concepts.md)
   - UTXO model in depth
   - Proof of Work mechanics
   - Merkle tree structure

2. **Core Features** - [Core Module Guide](./wallet.md)
   - In-depth wallet usage
   - Advanced transaction features
   - Blockchain operations

3. **Advanced Features** - [Advanced Functionality](../advanced/merkle.md)
   - Merkle tree and SPV
   - Multi-signature
   - Replace-By-Fee
   - Time locks

4. **API Reference** - [API Docs](../api/core.md)
   - Complete API documentation
   - Function signatures
   - Usage examples

## Tips

💡 **Tips**:
- Mining difficulty 3-4 is suitable for demos; 6+ is closer to reality
- Higher fees lead to faster transaction confirmation
- Call `is_valid()` regularly to verify blockchain integrity
- Use `print_chain()` to view detailed information

⚠️ **Caution**:
- A lost private key cannot be recovered
- This project is for learning only — do not use it in production
- The simplified cryptographic implementation is not as secure as real Bitcoin

---

Ready to explore further? Continue reading [Core Concepts](./concepts.md)!
