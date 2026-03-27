# Frequently Asked Questions (FAQ)

## Installation and Configuration

### Q: How do I install SimpleBTC?

**A:**
```bash
# 1. Make sure Rust is installed
rustc --version

# 2. Clone the project
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC

# 3. Build
cargo build --release

# 4. Run
cargo run --bin btc-demo
```

See the [Installation Guide](../guide/installation.md) for details.

---

### Q: I get a "linker 'cc' not found" error when compiling.

**A:** You need to install a C compiler:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Windows
# Install Visual Studio Build Tools
```

---

### Q: How do I change the mining difficulty?

**A:** Edit `src/blockchain.rs`:

```rust
pub fn new() -> Blockchain {
    Blockchain {
        difficulty: 3,  // change this value
        // 3-4 is good for demos
        // 5-6 is more secure but slower
        // ...
    }
}
```

---

## Basic Concepts

### Q: What is UTXO? Why not use account balances?

**A:** UTXO (Unspent Transaction Output) is a core Bitcoin concept.

**Account model** (Ethereum):
```
Alice account: 100 BTC
After transfer:
Alice: 70 BTC
Bob: 30 BTC
```

**UTXO model** (Bitcoin):
```
Alice has UTXOs: [50 BTC, 30 BTC, 20 BTC]
Transferring 30 BTC to Bob:
  - Consume the 50 BTC UTXO
  - Create 30 BTC output for Bob
  - Create 20 BTC change output for Alice (50 - 30)
```

**Advantages**:
- Better privacy (use a new address each time)
- Parallel processing (different UTXOs can be handled concurrently)
- Simpler validation logic

See [Basic Concepts - UTXO](../guide/concepts.md#utxo-model) for details.

---

### Q: Why do transactions need fees?

**A:** Transaction fees serve several purposes:

1. **Prevent spam attacks** — sending a transaction has a cost
2. **Incentivize miners** — miners prioritize transactions with higher fee rates
3. **Resource allocation** — when the network is congested, those willing to pay more get priority

**Fee calculation**:
```rust
fee = total inputs - total outputs

// Example
inputs: 100 satoshi
outputs: 90 satoshi
fee: 10 satoshi
```

**Fee rate recommendations**:
- 1–5 sat/byte: low priority (several hours)
- 10–20 sat/byte: medium priority (30–60 minutes)
- 50+ sat/byte: high priority (next block)

---

### Q: What is Proof of Work (PoW)? Why is mining necessary?

**A:** PoW is Bitcoin's consensus mechanism.

**Mining process**:
```rust
target = "000..."  // difficulty requirement

while hash(block_data + nonce) >= target {
    nonce++;  // keep trying
}
// Found a valid nonce; the block is accepted
```

**Why it is needed**:
- Prevents spam blocks (creating a block requires computational cost)
- Decentralized consensus (hash power as votes)
- Extremely high cost for a 51% attack (requires more than half the global hash power)

**Difficulty and time**:
- Difficulty 3: milliseconds (demo)
- Difficulty 10: seconds (private chain)
- Difficulty 20: minutes (Bitcoin scale)

See [Basic Concepts - PoW](../guide/concepts.md#proof-of-work) for details.

---

## Usage Questions

### Q: How do I create a wallet?

**A:**
```rust
use bitcoin_simulation::wallet::Wallet;

// Create a new wallet
let wallet = Wallet::new();

println!("Address: {}", wallet.address);
println!("Public key: {}", wallet.public_key);
// Keep the private key secret!
```

**Important**:
- Losing the private key = permanent loss of bitcoin
- Exposing the private key = bitcoin theft
- Back up the private key to a secure location

---

### Q: How do I send a transfer?

**A:**
```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

let mut blockchain = Blockchain::new();
let alice = Wallet::new();
let bob = Wallet::new();

// 1. Create the transaction
let tx = blockchain.create_transaction(
    &alice,           // sender
    bob.address,      // recipient
    1000,            // amount (satoshi)
    10,              // fee
)?;

// 2. Add to the pending pool
blockchain.add_transaction(tx)?;

