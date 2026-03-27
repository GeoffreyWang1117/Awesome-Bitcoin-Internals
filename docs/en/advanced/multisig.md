# Multi-Signature (MultiSig)

Multi-signature is an advanced Bitcoin feature that requires M signatures to spend funds controlled by N public keys (M-of-N).

## Overview

### What is Multi-Signature?

A multi-signature address requires multiple private keys to jointly sign before funds can be spent, rather than a single private key as in traditional setups.

**Examples**:
- **2-of-3**: Requires any 2 of 3 keys
- **3-of-5**: Requires any 3 of 5 keys
- **2-of-2**: Requires both keys to agree

### Why Use MultiSig?

#### 1. Improved Security
- No single point of failure
- A stolen private key does not immediately result in fund loss
- Risk is distributed

#### 2. Distributed Trust
- Corporate governance: prevents misuse by a single person
- Escrow services: buyer + seller + arbitrator
- Joint family management: shared management by spouses

#### 3. Flexibility
- Different M-N combinations meet different needs
- Emergency recovery mechanisms can be set up
- Supports complex business logic

## Technical Implementation

### MultiSig Address Structure

```rust
pub struct MultiSigAddress {
    pub address: String,            // Multi-sig address (starts with "3")
    pub required_sigs: usize,       // M (number of required signatures)
    pub total_keys: usize,          // N (total number of keys)
    pub public_keys: Vec<String>,   // Public keys of all participants
    pub script: String,             // Locking script
}
```

### Creating a Multi-Sig Address

```rust
use bitcoin_simulation::{multisig::MultiSigAddress, wallet::Wallet};

// Create participants
let ceo = Wallet::new();
let cfo = Wallet::new();
let cto = Wallet::new();

// Collect public keys
let public_keys = vec![
    ceo.public_key.clone(),
    cfo.public_key.clone(),
    cto.public_key.clone(),
];

// Create a 2-of-3 multi-sig address
let multisig = MultiSigAddress::new(2, public_keys)?;

println!("Multi-sig address: {}", multisig.address);
println!("Required signatures: {}/{}", multisig.required_sigs, multisig.total_keys);
```

### MultiSig Types

SimpleBTC provides preset common multi-sig types:

```rust
use bitcoin_simulation::multisig::MultiSigType;

// 2-of-2: both parties must agree
let two_of_two = MultiSigAddress::from_type(
    MultiSigType::TwoOfTwo,
    vec![alice.public_key, bob.public_key]
)?;

// 2-of-3: any two parties suffice (most common)
let two_of_three = MultiSigAddress::from_type(
    MultiSigType::TwoOfThree,
    vec![party1.public_key, party2.public_key, party3.public_key]
)?;

// 3-of-5: high-security scenarios
let three_of_five = MultiSigAddress::from_type(
    MultiSigType::ThreeOfFive,
    vec![pk1, pk2, pk3, pk4, pk5]
)?;
```

## Use Cases

### Use Case 1: Corporate Financial Management

**Requirement**: Company funds require joint approval from multiple executives

**Solution**: 2-of-3 MultiSig (CEO + CFO + CTO)

```rust
fn setup_corporate_wallet() -> Result<MultiSigAddress, String> {
    // 1. Create executive wallets
    let ceo = Wallet::new();
    let cfo = Wallet::new();
    let cto = Wallet::new();

    println!("=== Corporate Multi-Sig Wallet ===");
    println!("CEO: {}", &ceo.address[..16]);
    println!("CFO: {}", &cfo.address[..16]);
    println!("CTO: {}", &cto.address[..16]);

    // 2. Create multi-sig address
    let company_wallet = MultiSigAddress::new(
        2,  // Requires 2 signatures
        vec![
            ceo.public_key.clone(),
            cfo.public_key.clone(),
            cto.public_key.clone(),
        ]
    )?;

    println!("\nCompany multi-sig address: {}", company_wallet.address);
    println!("Rule: any 2 executives can authorize a transfer\n");

    Ok(company_wallet)
}

// Transfer scenario
fn corporate_payment(
    multisig: &MultiSigAddress,
    ceo: &Wallet,
    cfo: &Wallet,
    recipient: &str,
    amount: u64
) -> Result<(), String> {
    println!("Transferring {} satoshi to {}", amount, &recipient[..16]);

    // 1. CEO signs
    let ceo_sig = ceo.sign(&format!("{}{}", multisig.address, amount));
    println!("✓ CEO has signed");

    // 2. CFO signs
    let cfo_sig = cfo.sign(&format!("{}{}", multisig.address, amount));
    println!("✓ CFO has signed");

    // 3. Collect signatures
    let signatures = vec![ceo_sig, cfo_sig];

    // 4. Verify signature count
    if signatures.len() >= multisig.required_sigs {
        println!("✅ Signature count meets requirement; transaction can be executed");
        // Create and broadcast transaction...
        Ok(())
    } else {
        Err("Insufficient signatures".to_string())
    }
}
```

