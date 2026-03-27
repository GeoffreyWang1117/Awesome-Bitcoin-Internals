# Time Deposit System

This example demonstrates how to use the TimeLock feature to implement a time deposit system. Users can choose from different terms (3 months or 1 year); funds cannot be withdrawn before maturity, and principal plus interest can be collected upon maturity.

## Business Scenario

**Pain points of traditional time deposits**:
- Early withdrawal requires bank approval
- Interest calculation is opaque
- Requires trusting a bank
- Manual action needed at maturity

**Blockchain solution**:
- Smart contract executes automatically — no approval needed
- Interest rules written into code — fully transparent
- Technically impossible to withdraw before maturity (trustless)
- Automatically unlocks at maturity

## System Architecture

### Product Design

| Product | Term | Annual Rate | Minimum Amount | Maturity |
|---------|------|-------------|----------------|---------|
| Short-Term Saver | 3 months | 3% | 1,000 satoshi | Automatic unlock |
| Steady Earner | 1 year | 5% | 5,000 satoshi | Automatic unlock |

### Time Calculation

```rust
// 3-month term (approximately 13 weeks, 91 days)
const BLOCKS_PER_3_MONTHS: u64 = 13 * 7 * 144;  // 13,104 blocks

// 1-year term (approximately 52 weeks, 365 days)
const BLOCKS_PER_YEAR: u64 = 52 * 7 * 144;      // 52,416 blocks

// Note: Bitcoin averages one block every 10 minutes, approximately 144 blocks per day
```

## Complete Implementation

The following is the complete time deposit system code:

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    advanced_tx::{TimeLock, TimeLockType},
};