// 3. Mine to confirm
blockchain.mine_pending_transactions(miner.address)?;
```

---

### Q: What do I do if my balance is insufficient?

**A:** Check the following:

1. **Query balance**:
```rust
let balance = blockchain.get_balance(&address);
println!("Balance: {}", balance);
```

2. **Ensure you have UTXOs**:
```rust
let utxos = blockchain.utxo_set.find_utxos(&address);
println!("UTXO count: {}", utxos.len());
```

3. **Check that the fee is included**:
```rust
let total_needed = amount + fee;
if balance < total_needed {
    return Err("Insufficient balance (including fee)");
}
```

4. **Wait for transaction confirmation**:
A recently sent transaction must be confirmed by mining before its outputs can be used.

---

### Q: What if a transaction remains unconfirmed for a long time?

**A:** Possible causes and solutions:

**Cause 1: Fee too low**
```rust
// Increase the fee
let tx = blockchain.create_transaction(
    &alice,
    bob.address,
    1000,
    50,  // higher fee
)?;
```

**Cause 2: No miner is mining**
```bash
# Mine manually
cargo run --bin btc-demo
# Or in code
blockchain.mine_pending_transactions(miner.address)?;
```

**Cause 3: Transaction is invalid**
```rust
// Validate the transaction
if !tx.verify() {
    println!("Transaction invalid, check:");
    println!("- Whether the input UTXOs exist");
    println!("- Whether the signature is correct");
    println!("- Whether the balance is sufficient");
}
```

**Use RBF to speed up**:
```rust
// Create a replacement transaction with a higher fee rate
let faster_tx = blockchain.create_transaction(
    &alice,
    bob.address,
    1000,
    100,  // higher fee
)?;
```

---

## Advanced Features

### Q: How do I use multisig?

**A:**
```rust
use bitcoin_simulation::multisig::MultiSigAddress;

// 1. Create participant wallets
let alice = Wallet::new();
let bob = Wallet::new();
let charlie = Wallet::new();

// 2. Create a 2-of-3 multisig address
let multisig = MultiSigAddress::new(
    2,  // requires 2 signatures
    vec![
        alice.public_key,
        bob.public_key,
        charlie.public_key,
    ]
)?;

// 3. Send funds to the multisig address
let tx = blockchain.create_transaction(
    &funder,
    multisig.address.clone(),
    10000,
    0,
)?;

// 4. Spend from the multisig address (requires 2 signatures)
let alice_sig = alice.sign(&payment_data);
let bob_sig = bob.sign(&payment_data);

if vec![alice_sig, bob_sig].len() >= multisig.required_sigs {
    // Execute the transaction
}
```

See the [Multisig Tutorial](../advanced/multisig.md) for details.

---

### Q: What is a Merkle tree? What is it used for?

**A:** A Merkle tree is a hash tree of transactions, stored in the block header.

**Structure**:
```
        Root Hash
       /         \
     H(AB)      H(CD)
    /    \      /    \
  H(A)  H(B)  H(C)  H(D)
   tx1   tx2   tx3   tx4
```

**Uses**:

1. **SPV verification** — light wallets do not need to download the full block
```rust
// Only needs block header + Merkle proof
let proof = merkle_tree.get_proof(&tx_hash)?;
let valid = MerkleTree::verify_proof(
    &tx_hash,
    &proof,
    &block.merkle_root,
    tx_index
);
```

2. **Data integrity** — any change to a transaction changes the root hash

3. **Efficient verification** — O(log n) complexity

See the [Merkle Tree Tutorial](../advanced/merkle.md) for details.

---

### Q: What is a timelock? How do I use one?

**A:** A timelock restricts a transaction from being confirmed before a specific time.

**Two types**:

1. **Timestamp-based**:
```rust
use bitcoin_simulation::advanced_tx::TimeLock;

// Unlock after 3 months
let three_months = 90 * 24 * 3600;
let unlock_time = current_time + three_months;
let timelock = TimeLock::new_time_based(unlock_time);

// Check if it has matured
if timelock.is_mature(current_time, 0) {
    println!("Matured, funds can be spent");
}
```

2. **Block height-based**:
```rust
// Unlock after block 100,000
let timelock = TimeLock::new_block_based(100000);

if timelock.is_mature(current_time, current_block_height) {
    println!("Block height reached");
}
```

**Use cases**:
- Time deposits
- Inheritance
- Payroll disbursement
- Project vesting periods

See the [Timelock Tutorial](../advanced/timelock.md) for details.

---

## Development Questions

### Q: How do I integrate SimpleBTC into my project?

**A:** SimpleBTC can be used as a library:

```toml
# Cargo.toml
[dependencies]
bitcoin_simulation = { path = "../SimpleBTC" }
```

```rust
// In your code
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn my_app() {
    let blockchain = Blockchain::new();
    // ... your business logic
}
```

---

### Q: How do I use the REST API?

**A:**

**Start the server**:
```bash
cargo run --bin btc-server
# Server runs at http://localhost:3000
```

**API call examples**:

```bash
# Create a wallet
curl -X POST http://localhost:3000/api/wallet/create

