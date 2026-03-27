# Wallet Management

At its core, a Bitcoin wallet is a key pair: a private key and a public key. SimpleBTC uses secp256k1 elliptic curve cryptography that is fully compatible with real Bitcoin, implementing two structs: `Wallet` (the primary wallet) and `CryptoWallet` (extended wallet, supporting Bech32 and WIF).

---

## Key System Overview

Bitcoin key generation follows a strict one-way derivation chain:

```
Random number (256 bit)
       │
       ▼  secp256k1 elliptic curve multiplication
    Private key (SecretKey, 32 bytes)
       │
       ▼  scalar multiplication by generator point G
    Public key (PublicKey, 33 bytes compressed format)
       │
       ├─▶ SHA-256 hash
       │          │
       │          ▼  RIPEMD-160 hash
       │      Public key hash (20 bytes)
       │          │
       │          ▼  version prefix 0x00 + double SHA-256 checksum + Base58
       │      P2PKH address (starts with '1')
       │
       └─▶ SHA-256 + RIPEMD-160
                  │
                  ▼  Bech32 encoding (witness v0)
              Bech32 address (starts with 'bc1')
```

Elliptic curve equation (secp256k1):

```
y² = x³ + 7  (mod p)
p = 2²⁵⁶ − 2³² − 977  (a very large prime)
```

Deriving a public key from a private key is a one-way operation — computationally irreversible (discrete logarithm problem).

---

## The Wallet Struct

`Wallet` is the most commonly used wallet type in the project, defined in `src/wallet.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Bitcoin P2PKH address (starts with '1')
    pub address: String,
    /// Compressed public key in hexadecimal (33 bytes = 66 hex characters)
    pub public_key: String,
    /// secp256k1 private key (stored as hexadecimal during serialization, access restricted)
    #[serde(with = "secret_key_serde")]
    private_key: SecretKey,
}
```

**Field descriptions:**
- `address`: P2PKH format address, publicly used — safe to share with others as a receiving address
- `public_key`: Compressed public key (33 bytes), used to verify signatures; included in each transaction input
- `private_key`: Private key — must be strictly kept secret; possessing the private key is equivalent to owning all funds at the corresponding address

---

## Creating a Wallet

### Generating a Random Wallet

```rust
use bitcoin_simulation::wallet::Wallet;

let wallet = Wallet::new();

println!("Address:     {}", wallet.address);           // P2PKH address starting with '1'
println!("Public key:  {}", wallet.public_key);        // 66-character hex
println!("Private key: {}", wallet.private_key_hex()); // 64-character hex (keep secret!)
```

Internal flow of `Wallet::new()`:

```rust
pub fn new() -> Self {
    let secp = Secp256k1::new();
    // Uses a cryptographically secure random number generator (OsRng)
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
    let address = Self::pubkey_to_address(&public_key);
    let public_key_hex = hex::encode(public_key.serialize());

    Wallet { address, public_key: public_key_hex, private_key: secret_key }
}
```

### Genesis Wallet

The genesis wallet uses a fixed private key `0x01`, generating the same address on every startup. This makes it convenient for spending the initial funds from the genesis block during demos:

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

// Two equivalent ways to obtain it
let genesis = Blockchain::genesis_wallet();
let genesis2 = Wallet::genesis();

assert_eq!(genesis.address, genesis2.address); // Address is deterministically consistent