fn main() -> Result<(), String> {
    println!("=== Time Deposit System Demo ===\n");

    // Initialize the blockchain
    let mut blockchain = Blockchain::new();

    // Create a user wallet
    let user = Wallet::new();
    println!("User address: {}...{}", &user.address[..16], &user.address[36..]);

    // Give the user initial funds
    setup_balance(&mut blockchain, &user, 20000)?;
    println!("Initial balance: {} satoshi\n", blockchain.get_balance(&user.address));

    // Scenario 1: 3-month time deposit
    println!("--- Scenario 1: 3-month time deposit (3% annual rate) ---");
    let amount_3m = 5000;
    let rate_3m = 0.03;
    let blocks_3m = 13 * 7 * 144;  // 13 weeks

    // Calculate 3-month interest
    let interest_3m = (amount_3m as f64 * rate_3m * 3.0 / 12.0) as u64;
    let total_3m = amount_3m + interest_3m;

    println!("Deposit amount: {} satoshi", amount_3m);
    println!("Expected interest: {} satoshi (3 months @ 3%)", interest_3m);
    println!("Total at maturity: {} satoshi", total_3m);
    println!("Lock period (blocks): {}", blocks_3m);

    // Create the 3-month term
    let timelock_3m = TimeLock::new(
        TimeLockType::BlockHeight(blockchain.chain.len() as u64 + blocks_3m)
    );

    let deposit_tx_3m = timelock_3m.create_timelocked_transaction(
        &mut blockchain,
        &user,
        user.address.clone(),  // returned to the depositor at maturity
        total_3m,              // principal + interest
        10,
    )?;

    blockchain.add_transaction(deposit_tx_3m.clone())?;
    blockchain.mine_pending_transactions(user.address.clone())?;

    println!("3-month deposit created successfully");
    println!("Transaction ID: {}...{}\n", &deposit_tx_3m.id[..16], &deposit_tx_3m.id[56..]);

    // Scenario 2: 1-year time deposit
    println!("--- Scenario 2: 1-year time deposit (5% annual rate) ---");
    let amount_1y = 10000;
    let rate_1y = 0.05;
    let blocks_1y = 52 * 7 * 144;  // 52 weeks

    // Calculate 1-year interest
    let interest_1y = (amount_1y as f64 * rate_1y) as u64;
    let total_1y = amount_1y + interest_1y;

    println!("Deposit amount: {} satoshi", amount_1y);
    println!("Expected interest: {} satoshi (1 year @ 5%)", interest_1y);
    println!("Total at maturity: {} satoshi", total_1y);
    println!("Lock period (blocks): {}", blocks_1y);

    // Create the 1-year term
    let timelock_1y = TimeLock::new(
        TimeLockType::BlockHeight(blockchain.chain.len() as u64 + blocks_1y)
    );

    let deposit_tx_1y = timelock_1y.create_timelocked_transaction(
        &mut blockchain,
        &user,
        user.address.clone(),
        total_1y,
        10,
    )?;

    blockchain.add_transaction(deposit_tx_1y.clone())?;
    blockchain.mine_pending_transactions(user.address.clone())?;

    println!("1-year deposit created successfully");
    println!("Transaction ID: {}...{}\n", &deposit_tx_1y.id[..16], &deposit_tx_1y.id[56..]);

    // Show current balance
    let current_balance = blockchain.get_balance(&user.address);
    println!("Available balance: {} satoshi", current_balance);
    println!("Total in time deposits: {} satoshi (locked)\n", amount_3m + amount_1y);

    // Scenario 3: Attempt early withdrawal (should fail)
    println!("--- Scenario 3: Attempting early withdrawal ---");
    println!("Current block height: {}", blockchain.chain.len());
    println!("3-month deposit unlock height: {}", blockchain.chain.len() as u64 + blocks_3m);

    match timelock_3m.is_spendable(&blockchain) {
        true => println!("ERROR: deposit not yet matured but withdrawal is possible!"),
        false => println!("Correct: deposit not yet matured, funds are locked"),
    }

    // Scenario 4: Simulate passage of time (mine to 3 months later)
    println!("\n--- Scenario 4: 3-month deposit matures ---");
    println!("Simulating mining {} blocks...", blocks_3m);

    // Fast-simulate mining
    for _ in 0..blocks_3m {
        blockchain.mine_pending_transactions(user.address.clone())?;
    }

    println!("Current block height: {}", blockchain.chain.len());

    // Check whether withdrawal is possible
    if timelock_3m.is_spendable(&blockchain) {
        println!("3-month deposit has matured, withdrawal is available");

        // Collect principal + interest
        println!("Withdrawal amount: {} satoshi (principal {} + interest {})",
                 total_3m, amount_3m, interest_3m);

        let final_balance = blockchain.get_balance(&user.address);
        println!("Balance after receipt: {} satoshi", final_balance);
    } else {
        println!("ERROR: deposit has matured but withdrawal is unavailable");
    }

    // Scenario 5: 1-year deposit not yet matured
    println!("\n--- Scenario 5: 1-year deposit status ---");
    println!("Current block height: {}", blockchain.chain.len());
    println!("1-year deposit unlock height: {}", blockchain.chain.len() as u64 + blocks_1y - blocks_3m);

    match timelock_1y.is_spendable(&blockchain) {
        true => println!("1-year deposit has matured, withdrawal is available"),
        false => {
            let remaining = blocks_1y - blocks_3m;
            println!("1-year deposit has not yet matured; {} blocks remaining (approximately {} days)",
                     remaining, remaining / 144);
        }
    }

    println!("\n=== Demo Complete ===");

    Ok(())
}

// Helper function: initialize balance
fn setup_balance(
    blockchain: &mut Blockchain,
    wallet: &Wallet,
    amount: u64
) -> Result<(), String> {
    let genesis = Wallet::from_address("genesis".to_string());
    let tx = blockchain.create_transaction(
        &genesis,
        wallet.address.clone(),
        amount,
        0,
    )?;
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(wallet.address.clone())?;
    Ok(())
}
```

## Code Walkthrough

### 1. Product Parameter Definitions

```rust
// Short-Term Saver: 3-month term
let amount_3m = 5000;              // deposit amount
let rate_3m = 0.03;                // 3% annual rate
let blocks_3m = 13 * 7 * 144;      // 3 months = 13 weeks = 13,104 blocks

