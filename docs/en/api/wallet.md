# Wallet API

The wallet module is responsible for managing key pairs, addresses, and signatures.

## Data Structures

### `Wallet`

```rust
pub struct Wallet {
    pub address: String,        // Wallet address (public key hash)
    pub private_key: String,    // Private key
    pub public_key: String,     // Public key
}
```

## Methods

### Creating a Wallet

#### `new`

```rust
pub fn new() -> Self
```

Creates a new wallet, automatically generating a key pair and address.

**Key generation flow**:
1. Generate a random private key (64-character hexadecimal)
2. Derive the public key from the private key (SHA256)
3. Derive the address from the public key hash (first 40 characters)

**Return value**: New wallet instance

**Security notes**:
- ⚠️ The private key must be kept secret
- ⚠️ A lost private key cannot be recovered
- ⚠️ Back up to a secure location

**Example**:
```rust
use bitcoin_simulation::wallet::Wallet;

// Create a new wallet
let wallet = Wallet::new();

println!("Address: {}", wallet.address);
println!("Public key: {}", wallet.public_key);
// Never print or share the private key!

// Multiple wallets
let alice = Wallet::new();
let bob = Wallet::new();
let charlie = Wallet::new();
```

---

#### `from_address`

```rust
pub fn from_address(address: String) -> Self
```

Creates a wallet from a known address (for demonstration only).

**Note**:
- This generates a new random key pair
- The keys do not correspond to the address
- For testing and demo purposes only

**Parameters**:
- `address` - The specified address string

**Return value**: Wallet instance (keys are newly generated)

**Example**:
```rust
// Used for demonstrating the genesis address
let genesis = Wallet::from_address("genesis_address".to_string());

// In real applications, restore from private key
// let wallet = Wallet::from_private_key(private_key);
```

---

### Signing Operations

#### `sign`

```rust
pub fn sign(&self, data: &str) -> String
```

Signs data with the private key.

**Signing process** (simplified):
```
signature = SHA256(private_key + data)
```

**Real Bitcoin** uses ECDSA:
```
1. Double SHA256 the data
2. Generate signature using private key and secp256k1 curve
3. Signature contains two parts: r and s
```

**Parameters**:
- `data` - Data to sign (usually transaction data)

**Return value**: Signature string (64-character hexadecimal)

**Use cases**:
- Proving ownership of the private key
- Authorizing transactions
- Preventing transaction tampering

**Example**:
```rust
let wallet = Wallet::new();

// Sign transaction data
let tx_data = "send 100 BTC to Bob";
let signature = wallet.sign(tx_data);

println!("Signature: {}", signature);

// Use in a transaction
let input = TxInput::new(
    prev_txid,
    vout,
    wallet.sign(&tx_data),  // signature
    wallet.public_key.clone(),
);
```

---

#### `verify_signature` (static method)

```rust
pub fn verify_signature(
    public_key: &str,
    data: &str,
    signature: &str
) -> bool
```

Verifies whether a signature is valid (simplified version).

**Verification process** (simplified):
- Checks that the public key and signature are non-empty

**Real Bitcoin** uses ECDSA verification:
1. Recover the public key from the signature
2. Verify the public key matches
3. Verify the mathematical correctness of the signature

**Parameters**:
- `public_key` - Signer's public key
- `data` - Original data
- `signature` - Signature

**Return value**:
- `true` - Signature is valid
- `false` - Signature is invalid

**Example**:
```rust
let wallet = Wallet::new();
let data = "transaction data";
let signature = wallet.sign(data);

// Verify signature
if Wallet::verify_signature(&wallet.public_key, data, &signature) {
    println!("✓ Signature valid");
} else {
    println!("✗ Signature invalid");
}

// Used in transaction validation
for input in transaction.inputs {
    if !Wallet::verify_signature(&input.pub_key, &tx_data, &input.signature) {
        return Err("Signature verification failed");
    }
}
```

---

## Address Formats

### SimpleBTC Address

```
Format: 40-character hexadecimal string
Example: a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c
```

### Real Bitcoin Addresses

#### P2PKH (starts with 1)
```
1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
```
**Generation process**:
```
Public key → SHA256 → RIPEMD160 → Add version → Checksum → Base58 encoding
```

#### P2SH (starts with 3)
```
3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy
```
**Use case**: Multisig, script addresses

#### Bech32 (starts with bc1)
```
bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
```
**Advantage**: SegWit address, lower fees

---

## Key Management

### Private Key Security

**Best practices**:

```rust
// ✅ Good practice
let wallet = Wallet::new();

// Encrypt and store the private key
let encrypted = encrypt_private_key(&wallet.private_key, password);
save_to_secure_storage(&encrypted);

// Clear memory immediately after use
drop(wallet);

// Back up to multiple locations
backup_to_hardware_wallet(&wallet.private_key);
backup_to_paper(&wallet.private_key);
backup_to_encrypted_usb(&wallet.private_key);
```

```rust
// ❌ Bad practice
println!("Private key: {}", wallet.private_key);  // Never print
save_to_file(&wallet.private_key);        // Plaintext storage
send_via_email(&wallet.private_key);      // Network transmission
```

### Key Recovery

```rust
// Recover wallet from private key (needs implementation)
fn recover_wallet(private_key: &str) -> Wallet {
    // 1. Validate private key format
    // 2. Derive public key from private key
    // 3. Generate address from public key
    // 4. Return wallet instance
}

// Using mnemonic phrase (BIP39 standard, needs implementation)
fn from_mnemonic(words: &str) -> Wallet {
    // Mnemonic → seed → master private key → derived keys
}
```

---

## Use Cases

