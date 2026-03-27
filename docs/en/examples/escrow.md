# Escrow Service in Practice

This example demonstrates how to implement a Bitcoin escrow service using 2-of-3 multisig.

## Scenario Description

In e-commerce transactions, buyers and sellers don't trust each other and need a third-party escrow:
- Buyer's concern: pays but the seller doesn't ship
- Seller's concern: ships but the buyer doesn't pay
- Solution: funds held in a 2-of-3 multisig address

**Participants**:
- Buyer
- Seller
- Arbitrator

**Rules**:
- Normal transaction: Buyer + Seller sign → funds go to Seller
- Dispute resolution: Buyer/Seller + Arbitrator → outcome per arbitration ruling

## Run the Example

```bash
cargo run --example escrow_service
```

## Code Walkthrough

### 1. Initialize Participants

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::MultiSigAddress,
};

fn main() -> Result<(), String> {
    println!("=== Bitcoin Escrow Service Demo ===\n");

    // Create the blockchain
    let mut blockchain = Blockchain::new();

    // Create participant wallets
    let buyer = Wallet::new();
    let seller = Wallet::new();
    let arbitrator = Wallet::new();

    println!("Participants created:");
    println!("  Buyer:      {}", &buyer.address[..20]);
    println!("  Seller:     {}", &seller.address[..20]);
    println!("  Arbitrator: {}\n", &arbitrator.address[..20]);
```

### 2. Create the Escrow Multisig Address

```rust
    // Create a 2-of-3 escrow multisig address
    let escrow_multisig = MultiSigAddress::new(
        2,  // requires 2 signatures
        vec![
            buyer.public_key.clone(),
            seller.public_key.clone(),
            arbitrator.public_key.clone(),
        ]
    ).expect("Failed to create multisig address");

    println!("Escrow address created:");
    println!("  Address: {}", &escrow_multisig.address[..20]);
    println!("  Type: {}-of-{} multisig",
        escrow_multisig.required_sigs,
        escrow_multisig.total_keys);
    println!("  Rule: any 2 parties can sign");
    println!("  Possible combinations:");
    println!("    - Buyer + Seller (normal transaction)");
    println!("    - Buyer + Arbitrator (refund to buyer)");
    println!("    - Seller + Arbitrator (payment to seller)\n");
```

**Key points**:
- 2-of-3 ensures no single party has unilateral control
- In normal cases, buyer and seller resolve directly
- The arbitrator intervenes only in disputes

### 3. Buyer Deposits Funds

```rust
    println!("--- Scenario 1: Buyer deposits escrow funds ---");

    // Buyer receives initial funds
    let funding_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        buyer.address.clone(),
        100000,  // 100,000 satoshi
        0,
    )?;

    blockchain.add_transaction(funding_tx)?;
    blockchain.mine_pending_transactions(buyer.address.clone())?;

    let buyer_initial = blockchain.get_balance(&buyer.address);
    println!("Buyer balance: {} sat\n", buyer_initial);

    // Buyer transfers payment to the escrow address
    let escrow_amount = 50000;  // 50,000 sat
    println!("Item price: {} sat", escrow_amount);
    println!("Buyer is transferring payment to the escrow address...\n");

    let deposit_tx = blockchain.create_transaction(
        &buyer,
        escrow_multisig.address.clone(),
        escrow_amount,
        100,  // fee
    )?;

    blockchain.add_transaction(deposit_tx)?;
    blockchain.mine_pending_transactions(buyer.address.clone())?;

    let escrow_balance = blockchain.get_balance(&escrow_multisig.address);
    println!("Funds escrowed");
    println!("  Escrow amount: {} sat", escrow_balance);
    println!("  Buyer balance: {} sat\n", blockchain.get_balance(&buyer.address));