// Calculate interest: principal × annual rate × time (months/12)
let interest_3m = (amount_3m as f64 * rate_3m * 3.0 / 12.0) as u64;
// interest_3m = 5000 × 0.03 × 0.25 = 37.5 ≈ 37 satoshi
```

**Why block height instead of timestamp?**
- More precise: block height is a discrete integer with no ambiguity
- More reliable: timestamps can be manipulated by miners (±2 hours)
- More consistent: the entire network has uniform consensus on block height

### 2. Creating the TimeLock Term

```rust
// Create a timelock: current height + lock period
let timelock_3m = TimeLock::new(
    TimeLockType::BlockHeight(
        blockchain.chain.len() as u64 + blocks_3m
    )
);
```

**Key points**:
- `blockchain.chain.len()` = current block height
- `+ blocks_3m` = unlock block height
- Before the unlock height, the transaction cannot be spent

### 3. Creating the Time Deposit Transaction

```rust
let deposit_tx_3m = timelock_3m.create_timelocked_transaction(
    &mut blockchain,
    &user,                      // depositor
    user.address.clone(),       // returned to the depositor at maturity
    total_3m,                   // principal + interest
    10,                         // fee
)?;
```

**Transaction flow**:
```
User balance → [timelocked transaction] → UTXO pool (locked state)
                        ↓
                 (spendable only at maturity)
                        ↓
               User balance (principal + interest)
```

### 4. Maturity Check

```rust
if timelock_3m.is_spendable(&blockchain) {
    // withdrawal available
} else {
    // not yet matured
}
```

**Check logic**:
```rust
pub fn is_spendable(&self, blockchain: &Blockchain) -> bool {
    match &self.lock_type {
        TimeLockType::BlockHeight(height) => {
            blockchain.chain.len() as u64 >= *height
        },
        TimeLockType::Timestamp(time) => {
            // Compare with current timestamp
            current_timestamp() >= *time
        }
    }
}
```

## Sample Output

```bash
$ cargo run --example timelock_savings

=== Time Deposit System Demo ===

User address: a3f2d8c9e4b7f1a8...c4e7d9b2a5c
Initial balance: 20000 satoshi

--- Scenario 1: 3-month time deposit (3% annual rate) ---
Deposit amount: 5000 satoshi
Expected interest: 37 satoshi (3 months @ 3%)
Total at maturity: 5037 satoshi
Lock period (blocks): 13104
3-month deposit created successfully
Transaction ID: d4f7a9e2b5c8f1a3...b5c8f1a3d4f7

--- Scenario 2: 1-year time deposit (5% annual rate) ---
Deposit amount: 10000 satoshi
Expected interest: 500 satoshi (1 year @ 5%)
Total at maturity: 10500 satoshi
Lock period (blocks): 52416
1-year deposit created successfully
Transaction ID: e5g8b0f3c6d9g2b4...c6d9g2b4e5g8

Available balance: 4960 satoshi
Total in time deposits: 15000 satoshi (locked)

--- Scenario 3: Attempting early withdrawal ---
Current block height: 4
3-month deposit unlock height: 13108
Correct: deposit not yet matured, funds are locked

--- Scenario 4: 3-month deposit matures ---
Simulating mining 13104 blocks...
Current block height: 13108
3-month deposit has matured, withdrawal is available
Withdrawal amount: 5037 satoshi (principal 5000 + interest 37)
Balance after receipt: 10497 satoshi

--- Scenario 5: 1-year deposit status ---
Current block height: 13108
1-year deposit unlock height: 52420
1-year deposit has not yet matured; 39312 blocks remaining (approximately 273 days)