// Internal implementation (src/wallet.rs)
pub fn genesis() -> Self {
    Self::from_private_key_hex(
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .expect("genesis private key is valid")
}
```

> **Warning:** The genesis wallet's private key is public — never use it in a production environment.

### Recovering a Wallet from a Private Key

```rust
// Recover from a hexadecimal private key
let wallet = Wallet::new();
let hex = wallet.private_key_hex();  // Export private key

let recovered = Wallet::from_private_key_hex(&hex)?;
assert_eq!(wallet.address, recovered.address);  // Address is identical
```

---

## P2PKH Address Derivation

`Wallet::pubkey_to_address()` implements address generation steps that are identical to real Bitcoin:

```rust
fn pubkey_to_address(public_key: &PublicKey) -> String {
    // Step 1: Serialize compressed public key (33 bytes: 1-byte prefix + 32-byte x coordinate)
    let pubkey_bytes = public_key.serialize();

    // Step 2: SHA-256 hash
    let sha256_hash = sha256::Hash::hash(&pubkey_bytes);

    // Step 3: RIPEMD-160 hash → public key hash (20 bytes)
    let mut ripemd = Ripemd160::new();
    ripemd.update(&sha256_hash[..]);
    let pubkey_hash = ripemd.finalize();

    // Step 4: Add version byte (mainnet = 0x00)
    let mut versioned = vec![0x00];
    versioned.extend_from_slice(&pubkey_hash);  // 21 bytes total

    // Step 5: Double SHA-256, take first 4 bytes as checksum
    let checksum = sha256d::Hash::hash(&versioned);
    versioned.extend_from_slice(&checksum[0..4]);  // 25 bytes total

    // Step 6: Base58 encode → address starting with '1' (~34 characters)
    bs58::encode(versioned).into_string()
}
```

**Why RIPEMD-160?**
- Compresses the 33-byte public key to 20 bytes, saving blockchain storage space
- Even if a quantum computer breaks ECDSA, the attacker would still need to break the hash function

**Why Base58 (not Base64)?**
- Removes easily confused characters: 0 (zero), O (uppercase O), I (uppercase i), l (lowercase L)
- Avoids issues such as spaces being included when double-clicking to copy

---

## Transaction Signing

`Wallet::sign()` uses the private key to generate an ECDSA signature over data:

```rust
pub fn sign(&self, data: &str) -> String {
    let secp = Secp256k1::new();
    // 1. SHA-256 hash the raw data
    let msg_hash = sha256::Hash::hash(data.as_bytes());
    let message = Message::from_digest(msg_hash.to_byte_array());
    // 2. Generate ECDSA signature using the private key
    let signature = secp.sign_ecdsa(&message, &self.private_key);
    // 3. DER-encode and return as a hex string
    hex::encode(signature.serialize_der())
}
```

In `Blockchain::create_transaction()`, the signed data is `"{txid}{vout}"` — the location identifier of the UTXO being referenced:

```rust
// Excerpt from src/blockchain.rs
for (txid, vout) in utxos {
    let signature = from_wallet.sign(&format!("{}{}", txid, vout));
    let input = TxInput::new(txid, vout, signature, from_wallet.public_key.clone());
    inputs.push(input);
}
```

This binds each input's signature to a specific UTXO, preventing the signature from being replayed against other UTXOs.

---

## Signature Verification

`Wallet::verify_signature()` is a static method — no private key is needed:

```rust
pub fn verify_signature(public_key_hex: &str, data: &str, signature_hex: &str) -> bool {
    // 1. Decode the public key
    let Ok(pubkey_bytes) = hex::decode(public_key_hex) else { return false; };
    let Ok(public_key) = PublicKey::from_slice(&pubkey_bytes) else { return false; };

    // 2. Decode the DER signature
    let Ok(sig_bytes) = hex::decode(signature_hex) else { return false; };
    let Ok(signature) = Signature::from_der(&sig_bytes) else { return false; };

    // 3. Re-hash the raw data (exactly as during signing)
    let secp = Secp256k1::new();
    let msg_hash = sha256::Hash::hash(data.as_bytes());
    let message = Message::from_digest(msg_hash.to_byte_array());

    // 4. Mathematical verification: check whether the signature was generated by the corresponding private key
    secp.verify_ecdsa(&message, &signature, &public_key).is_ok()
}
```

Complete sign + verify example:

```rust
use bitcoin_simulation::wallet::Wallet;

let wallet = Wallet::new();
let data = "Hello, Bitcoin!";

// Sign
let signature = wallet.sign(data);
println!("Signature: {}", &signature[..32]); // DER-encoded hex

// Verify correct data: should pass
assert!(Wallet::verify_signature(&wallet.public_key, data, &signature));

// Verify tampered data: should fail
assert!(!Wallet::verify_signature(&wallet.public_key, "Tampered!", &signature));

// Verify with wrong public key: should fail
let other = Wallet::new();
assert!(!Wallet::verify_signature(&other.public_key, data, &signature));
```

---

## Extended Wallet: CryptoWallet

`CryptoWallet` in `src/crypto.rs` adds more Bitcoin protocol features on top of `Wallet`:

```rust
use bitcoin_simulation::crypto::CryptoWallet;

let wallet = CryptoWallet::new();

println!("P2PKH address:   {}", wallet.address);           // starts with '1'
println!("Bech32 address:  {}", wallet.bech32_address);    // starts with 'bc1' (SegWit)
println!("Private key hex: {}", wallet.private_key_hex()); // 64 characters
println!("Public key hex:  {}", wallet.public_key_hex());  // 66 characters
```

### WIF Private Key Format

WIF (Wallet Import Format) is the standard format for importing and exporting private keys between Bitcoin wallets:

```rust
let wallet = CryptoWallet::new();

// Export as WIF (starts with '5', 'K', or 'L')
let wif = wallet.export_private_key_wif();
println!("WIF: {}", wif);

// Recover wallet from WIF
let imported = CryptoWallet::import_from_wif(&wif)?;
assert_eq!(wallet.address, imported.address);
```

WIF encoding steps:
1. Add version byte `0x80` (mainnet private key prefix)
2. Compute double SHA-256 checksum (take first 4 bytes)
3. Concatenate and Base58-encode

### CryptoWallet Signing Interface

`CryptoWallet`'s signing interface accepts a byte slice, which is more flexible:

```rust
let wallet = CryptoWallet::new();
let message = b"Hello, Bitcoin!";

// Sign (returns a secp256k1::ecdsa::Signature type)
let signature = wallet.sign(message);

// Verify (static method)
assert!(CryptoWallet::verify(message, &signature, &wallet.public_key));
```

---

## Wallet Serialization

Both `Wallet` and `CryptoWallet` implement `Serialize` / `Deserialize`, with the private key securely stored as a hexadecimal string:

```rust
use bitcoin_simulation::wallet::Wallet;

let wallet = Wallet::new();

// Serialize to JSON
let json = serde_json::to_string(&wallet)?;
// {"address":"1...","public_key":"02...","private_key":"a1b2c3..."}

// Restore from JSON with full signing capability preserved
let restored: Wallet = serde_json::from_str(&json)?;
let sig = restored.sign("test");
assert!(Wallet::verify_signature(&restored.public_key, "test", &sig));
```

---

## Security Recommendations

| Item | Description |
|------|-------------|
| Keep private key secret | Anyone who obtains the private key can spend all funds at the corresponding address |
| Do not reuse addresses | Use a new address for each payment to protect privacy |
| Genesis wallet for demo only | `Wallet::genesis()` uses a public private key — never use it for real funds |
| Back up your private key | Losing the private key means permanently losing the corresponding funds |
| Use WIF format for backups | WIF format includes a checksum, which can detect transcription errors |