```

**Flow**:
1. Buyer obtains initial funds
2. Buyer transfers payment to the escrow address
3. Funds are locked in the multisig address
4. Seller sees the successful escrow and ships

### 4. Scenario A: Normal Transaction Completed

```rust
    println!("--- Scenario 2A: Normal transaction (buyer satisfied) ---");
    println!("Seller has shipped");
    println!("Buyer received the item and is satisfied\n");

    // Both buyer and seller sign to release funds to the seller
    let payment_amount = escrow_balance - 50;  // deduct fee
    let payment_data = format!("{}{}{}",
        escrow_multisig.address,
        seller.address,
        payment_amount);

    println!("Signing process:");
    // Buyer signs
    let buyer_signature = buyer.sign(&payment_data);
    println!("  Buyer has signed (confirming receipt)");

    // Seller signs
    let seller_signature = seller.sign(&payment_data);
    println!("  Seller has signed (accepting payment)");

    // Verify signature count
    let signatures = vec![buyer_signature, seller_signature];

    if signatures.len() >= escrow_multisig.required_sigs {
        println!("\nSignatures satisfy requirement (2/3)");
        println!("Releasing funds to seller\n");

        // Create the payment transaction
        let payment_tx = blockchain.create_transaction(
            &Wallet::from_address(escrow_multisig.address.clone()),
            seller.address.clone(),
            payment_amount,
            50,
        )?;

        blockchain.add_transaction(payment_tx)?;
        blockchain.mine_pending_transactions(seller.address.clone())?;

        println!("=== Transaction Complete ===");
        println!("Seller balance: {} sat", blockchain.get_balance(&seller.address));
        println!("Escrow balance: {} sat", blockchain.get_balance(&escrow_multisig.address));
    }
```

**Normal flow**:
1. Seller ships
2. Buyer confirms receipt
3. Buyer signs (confirming satisfaction)
4. Seller signs (accepting payment)
5. 2 signatures satisfy the requirement
6. Funds released to seller

### 5. Scenario B: Dispute Resolution

```rust
    println!("\n--- Scenario 2B: Dispute resolution (item has a problem) ---");

    // Re-create the scenario (hypothetical)
    let escrow_multisig_dispute = MultiSigAddress::new(
        2,
        vec![
            buyer.public_key.clone(),
            seller.public_key.clone(),
            arbitrator.public_key,
        ]
    )?;

    println!("Buyer: item doesn't match the description, requesting a refund");
    println!("Seller: item is fine, refusing refund");
    println!("Arbitrator intervenes to investigate...\n");

    println!("Arbitration ruling:");
    println!("  Investigation confirms the item has a problem");
    println!("  Decision: refund the buyer\n");

    // Buyer + Arbitrator sign
    let refund_data = format!("{}{}{}",
        escrow_multisig_dispute.address,
        buyer.address,
        payment_amount);

    println!("Signing process:");
    let buyer_sig_dispute = buyer.sign(&refund_data);
    println!("  Buyer has signed (agreeing to refund)");

    let arbitrator_sig = arbitrator.sign(&refund_data);
    println!("  Arbitrator has signed (executing ruling)");

    let dispute_sigs = vec![buyer_sig_dispute, arbitrator_sig];

    if dispute_sigs.len() >= escrow_multisig_dispute.required_sigs {
        println!("\nSignatures satisfy requirement (2/3)");
        println!("Executing refund\n");

        println!("=== Dispute Resolved ===");
        println!("Refund to buyer: {} sat", payment_amount);
        println!("Arbitration fee: 50 sat (deducted from escrow)");
    }

    Ok(())
}
```

**Dispute flow**:
1. Buyer reports a problem with the item
2. Seller refuses a refund
3. Arbitrator investigates
4. Arbitrator issues a ruling
5. Buyer + Arbitrator sign
6. Funds returned to buyer

---

## Sample Output

```
=== Bitcoin Escrow Service Demo ===

Participants created:
  Buyer:      a3f2d8c9e4b7f1a8...
  Seller:     b9e4c7d2a3f1e8b6...
  Arbitrator: c8f1e9d3b4a7c2e5...