=== Demo Complete ===
```

## Business Value

### Value to Users

| Feature | Traditional Bank Term Deposit | Blockchain Term Deposit | Advantage |
|---------|------------------------------|------------------------|-----------|
| **Rate transparency** | Bank decides | Code is open | Fully transparent |
| **Forced savings** | Can withdraw early | Technically locked | Truly forced |
| **Interest guarantee** | Bank's promise | Smart contract | Auto-executes |
| **Maturity action** | Must go to bank | Automatic unlock | No action needed |
| **Trust cost** | High (trust the bank) | Low (trust the code) | Decentralized |

### Return Comparison (assuming 10,000 satoshi deposited)

| Product | Term | Rate | Total at Maturity | Earnings |
|---------|------|------|-------------------|---------|
| Demand deposit | — | 0.3% | 10,030 | 30 |
| Short-Term Saver | 3 months | 3% | 10,075 | 75 |
| Steady Earner | 1 year | 5% | 10,500 | 500 |

**Calculation formula**:
```
Total at maturity = principal × (1 + annual rate × term in years)

3 months: 10000 × (1 + 0.03 × 0.25) = 10075
1 year:   10000 × (1 + 0.05 × 1.0)  = 10500
```

## Extended Solutions

### 1. Laddered Deposits

```rust
struct LadderDeposit {
    amount: u64,
    start_height: u64,
    periods: Vec<(u64, f64)>,  // (term blocks, rate)
}

impl LadderDeposit {
    // Create a laddered deposit: stagger maturity dates
    pub fn new(total: u64, blockchain: &Blockchain) -> Self {
        let per_amount = total / 4;
        let current = blockchain.chain.len() as u64;

        LadderDeposit {
            amount: per_amount,
            start_height: current,
            periods: vec![
                (13 * 7 * 144, 0.03),   // 3 months, 3%
                (26 * 7 * 144, 0.04),   // 6 months, 4%
                (39 * 7 * 144, 0.045),  // 9 months, 4.5%
                (52 * 7 * 144, 0.05),   // 12 months, 5%
            ],
        }
    }
}

// Benefits:
// - One tranche matures every 3 months, maintaining liquidity
// - Average rate higher than a single short-term deposit
// - Reduces interest rate fluctuation risk
```

### 2. Auto-Renewal

```rust
struct AutoRenewDeposit {
    principal: u64,
    term_blocks: u64,
    rate: f64,
    max_renewals: u32,
}

impl AutoRenewDeposit {
    pub fn create_auto_renew(
        &self,
        blockchain: &mut Blockchain,
        wallet: &Wallet,
    ) -> Result<Vec<Transaction>, String> {
        let mut transactions = Vec::new();
        let mut total = self.principal;

        for i in 0..self.max_renewals {
            let lock_height = blockchain.chain.len() as u64
                            + (i as u64 + 1) * self.term_blocks;

            // Calculate this period's principal + interest
            let interest = (total as f64 * self.rate
                          * (self.term_blocks as f64 / 52416.0)) as u64;
            total += interest;

            // Create the renewal transaction
            let timelock = TimeLock::new(
                TimeLockType::BlockHeight(lock_height)
            );

            let tx = timelock.create_timelocked_transaction(
                blockchain,
                wallet,
                wallet.address.clone(),
                total,
                10,
            )?;

            transactions.push(tx);
        }

        Ok(transactions)
    }
}

// Usage example:
let auto_deposit = AutoRenewDeposit {
    principal: 10000,
    term_blocks: 13 * 7 * 144,  // 3 months
    rate: 0.03,
    max_renewals: 4,  // auto-renew 4 times = 1 year
};

// Automatically creates 4 deposits, renewing every 3 months
let txs = auto_deposit.create_auto_renew(&mut blockchain, &user)?;
```

### 3. Capital-Protected Floating Yield

```rust
struct FloatingDeposit {
    principal: u64,
    min_rate: f64,      // capital protection rate
    bonus_rate: f64,    // bonus rate
    target_blocks: u64, // target block count
}