**Advantages**:
- ✅ Prevents misuse of funds by a single person
- ✅ CFO + CTO can still operate when CEO is traveling
- ✅ Even if one person is compromised, funds remain safe

### Use Case 2: Escrow Service

**Requirement**: Buyer and seller do not trust each other; a third-party arbitrator is needed

**Solution**: 2-of-3 MultiSig (buyer + seller + arbitrator)

```rust
fn escrow_service() -> Result<(), String> {
    // Participants
    let buyer = Wallet::new();
    let seller = Wallet::new();
    let arbitrator = Wallet::new();

    println!("=== Escrow Service ===");
    println!("Buyer: {}", &buyer.address[..16]);
    println!("Seller: {}", &seller.address[..16]);
    println!("Arbitrator: {}", &arbitrator.address[..16]);

    // Create escrow multi-sig address
    let escrow = MultiSigAddress::new(
        2,
        vec![
            buyer.public_key.clone(),
            seller.public_key.clone(),
            arbitrator.public_key.clone(),
        ]
    )?;

    println!("\nEscrow address: {}", escrow.address);

    // Scenario 1: Normal transaction (buyer + seller)
    println!("\n--- Scenario 1: Transaction completed smoothly ---");
    println!("Buyer received goods; satisfied");
    println!("Buyer signs: ✓");
    println!("Seller signs: ✓");
    println!("✅ 2/3 signatures; funds released to seller");

    // Scenario 2: Dispute (buyer + arbitrator or seller + arbitrator)
    println!("\n--- Scenario 2: Dispute arises ---");
    println!("Buyer: goods are defective");
    println!("Seller: goods are fine");
    println!("Arbitrator investigates...");
    println!("Arbitrator: buyer is right");
    println!("Buyer signs: ✓");
    println!("Arbitrator signs: ✓");
    println!("✅ 2/3 signatures; funds refunded to buyer");

    Ok(())
}
```

**Advantages**:
- ✅ Buyer protection: refund if goods don't match description
- ✅ Seller protection: funds released automatically for normal transactions
- ✅ Fair: arbitrator cannot control funds alone

### Use Case 3: Personal Asset Protection

**Requirement**: Prevent loss due to a single private key being lost or stolen

**Solution**: 2-of-3 MultiSig (primary key + backup key + custodian key)

```rust
fn personal_security_setup() -> Result<(), String> {
    // Key assignment
    let main_key = Wallet::new();      // Daily use
    let backup_key = Wallet::new();    // Safe deposit box
    let custodian_key = Wallet::new(); // Lawyer / trust company

    println!("=== Personal Asset Protection ===");
    println!("Primary key (daily): {}", &main_key.address[..16]);
    println!("Backup key (safe): {}", &backup_key.address[..16]);
    println!("Custodian key (lawyer): {}", &custodian_key.address[..16]);

    let secure_wallet = MultiSigAddress::new(
        2,
        vec![
            main_key.public_key,
            backup_key.public_key,
            custodian_key.public_key,
        ]
    )?;

    println!("\nSecure wallet: {}", secure_wallet.address);

    // Usage scenarios
    println!("\n--- Usage Scenarios ---");
    println!("Daily transfers: primary key + backup key");
    println!("Primary key lost: backup key + custodian key");
    println!("Theft risk: requires 2 keys; single key theft poses no risk");

    Ok(())
}
```

### Use Case 4: Cold-Hot Wallet Combination

**Requirement**: Security for large storage + convenience for small amounts

**Solution**: 2-of-3 (hot wallet + cold wallet 1 + cold wallet 2)

```rust
fn cold_hot_wallet_setup() -> Result<(), String> {
    let hot_wallet = Wallet::new();    // Online device
    let cold_wallet_1 = Wallet::new(); // Hardware wallet 1
    let cold_wallet_2 = Wallet::new(); // Paper wallet

    println!("=== Cold-Hot Wallet Combination ===");
    println!("Hot wallet (phone): {}", &hot_wallet.address[..16]);
    println!("Cold wallet 1 (Ledger): {}", &cold_wallet_1.address[..16]);
    println!("Cold wallet 2 (paper wallet): {}", &cold_wallet_2.address[..16]);

    let vault = MultiSigAddress::new(
        2,
        vec![
            hot_wallet.public_key,
            cold_wallet_1.public_key,
            cold_wallet_2.public_key,
        ]
    )?;

    println!("\nVault address: {}", vault.address);

    println!("\n--- Usage Strategy ---");
    println!("Daily small amounts: hot wallet + cold wallet 1 (convenient)");
    println!("Large transfers: cold wallet 1 + cold wallet 2 (most secure)");
    println!("Hot wallet hacked: still requires cold wallet cooperation; funds safe");

    Ok(())
}
```

## Advanced Usage

### TimeLock + MultiSig

Combining time locks for inheritance planning:

```rust
use bitcoin_simulation::advanced_tx::TimeLock;

fn inheritance_setup() -> Result<(), String> {
    let owner = Wallet::new();
    let heir = Wallet::new();
    let lawyer = Wallet::new();

    // Normal: 2-of-2 (owner + heir; protects privacy)
    let normal_multisig = MultiSigAddress::new(
        2,
        vec![owner.public_key.clone(), heir.public_key.clone()]
    )?;

    // Time lock: 1 year later
    let one_year = 365 * 24 * 60 * 60;
    let unlock_time = current_timestamp() + one_year;
    let timelock = TimeLock::new_time_based(unlock_time);

    println!("=== Inheritance Plan ===");
    println!("Normal period: requires owner + heir (2-of-2)");
    println!("After 1 year: heir can operate independently");

    // Or use 3-of-3, downgraded to 2-of-3 after 1 year
    let emergency_multisig = MultiSigAddress::new(
        2,  // Only 2 required after 1 year
        vec![owner.public_key, heir.public_key, lawyer.public_key]
    )?;

    Ok(())
}
```

### Hierarchical MultiSig

Multi-level multi-sig structure for large organizations:

```rust
// Board of directors: 5-of-9
let board = MultiSigAddress::new(5, board_members)?;

// Executive committee: 3-of-5
let exec_committee = MultiSigAddress::new(3, executives)?;

// Petty cash: 2-of-3
let petty_cash = MultiSigAddress::new(2, managers)?;

println!("Permission levels:");
println!("< 10 BTC: manager level 2-of-3");
println!("10-100 BTC: executive level 3-of-5");
println!("> 100 BTC: board level 5-of-9");
```

## Security Considerations

### ⚠️ Important Notes

1. **Key Management**
   - Store keys in dispersed locations; do not keep them together
   - Use hardware wallets for cold keys
   - Regularly test backup recovery

2. **Choosing M**
   - M too small: security is reduced
   - M too large: usability is reduced
   - Recommended: M = (N+1)/2 or N-1

3. **Choosing N**
   - N=2: simple but has a single point of failure
   - N=3: balances security and convenience (most common)
   - N=5+: high security but complex

4. **Choosing Participants**
   - Geographically dispersed
   - Trusted but mutually independent
   - Have emergency contact information

### Best Practices

```rust
// ✅ Good practice
let multisig = MultiSigAddress::new(
    2,  // Reasonable M value
    vec![key1, key2, key3]  // 3 independent keys
)?;

// Dispersed storage
// key1 -> mobile hot wallet
// key2 -> hardware wallet (safe)
// key3 -> paper wallet (bank safe deposit box)

// ❌ Bad practice
// All keys stored on the same computer
// M=N (loses fault tolerance)
// Multiple keys derived from the same mnemonic
```

## Complete Example

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::MultiSigAddress,
};

fn complete_multisig_demo() -> Result<(), String> {
    let mut blockchain = Blockchain::new();

    // Create participants
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    // Create 2-of-3 multi-sig
    let multisig = MultiSigAddress::new(
        2,
        vec![
            alice.public_key.clone(),
            bob.public_key.clone(),
            charlie.public_key.clone(),
        ]
    )?;

    println!("Multi-sig address: {}", multisig.address);

    // 1. Deposit funds
    let funding_tx = blockchain.create_transaction(
        &Wallet::from_address("funder".to_string()),
        multisig.address.clone(),
        10000,
        0,
    )?;
    blockchain.add_transaction(funding_tx)?;
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("Multi-sig balance: {}", blockchain.get_balance(&multisig.address));

    // 2. Multi-sig transfer (requires 2 signatures)
    let recipient = Wallet::new();

    // Alice signs
    let alice_sig = alice.sign(&format!("{}{}",
        multisig.address, recipient.address));

    // Bob signs
    let bob_sig = bob.sign(&format!("{}{}",
        multisig.address, recipient.address));

    // Verify signatures
    println!("\nCollecting signatures:");
    println!("Alice: ✓");
    println!("Bob: ✓");

    if vec![alice_sig, bob_sig].len() >= multisig.required_sigs {
        println!("✅ Signatures meet requirement; transfer can proceed");

        // Create transfer transaction
        // Note: actual implementation requires multi-sig transaction building logic
        println!("Transaction created and broadcast");
    }

    Ok(())
}
```

## References

- [BIP11 - M-of-N Standard Transactions](https://github.com/bitcoin/bips/blob/master/bip-0011.mediawiki)
- [BIP16 - P2SH](https://github.com/bitcoin/bips/blob/master/bip-0016.mediawiki)
- [Corporate MultiSig Example](../examples/enterprise-multisig.md)
- [Escrow Service Example](../examples/escrow.md)

---

**Next**: [TimeLock Tutorial](./timelock.md) | [RBF Mechanism](./rbf.md)

[Back to Advanced Features](./README.md)