Escrow address created:
  Address: 3Mf2d8c9e4b7f1a8...
  Type: 2-of-3 multisig
  Rule: any 2 parties can sign
  Possible combinations:
    - Buyer + Seller (normal transaction)
    - Buyer + Arbitrator (refund to buyer)
    - Seller + Arbitrator (payment to seller)

--- Scenario 1: Buyer deposits escrow funds ---
Buyer balance: 100000 sat

Item price: 50000 sat
Buyer is transferring payment to the escrow address...

Funds escrowed
  Escrow amount: 50000 sat
  Buyer balance: 49900 sat

--- Scenario 2A: Normal transaction (buyer satisfied) ---
Seller has shipped
Buyer received the item and is satisfied

Signing process:
  Buyer has signed (confirming receipt)
  Seller has signed (accepting payment)

Signatures satisfy requirement (2/3)
Releasing funds to seller

=== Transaction Complete ===
Seller balance: 49950 sat
Escrow balance: 0 sat

--- Scenario 2B: Dispute resolution (item has a problem) ---
Buyer: item doesn't match the description, requesting a refund
Seller: item is fine, refusing refund
Arbitrator intervenes to investigate...

Arbitration ruling:
  Investigation confirms the item has a problem
  Decision: refund the buyer

Signing process:
  Buyer has signed (agreeing to refund)
  Arbitrator has signed (executing ruling)

Signatures satisfy requirement (2/3)
Executing refund

=== Dispute Resolved ===
Refund to buyer: 49950 sat
Arbitration fee: 50 sat (deducted from escrow)
```

---

## Business Value

### 1. Buyer Protection

| Traditional Transaction | Escrow Service |
|------------------------|---------------|
| Pay and seller doesn't ship | Funds escrowed; released only after shipping |
| Item doesn't match; no refund | Arbitrator can rule for a refund |
| No recourse for disputes | Arbitration mechanism protects rights |

### 2. Seller Protection

| Traditional Transaction | Escrow Service |
|------------------------|---------------|
| Ships but buyer refuses to pay | Payment escrowed; ship normally |
| Malicious refund requests | Arbitrator makes a fair judgment |
| No guaranteed payment | Payment certainty |

### 3. Fairness

```
Buyer alone cannot withdraw funds (needs seller or arbitrator)
Seller alone cannot withdraw funds (needs buyer or arbitrator)
Arbitrator alone cannot withdraw funds (needs buyer or seller)

→ Three-party checks and balances — fair and impartial
```

---

## Extended Solutions

### 1. Automated Arbitration

```rust
struct AutoArbitration {
    logistics_tracking: bool,
    photo_evidence: Vec<String>,
    chat_records: Vec<Message>,
}

fn auto_judge(evidence: &AutoArbitration) -> Decision {
    if evidence.logistics_tracking && evidence.photo_evidence.len() > 3 {
        Decision::RefundBuyer  // automatic refund
    } else {
        Decision::ManualReview  // human review
    }
}
```

### 2. Staged Release

```rust
// Stage 1: shipping confirmed — release 50%
// Stage 2: receipt confirmed — release remaining 50%

let stage1 = escrow_amount / 2;
let stage2 = escrow_amount - stage1;

// Seller provides tracking number → release stage1
// Buyer confirms receipt → release stage2
```

### 3. Timelock Protection

```rust
use bitcoin_simulation::advanced_tx::TimeLock;

// If no dispute within 7 days, automatically release to seller
let seven_days = 7 * 24 * 3600;
let auto_release = TimeLock::new_time_based(current_time + seven_days);

if auto_release.is_mature(...) && no_dispute {
    release_to_seller();
}
```

### 4. Multi-Tier Arbitration

```rust
// Level 1 arbitration: standard arbitrator
// Level 2 arbitration: senior arbitrator
// Level 3 arbitration: arbitration committee (3-of-5)