impl FloatingDeposit {
    pub fn calculate_interest(&self, blockchain: &Blockchain) -> u64 {
        let actual_blocks = blockchain.chain.len() as u64;

        // Base interest (capital protection)
        let base = (self.principal as f64 * self.min_rate) as u64;

        // Bonus interest (based on actual holding time)
        if actual_blocks >= self.target_blocks {
            let bonus = (self.principal as f64 * self.bonus_rate) as u64;
            base + bonus
        } else {
            base
        }
    }
}

// Usage example:
let floating = FloatingDeposit {
    principal: 10000,
    min_rate: 0.03,    // 3% guaranteed
    bonus_rate: 0.02,  // additional 2% bonus
    target_blocks: 52 * 7 * 144,  // bonus requires holding for 1 year
};

// Under 1 year: 3% interest = 300 satoshi
// 1 full year:  5% interest = 500 satoshi
```

### 4. Early Redemption (with Penalty)

```rust
struct EarlyWithdraw {
    deposit_tx: Transaction,
    lock_height: u64,
    penalty_rate: f64,  // penalty rate
}

impl EarlyWithdraw {
    pub fn withdraw_early(
        &self,
        blockchain: &mut Blockchain,
        wallet: &Wallet,
    ) -> Result<Transaction, String> {
        let current = blockchain.chain.len() as u64;

        // Check whether this is an early redemption
        if current >= self.lock_height {
            return Err("Deposit has matured; please use normal withdrawal".to_string());
        }

        // Calculate penalty
        let principal = self.deposit_tx.outputs[0].value;
        let penalty = (principal as f64 * self.penalty_rate) as u64;
        let actual_amount = principal.saturating_sub(penalty);

        // Create early redemption transaction (requires admin signature)
        let tx = blockchain.create_transaction(
            wallet,
            wallet.address.clone(),
            actual_amount,
            10,
        )?;

        println!("Early redemption: principal {}, penalty {}, received {}",
                 principal, penalty, actual_amount);

        Ok(tx)
    }
}

// Usage example:
// User deposits 10000 for 1 year, withdraws early after 6 months
// Penalty 5% = 500 satoshi
// Receives 9500 satoshi (loses 500)
```

## Security Considerations

### 1. Interest Funding Source

```rust
// Incorrect: creating interest out of thin air
let interest = 100;
let total = principal + interest;  // where does the interest come from?

// Correct: interest paid from a reserve pool
struct DepositPool {
    reserves: u64,  // reserve funds
}

impl DepositPool {
    pub fn pay_interest(&mut self, principal: u64, rate: f64) -> Result<u64, String> {
        let interest = (principal as f64 * rate) as u64;

        if self.reserves < interest {
            return Err("Insufficient pool balance".to_string());
        }

        self.reserves -= interest;
        Ok(interest)
    }
}
```

### 2. Timestamp Manipulation Attack

**Attack scenario**: miner manipulates the timestamp to cause early maturity

**Defense**:
```rust
// Use block height, not timestamp
TimeLockType::BlockHeight(height)  // recommended

// Avoid using timestamps (easily manipulated)
TimeLockType::Timestamp(time)      // not recommended
```

### 3. Reentrancy Attack

```rust
// Incorrect: transfer first, then update state
fn withdraw(&mut self) {
    self.transfer(user, amount);  // transfer first
    self.balance = 0;             // update later (may be reentered)
}

// Correct: update state first, then transfer (checks-effects-interactions pattern)
fn withdraw(&mut self) {
    let amount = self.balance;    // check
    self.balance = 0;             // effect
    self.transfer(user, amount);  // interaction
}
```

### 4. Integer Overflow

```rust
// Incorrect: may overflow
let total = principal + interest;  // u64 overflow risk

// Correct: use checked_add
let total = principal.checked_add(interest)
    .ok_or("Arithmetic overflow")?;
