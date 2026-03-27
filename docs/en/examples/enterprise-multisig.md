# Enterprise Multisig Wallet in Practice

This example demonstrates how to use 2-of-3 multisig to manage corporate funds.

## Scenario Description

A technology company needs to manage its bitcoin assets with the following requirements:
- Three executives (CEO, CFO, CTO) each hold one key
- Any two executives can authorize a transfer
- Prevent misuse or loss of control by a single person
- Normal operations continue even if one executive is unavailable

## Run the Example

```bash
cargo run --example enterprise_multisig
```

## Code Walkthrough

### 1. Initialization

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::MultiSigAddress,
};

fn main() -> Result<(), String> {
    println!("=== Enterprise Multisig Wallet Demo ===\n");

    // Create the blockchain
    let mut blockchain = Blockchain::new();

    // Create wallets for the three executives
    let ceo = Wallet::new();
    let cfo = Wallet::new();
    let cto = Wallet::new();

    println!("Created executive wallets:");
    println!("  CEO: {}", &ceo.address[..20]);
    println!("  CFO: {}", &cfo.address[..20]);
    println!("  CTO: {}\n", &cto.address[..20]);
```

### 2. Create the Multisig Address

```rust
    // Create a 2-of-3 multisig address
    let company_multisig = MultiSigAddress::new(
        2,  // requires 2 signatures
        vec![
            ceo.public_key.clone(),
            cfo.public_key.clone(),
            cto.public_key.clone(),
        ]
    ).expect("Failed to create multisig address");

    println!("Company multisig address created:");
    println!("  Address: {}", &company_multisig.address[..20]);
    println!("  Type: {}-of-{} multisig",
        company_multisig.required_sigs,
        company_multisig.total_keys);
    println!("  Rule: any 2 executives can authorize a transfer\n");
```

**Key points**:
- `required_sigs = 2`: requires 2 signatures
- `total_keys = 3`: 3 total keys
- Any combination of two executives works: CEO+CFO, CEO+CTO, or CFO+CTO

### 3. Fund the Address

```rust
    // Fund the company multisig address
    println!("--- Scenario 1: Company receives investment ---");

    let investor = Wallet::new();
    println!("Investor address: {}\n", &investor.address[..20]);

    // Create a funding transaction (from the genesis address)
    let funding_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        company_multisig.address.clone(),
        100000,  // 100,000 satoshi
        0,
    )?;

    blockchain.add_transaction(funding_tx)?;
    blockchain.mine_pending_transactions(investor.address.clone())?;

    let company_balance = blockchain.get_balance(&company_multisig.address);
    println!("Funding complete");
    println!("  Company account balance: {} satoshi\n", company_balance);
```

### 4. Scenario Demo: Normal Expenditure

```rust
    println!("--- Scenario 2: Normal payment (approved by CEO + CFO) ---");

    let supplier = Wallet::new();
    println!("Supplier address: {}\n", &supplier.address[..20]);

    // Simulate the multisig flow
    let payment_amount = 30000;
    let payment_data = format!("{}{}", company_multisig.address, supplier.address);

    // Step 1: CEO signs
    let ceo_signature = ceo.sign(&payment_data);
    println!("CEO has approved and signed");

    // Step 2: CFO signs
    let cfo_signature = cfo.sign(&payment_data);
    println!("CFO has approved and signed");

    // Step 3: Verify signature count
    let signatures = vec![ceo_signature, cfo_signature];

    if signatures.len() >= company_multisig.required_sigs {
        println!("Signature count satisfies requirement (2/3)");
        println!("Transaction can be executed\n");

        // Execute the transfer
        let payment_tx = blockchain.create_transaction(
            &Wallet::from_address(company_multisig.address.clone()),
            supplier.address.clone(),
            payment_amount,
            100,
        )?;

        blockchain.add_transaction(payment_tx)?;
        blockchain.mine_pending_transactions(ceo.address.clone())?;

        println!("Payment complete");
        println!("  Amount paid: {} satoshi", payment_amount);
        println!("  Company balance: {} satoshi\n",
            blockchain.get_balance(&company_multisig.address));
    }
```

**Workflow**:
1. CEO initiates a payment request
2. CEO signs with their private key
3. CFO reviews and signs
4. The system verifies the signature count (2 ≥ required 2)
5. The transfer is executed

### 5. Scenario Demo: CEO Unavailable

```rust
    println!("--- Scenario 3: Emergency payment while CEO is traveling (CFO + CTO) ---");

    let emergency_vendor = Wallet::new();
    println!("Emergency vendor: {}\n", &emergency_vendor.address[..20]);

    let emergency_amount = 20000;
    let emergency_data = format!("{}{}",
        company_multisig.address, emergency_vendor.address);

    println!("CEO is traveling and cannot be reached");
    println!("CFO and CTO decide to approve the emergency payment\n");

    // CFO signs
    let cfo_sig = cfo.sign(&emergency_data);
    println!("CFO has signed");

    // CTO signs
    let cto_sig = cto.sign(&emergency_data);
    println!("CTO has signed");

    let emergency_sigs = vec![cfo_sig, cto_sig];

    if emergency_sigs.len() >= company_multisig.required_sigs {
        println!("Signatures satisfy requirement (2/3)");
        println!("Business continues normally even without the CEO\n");

        // Execute the transfer
        let emergency_tx = blockchain.create_transaction(
            &Wallet::from_address(company_multisig.address.clone()),
            emergency_vendor.address,
            emergency_amount,
            100,
        )?;

        blockchain.add_transaction(emergency_tx)?;
        blockchain.mine_pending_transactions(cfo.address.clone())?;

        println!("Emergency payment complete");
        println!("  Final balance: {} satoshi\n",
            blockchain.get_balance(&company_multisig.address));
    }

    Ok(())
}
```

## Sample Output

```
=== Enterprise Multisig Wallet Demo ===