### Case 1: Basic Transfer

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn basic_transfer() -> Result<(), String> {
    let mut blockchain = Blockchain::new();

    // Create participants
    let alice = Wallet::new();
    let bob = Wallet::new();

    // Alice receives initial funds
    setup_balance(&mut blockchain, &alice, 10000)?;

    // Alice transfers to Bob
    let tx = blockchain.create_transaction(
        &alice,              // from_wallet
        bob.address.clone(),
        5000,               // amount
        10,                 // fee
    )?;

    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(alice.address.clone())?;

    // Check balances
    println!("Alice: {}", blockchain.get_balance(&alice.address));
    println!("Bob: {}", blockchain.get_balance(&bob.address));

    Ok(())
}
```

### Case 2: Batch Wallet Creation

```rust
fn create_wallet_pool(count: usize) -> Vec<Wallet> {
    let mut wallets = Vec::new();

    for i in 0..count {
        let wallet = Wallet::new();
        println!("Wallet #{}: {}", i, &wallet.address[..16]);
        wallets.push(wallet);
    }

    wallets
}

// Usage
let users = create_wallet_pool(100);  // Create 100 wallets
```

### Case 3: Wallet Import/Export

```rust
use serde_json;

// Export wallet (encrypted)
fn export_wallet(wallet: &Wallet, password: &str) -> Result<String, String> {
    let wallet_json = serde_json::to_string(wallet)?;
    let encrypted = encrypt(&wallet_json, password);
    Ok(encrypted)
}

// Import wallet
fn import_wallet(encrypted_data: &str, password: &str) -> Result<Wallet, String> {
    let decrypted = decrypt(encrypted_data, password)?;
    let wallet: Wallet = serde_json::from_str(&decrypted)?;
    Ok(wallet)
}

// Usage
let wallet = Wallet::new();
let backup = export_wallet(&wallet, "strong_password")?;
save_to_file("wallet_backup.enc", &backup)?;

// Restore
let backup_data = read_from_file("wallet_backup.enc")?;
let recovered = import_wallet(&backup_data, "strong_password")?;
```

### Case 4: Multisig Wallet Integration

```rust
use bitcoin_simulation::multisig::MultiSigAddress;

fn create_multisig_wallet() -> Result<MultiSigAddress, String> {
    // Create participant wallets
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    // Collect public keys
    let public_keys = vec![
        alice.public_key,
        bob.public_key,
        charlie.public_key,
    ];

    // Create 2-of-3 multisig address
    let multisig = MultiSigAddress::new(2, public_keys)?;

    println!("Multisig address: {}", multisig.address);

    Ok(multisig)
}
```

---

## Differences from Real Bitcoin

| Feature | SimpleBTC | Real Bitcoin |
|---------|-----------|--------------|
| Private key generation | Random string | 256-bit random number |
| Public key derivation | SHA256 | secp256k1 elliptic curve |
| Address format | 40-character hex | Base58/Bech32 encoding |
| Signing algorithm | SHA256 | ECDSA |
| Signature verification | Simplified check | Full mathematical verification |

**Real Bitcoin flow**:
```
Private key (256 bits)
  ↓ secp256k1
Public key (33/65 bytes)
  ↓ SHA256 + RIPEMD160
Public key hash (20 bytes)
  ↓ Version + checksum + Base58
Address (25-34 chars)
```

---

## Security Recommendations

### 1. Private Key Protection

```rust
// Use OS keyring
use keyring::Entry;

fn store_private_key(address: &str, private_key: &str) -> Result<(), String> {
    let entry = Entry::new("SimpleBTC", address)?;
    entry.set_password(private_key)?;
    Ok(())
}

fn retrieve_private_key(address: &str) -> Result<String, String> {
    let entry = Entry::new("SimpleBTC", address)?;
    let private_key = entry.get_password()?;
    Ok(private_key)
}
```

### 2. Multiple Backups

- ✅ Paper wallet (fireproof and waterproof)
- ✅ Hardware wallet (Ledger, Trezor)
- ✅ Encrypted USB drive (offsite storage)
- ✅ Secret sharing (Shamir's Secret Sharing)

### 3. Regular Audits

```rust
fn audit_wallets(wallets: &[Wallet]) {
    for (i, wallet) in wallets.iter().enumerate() {
        println!("Wallet #{}", i);
        println!("  Address: {}", wallet.address);
        println!("  Public key present: {}", !wallet.public_key.is_empty());
        println!("  Private key present: {}", !wallet.private_key.is_empty());

        // Test signing
        let test_sig = wallet.sign("test");
        assert!(Wallet::verify_signature(
            &wallet.public_key,
            "test",
            &test_sig
        ));
    }
}
```

---

## Frequently Asked Questions

### Q: How do I recover a lost wallet?

**A:** You can only recover from a backed-up private key. If the private key is lost, the bitcoin is permanently lost.

### Q: Can I derive a private key from an address?

**A:** No. Addresses are one-way hashes, computationally irreversible.

### Q: Can one private key generate multiple addresses?

**A:** Hierarchical Deterministic Wallets (HD Wallet, BIP32) can derive multiple key pairs from a single seed.

### Q: How do I know if a wallet has been compromised?

**A:** Monitor transactions on the blockchain. If unauthorized transactions appear, the private key has been leaked.

---

## References

- [Transaction API](./transaction.md) - Using wallets to create transactions
- [MultiSig API](./multisig.md) - Multisig wallets
- [Quick Start](../guide/quickstart.md) - Wallet usage tutorial
- [BIP32 - HD Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP39 - Mnemonic Phrases](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)

---

[Back to API Index](./core.md)