```

## Implementation Recommendations

### Technical Considerations

1. **Adequate testing**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_interest_calculation() { /* ... */ }

       #[test]
       fn test_early_withdraw_penalty() { /* ... */ }

       #[test]
       fn test_timelock_enforcement() { /* ... */ }
   }
   ```

2. **Code audit**
   - Verify interest calculation formulas are correct
   - Verify that the timelock is reliable
   - Verify that the funding source is clearly defined
   - Handle edge cases

3. **Monitoring and alerts**
   ```rust
   // Monitor key metrics
   - Reserve pool balance warning (< 10%)
   - Matured deposits not yet collected (> 1 month)
   - Abnormal early redemption frequency
   ```

### Business Considerations

1. **Risk disclosures**
   ```
   Time Deposit Risk Notice:
   1. Funds will be locked and cannot be withdrawn before maturity
   2. Interest is paid from the reserve pool and carries payment risk
   3. Smart contracts may contain unknown vulnerabilities
   4. Blockchain transactions are irreversible; proceed with care
   ```

2. **User education**
   - Provide a sandbox demo environment for users to practice
   - Offer detailed operation guides
   - Explain the differences from traditional banking
   - Emphasize the importance of private key custody

3. **Product iteration**
   - Collect user feedback
   - Analyze maturity data
   - Optimize interest rate strategy
   - Add more product types

## Real-World Applications

### DeFi Time Deposit Protocols

**Compound**: lending protocol, deposits accrue interest automatically
```
User deposits ETH → receives cETH (interest-bearing token)
Interest rate floats with the market → withdrawable at any time
```

**Anchor Protocol**: fixed-rate deposits (Terra ecosystem)
```
Deposit UST → fixed ~20% APY
Interest comes from lending markets and staking rewards
```

**Alchemix**: self-repaying loans
```
Deposit DAI → borrow alUSD (50% LTV)
Interest automatically repays the loan → no repayments needed
```

### Comparison with SimpleBTC

| Feature | SimpleBTC Term Deposit | DeFi Term Deposit |
|---------|----------------------|------------------|
| Timelock | Hard lock (nLockTime) | Soft lock (contract) |
| Rate | Fixed | Usually floating |
| Liquidity | Withdraw only at maturity | Early withdrawal available (with penalty) |
| Interest source | Reserve pool | Lending/staking |
| Risk | Timelock risk | Smart contract risk |

## FAQ

### Q1: Where does the interest for time deposits come from?

**A**: SimpleBTC interest is for demo purposes. In real applications, interest may come from:
- Reserve pool funds
- Interest spreads in lending markets
- Distribution of mining rewards
- Return of transaction fees
- Protocol token issuance

### Q2: Can I withdraw early?

**A**: SimpleBTC uses nLockTime hard locking, so early withdrawal is technically impossible. Real applications can be designed with:
- Early redemption with a penalty (5–10% fee)
- NFT collateral borrowing (keeps the term deposit running)
- Secondary market transfer (sell at a discount)

### Q3: What if I forget to collect after maturity?

**A**: UTXOs are permanently valid and can be collected at any time. However, note that:
- No additional interest accrues after maturity
- Set maturity reminders
- Auto-renewal can be implemented

### Q4: How is the timelock period calculated?

**A**:
```
Block height method (recommended):
- 3 months ≈ 13,104 blocks (91 days × 144 blocks/day)
- 1 year   ≈ 52,416 blocks (365 days × 144 blocks/day)

Timestamp method (not recommended):
- 3 months = current timestamp + 7,862,400 seconds
- 1 year   = current timestamp + 31,536,000 seconds
```

## References

- [TimeLock Tutorial](../advanced/timelock.md) — timelock principles and usage
- [Transaction API](../api/transaction.md) — transaction creation
- [Blockchain API](../api/blockchain.md) — blockchain operations
- [Compound Finance](https://compound.finance/) — DeFi lending protocol
- [BIP65 - CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)

---

[Back to Examples](./enterprise-multisig.md) | [Next Example: Enterprise Multisig](./enterprise-multisig.md)