# Create a transaction
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice_address",
    "to": "bob_address",
    "amount": 1000,
    "fee": 10
  }'

# Query balance
curl http://localhost:3000/api/balance/alice_address

# Mine
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address": "miner_address"}'
```

See the [REST API documentation](../api/rest.md) for details.

---

### Q: How do I run tests?

**A:**
```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_blockchain

# Show output
cargo test -- --nocapture

# Run examples
cargo run --example enterprise_multisig
cargo run --example escrow_service
cargo run --example timelock_savings
```

---

### Q: How do I deploy the documentation site?

**A:**

**Local preview**:
```bash
cd docs
mdbook serve --open
```

**GitHub Pages deployment**:
```bash
# Build
mdbook build

# Deploy to the gh-pages branch
# See docs/README.md
```

**Docker deployment**:
```dockerfile
FROM nginx:alpine
COPY docs/book /usr/share/nginx/html
EXPOSE 80
```

See the [Documentation Deployment Guide](../../../docs/README.md) for details.

---

## Performance Questions

### Q: Mining is too slow. What can I do?

**A:** Adjust the difficulty:

```rust
// In blockchain.rs
blockchain.difficulty = 3;  // lower the difficulty
// 3: milliseconds
// 4: seconds
// 5: several seconds
// 6+: may be very slow
```

Or use Release mode:
```bash
cargo run --release --bin btc-demo
# Release mode is much faster than Debug
```

---

### Q: Balance queries are slow. How can I speed them up?

**A:** Use the built-in indexer:

```rust
// SimpleBTC has a built-in indexer
let txs = blockchain.indexer.get_transactions_by_address(&address);

// Or cache balances
let balance_cache: HashMap<String, u64> = HashMap::new();
```

---

## Security Questions

### Q: Is SimpleBTC secure? Can it be used in production?

**A:** **SimpleBTC is an educational project and is not recommended for production use.**

**Differences from real Bitcoin**:
- Simplified cryptography (SHA256 instead of ECDSA)
- No P2P network layer
- Simplified script system
- No full SPV implementation
- JSON storage (should use LevelDB)

**Requirements for production use**:
- Implement full secp256k1 elliptic curve
- Implement ECDSA signature verification
- Add a P2P network protocol
- Use a professional database
- Full Script engine
- Security audit

---

### Q: How do I protect my private key?

**A:** Private key security recommendations:

1. **Never share the private key**
2. **Multiple backups**:
   - Paper wallet (fire- and water-resistant)
   - Hardware wallet
   - Encrypted USB drive
3. **Distributed storage**:
   - Home safe
   - Bank safe deposit box
   - Offsite backup
4. **Use multisig**:
   - 2-of-3 reduces single-point-of-failure risk
5. **Regularly test recovery**

---

## Other Questions

### Q: What are the differences between SimpleBTC and real Bitcoin?

**A:**

| Feature | SimpleBTC | Real Bitcoin |
|---------|-----------|-------------|
| Cryptography | SHA256 (simplified) | secp256k1 ECDSA |
| Consensus | PoW (simplified) | PoW (full) |
| Script | Simplified | Full Script language |
| Network | None | P2P network |
| Storage | JSON | LevelDB |
| Difficulty adjustment | Fixed | Every 2016 blocks |

**Value of SimpleBTC**:
- Learn Bitcoin principles
- Understand the UTXO model
- Practice blockchain development
- Rapid prototype validation

---

### Q: How do I contribute code?

**A:**

1. Fork the project
2. Create a feature branch
3. Submit a Pull Request
4. Wait for review

See the [Contributing Guide](./contributing.md) for details.

---

### Q: What do I do if I find a bug?

**A:**

1. Open an issue on GitHub:
   https://github.com/GeoffreyWang1117/SimpleBTC/issues

2. Include the following information:
   - Operating system
   - Rust version
   - Error message
   - Steps to reproduce
   - Relevant code

---

### Q: Where can I get help?

**A:**

- **Documentation**: this site
- **GitHub Issues**: report problems and suggestions
- **Rust community**: https://users.rust-lang.org/
- **Bitcoin whitepaper**: https://bitcoin.org/bitcoin.pdf

---

## More Resources

- [Quick Start](../guide/quickstart.md)
- [Basic Concepts](../guide/concepts.md)
- [API Documentation](../api/core.md)
- [Advanced Features](../advanced/multisig.md)
- [Practical Examples](../examples/enterprise-multisig.md)

---

**Didn't find your question?** [Open an issue on GitHub](https://github.com/GeoffreyWang1117/SimpleBTC/issues)