let appeals_committee = MultiSigAddress::new(
    3,
    vec![arbitrator1, arbitrator2, arbitrator3, arbitrator4, arbitrator5]
)?;
```

---

## Arbitrator Mechanism

### Selection Criteria

```
Good reputation (track record)
Subject matter expertise (product category)
Neutral and impartial (no conflicts of interest)
Responsive (within 24 hours)
```

### Arbitration Fees

```rust
let arbitration_fee = match dispute_complexity {
    Simple => 50,      // 0.1%
    Medium => 100,     // 0.2%
    Complex => 500,    // 1%
};

// Deducted from the escrow amount
let net_amount = escrow_amount - arbitration_fee;
```

### Arbitration Process

```
1. Buyer or seller initiates dispute
2. Submit evidence (photos, chat records)
3. Arbitrator reviews (within 3 business days)
4. Issue a ruling
5. Execute ruling (sign)
6. Collect arbitration fee
```

---

## Security Considerations

### 1. Arbitrator Collusion

**Risk**: arbitrator colludes with buyer or seller

**Protection**:
```rust
// Arbitrator must post collateral
let arbitrator_deposit = 100000;

// Collusion detected → slash collateral
if collusion_detected {
    slash_deposit(&arbitrator);
    ban_arbitrator(&arbitrator);
}

// Multiple arbitrators vote
let arbitrators = vec![arb1, arb2, arb3];
let decision = majority_vote(&arbitrators);
```

### 2. Evidence Fabrication

**Protection**:
```rust
// Logistics information on-chain
blockchain.add_tracking_info(tracking_number);

// Photo hash on-chain (tamper-proof)
let photo_hash = hash_photo(photo);
blockchain.add_evidence_hash(photo_hash);

// Timestamp proof
let timestamp = blockchain.get_block_time();
```

### 3. Malicious Delays

**Protection**:
```rust
// Set arbitration deadline
let deadline = current_time + 7 * 86400;  // 7 days

if current_time > deadline && no_decision {
    // Timeout → automatic refund
    refund_to_buyer();
}
```

---

## Implementation Recommendations

### 1. Tech Stack

```
Frontend: web interface to display escrow flow
Backend: SimpleBTC + database
Storage: evidence storage (IPFS)
Notifications: email/SMS alerts
```

### 2. User Flow

```
Buyer:
  1. Browse items
  2. Place order and transfer payment to escrow
  3. Wait for seller to ship
  4. Receive item and confirm
  5. Sign to release funds

Seller:
  1. Wait for buyer to deposit escrow funds
  2. Ship upon seeing successful escrow
  3. Provide tracking number
  4. Wait for buyer to confirm
  5. Sign to receive payment
```

### 3. Fee Structure

```
Platform fee: 1%
Arbitration fee: 0.1–1% (in disputes)
Blockchain transaction fee: dynamic (50–200 sat)
```

---

## Comparison with Traditional Solutions

### vs. Alipay Escrow

| Feature | SimpleBTC Escrow | Alipay Escrow |
|---------|-----------------|--------------|
| Decentralized | Yes | No (centralized) |
| Censorship resistance | Yes | No (can be censored) |
| Cross-border payments | Yes | Restricted |
| Fee | Low (0.1–1%) | Higher (1–3%) |
| Privacy | Better | Worse |

### vs. PayPal Disputes

| Feature | SimpleBTC Escrow | PayPal |
|---------|-----------------|--------|
| Dispute resolution | Arbitrator | Platform support |
| Transparency | On-chain, publicly verifiable | Black-box process |
| Irreversibility | Yes | No (can freeze accounts) |

---

## Related Resources

- [Multisig Explained](../advanced/multisig.md)
- [MultiSig API](../api/multisig.md)
- [Enterprise Wallet Example](./enterprise-multisig.md)

## Summary

The escrow service achieves the following through 2-of-3 multisig:

**Buyer protection** — refunds available if item doesn't match
**Seller protection** — guaranteed receipt of payment
**Fair arbitration** — impartial third-party ruling
**Decentralized** — no need to trust a central platform
**Transparent** — on-chain records are publicly accessible

This is an ideal solution for e-commerce, freelancing, and cross-border trade.

---

[View full source code](../../examples/escrow_service.rs)