Created executive wallets:
  CEO: a3f2d8c9e4b7f1a8...
  CFO: b9e4c7d2a3f1e8b6...
  CTO: c8f1e9d3b4a7c2e5...

Company multisig address created:
  Address: 3Mf2d8c9e4b7f1a8...
  Type: 2-of-3 multisig
  Rule: any 2 executives can authorize a transfer

--- Scenario 1: Company receives investment ---
Investor address: d7c2e8f3a9b1d4c6...

Block mined: 0003ab4f9c2d...
Funding complete
  Company account balance: 100000 satoshi

--- Scenario 2: Normal payment (approved by CEO + CFO) ---
Supplier address: e6d1f8c2b9a3e7d4...

CEO has approved and signed
CFO has approved and signed
Signature count satisfies requirement (2/3)
Transaction can be executed

Block mined: 0007c3e8d1a9...
Payment complete
  Amount paid: 30000 satoshi
  Company balance: 69900 satoshi

--- Scenario 3: Emergency payment while CEO is traveling (CFO + CTO) ---
Emergency vendor: f5e2d9c3a8b7f1e6...

CEO is traveling and cannot be reached
CFO and CTO decide to approve the emergency payment

CFO has signed
CTO has signed
Signatures satisfy requirement (2/3)
Business continues normally even without the CEO

Block mined: 000ab7e4f2c8...
Emergency payment complete
  Final balance: 49800 satoshi
```

## Business Value

### 1. Security

| Traditional Single-Sig | Enterprise Multisig |
|------------------------|---------------------|
| CEO's private key stolen → all funds lost | Requires 2 keys; theft of one is harmless |
| Single point of failure | Distributed risk |
| High risk of insider fraud | Requires two people to collude |

### 2. Business Continuity

| Scenario | Traditional Approach | Multisig Approach |
|----------|---------------------|------------------|
| CEO on vacation | Business halts | CFO+CTO continue operations |
| Executive resigns | Must transfer all funds | Replace one key |
| Emergency payment | Key holder unavailable | Any 2 people can approve |

### 3. Compliance

```
Audit trail:
- Every transaction requires 2 signatures
- Clear record of who approved what
- Satisfies internal control requirements
- Meets financial audit standards
```

## Extended Solutions

### Tiered Authorization

```rust
// Small amounts: manager level 2-of-3
if amount < 10000 {
    let managers_multisig = MultiSigAddress::new(2, manager_keys)?;
}

// Medium amounts: executive level 2-of-3
else if amount < 100000 {
    let exec_multisig = MultiSigAddress::new(2, exec_keys)?;
}

// Large amounts: board 5-of-9
else {
    let board_multisig = MultiSigAddress::new(5, board_keys)?;
}
```

### Timelock Protection

```rust
use bitcoin_simulation::advanced_tx::TimeLock;

// Large transfers require a 24-hour delay
let timelock = TimeLock::new_time_based(
    current_time() + 24 * 3600
);

// Can be cancelled during the delay period
// Protects against coerced transfers
```

### Emergency Recovery

```rust
// Normal: 2-of-3
let normal_multisig = MultiSigAddress::new(
    2,
    vec![ceo_key, cfo_key, cto_key]
)?;

// Emergency (2 keys lost): attorney-custodied recovery key
let recovery_multisig = MultiSigAddress::new(
    1,
    vec![lawyer_key]  // requires legal documentation
)?;
```

## Implementation Recommendations

### 1. Key Management

```
CEO key:
  - Primary: mobile hot wallet (daily signing)
  - Backup: hardware wallet (safe)

CFO key:
  - Primary: desktop hot wallet (office)
  - Backup: paper wallet (bank safe deposit box)

CTO key:
  - Primary: hardware wallet (carried personally)
  - Backup: encrypted USB drive (offsite storage)
```

### 2. Operating Procedure

```
1. Initiator creates transfer request
2. Initiator signs
3. Notify the second approver
4. Second approver reviews and signs
5. System automatically verifies signature count
6. Execute transaction and notify everyone
7. Record audit log
```

### 3. Security Checklist

- [ ] Keys stored in separate locations
- [ ] Regularly test the recovery process
- [ ] Back up all keys
- [ ] Set amount thresholds
- [ ] Enable transaction notifications
- [ ] Regularly audit transaction records
- [ ] Prepare a contingency plan for key loss
- [ ] Train all key holders

## Related Resources

- [Multisig Explained](../advanced/multisig.md)
- [MultiSig API Documentation](../api/multisig.md)
- [Escrow Service Example](./escrow.md)

## Summary

The enterprise multisig wallet achieves the following through a 2-of-3 mechanism:

**Security** — no single point of failure
**Flexibility** — any two people can approve
**Continuity** — operations continue even if one person is absent
**Compliance** — satisfies internal controls
**Transparency** — all operations are traceable

This is a best practice for enterprises managing digital assets.

---

[View full source code](../../examples/enterprise_multisig.rs)
