# Time Lock (TimeLock / nLockTime)

Time locks are an important Bitcoin feature that prevent a transaction from being confirmed before a specific time or block height.

## Overview

### What is a Time Lock?

A time lock allows a transaction to be used only at some future point in time, implementing a "deferred payment" feature.

**Example**:
```
Alice creates a transaction: 1 BTC → Bob
Time lock: January 1, 2025

Before January 1, 2025:
  ❌ Transaction cannot be confirmed
  ❌ Miners refuse to pack it

After January 1, 2025:
  ✓ Transaction can be confirmed
  ✓ Miners can pack it
```

---

## Two Types

### 1. Unix Timestamp-Based

```rust
// locktime >= 500,000,000
let unlock_time = 1735689600;  // 2025-01-01 00:00:00
let timelock = TimeLock::new_time_based(unlock_time);
```

**Characteristics**:
- Unit: seconds
- Suitable for precise time control
- Affected by system time

**Use cases**:
- Salary payment (1st of each month)
- Bond maturity (fixed date)
- Term deposit (3/6/12 months)

### 2. Block Height-Based

```rust
// locktime < 500,000,000
let unlock_height = 800000;  // Block #800,000
let timelock = TimeLock::new_block_based(unlock_height);
```

**Characteristics**:
- Unit: blocks
- More precise (approximately 10 minutes per block)
- Not affected by system time

**Use cases**:
- More precise time control
- Avoid timestamp manipulation
- Smart contract triggers

**Time estimation**:
```
1 block  ≈ 10 minutes
6 blocks ≈ 1 hour
144 blocks ≈ 1 day
1008 blocks ≈ 1 week
4032 blocks ≈ 1 month
```

---

## TimeLock Implementation

### Data Structure

```rust
pub struct TimeLock {
    pub locktime: u64,         // Lock time / height
    pub is_block_height: bool, // true: block height, false: timestamp
}
```

### Methods

#### `new_time_based`

```rust
pub fn new_time_based(timestamp: u64) -> Self
```

Creates a time-based time lock.

**Parameters**:
- `timestamp` - Unix timestamp (seconds)

**Example**:
```rust
use bitcoin_simulation::advanced_tx::TimeLock;
use std::time::{SystemTime, UNIX_EPOCH};

let current_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

// Unlock after 3 months
let three_months = 90 * 24 * 3600;
let unlock_time = current_time + three_months;
let timelock = TimeLock::new_time_based(unlock_time);

println!("Locked until: {}", format_timestamp(unlock_time));
```

#### `new_block_based`

```rust
pub fn new_block_based(block_height: u64) -> Self
```

Creates a block height-based time lock.

**Parameters**:
- `block_height` - Target block height

**Example**:
```rust
let current_height = blockchain.chain.len() as u64;

// Unlock after 1000 blocks (approximately 1 week)
let unlock_height = current_height + 1000;
let timelock = TimeLock::new_block_based(unlock_height);

println!("Locked until block #{}", unlock_height);
```

#### `is_mature`

```rust
pub fn is_mature(&self, current_time: u64, current_height: u64) -> bool
```

Checks whether the time lock has expired.

**Parameters**:
- `current_time` - Current Unix timestamp
- `current_height` - Current block height

**Return value**:
- `true` - Expired; can be used
- `false` - Not yet expired; still locked

**Example**:
```rust
let timelock = TimeLock::new_time_based(unlock_time);

if timelock.is_mature(current_time, 0) {
    println!("✓ Expired; can be spent");
} else {
    let remaining = unlock_time - current_time;
    println!("🔒 Still locked; {} seconds remaining", remaining);
}
```

---

## Use Cases

### Use Case 1: Term Deposit

```rust
fn savings_account_demo() -> Result<(), String> {
    println!("=== Term Deposit Demo ===\n");

    let alice = Wallet::new();
    let mut blockchain = Blockchain::new();

    // Initialize balance
    setup_balance(&mut blockchain, &alice, 100000)?;

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Product 1: 3-month term deposit
    println!("--- Product A: 3-Month Term Deposit ---");
    let three_months = 90 * 24 * 3600;
    let maturity_3m = current_time + three_months;
    let deposit_3m = TimeLock::new_time_based(maturity_3m);

    println!("Deposit amount: 30,000 sat");
    println!("Term: 3 months");
    println!("Maturity date: {}", format_date(maturity_3m));
    println!("Annual interest rate: 3%\n");

    // Product 2: 1-year term deposit
    println!("--- Product B: 1-Year Term Deposit ---");
    let one_year = 365 * 24 * 3600;
    let maturity_1y = current_time + one_year;
    let deposit_1y = TimeLock::new_time_based(maturity_1y);

    println!("Deposit amount: 50,000 sat");
    println!("Term: 1 year");
    println!("Maturity date: {}", format_date(maturity_1y));
    println!("Annual interest rate: 5%\n");

    // Check maturity status
    println!("--- Current Status Check ---");
    println!("Current time: {}", format_date(current_time));

    if deposit_3m.is_mature(current_time, 0) {
        println!("✓ 3-month deposit has matured; can be withdrawn");
        let interest = 30000 * 3 / 100 / 4;  // Quarterly interest
        println!("  Principal + interest: {} sat", 30000 + interest);
    } else {
        let days_left = (maturity_3m - current_time) / 86400;
        println!("🔒 3-month deposit is locked");
        println!("  Remaining: {} days", days_left);
    }

    if deposit_1y.is_mature(current_time, 0) {
        println!("✓ 1-year deposit has matured; can be withdrawn");
        let interest = 50000 * 5 / 100;  // Annual interest
        println!("  Principal + interest: {} sat", 50000 + interest);
    } else {
        let days_left = (maturity_1y - current_time) / 86400;
        println!("🔒 1-year deposit is locked");
        println!("  Remaining: {} days", days_left);
    }

    Ok(())
}
```

**Output**:
```
=== Term Deposit Demo ===

--- Product A: 3-Month Term Deposit ---
Deposit amount: 30,000 sat
Term: 3 months
Maturity date: 2025-03-15 00:00:00
Annual interest rate: 3%

--- Product B: 1-Year Term Deposit ---
Deposit amount: 50,000 sat
Term: 1 year
Maturity date: 2025-12-15 00:00:00
Annual interest rate: 5%

--- Current Status Check ---
Current time: 2024-12-15 00:00:00
🔒 3-month deposit is locked
  Remaining: 90 days
🔒 1-year deposit is locked
  Remaining: 365 days
```

---

### Use Case 2: Inheritance Planning

```rust
fn inheritance_planning() -> Result<(), String> {
    println!("=== Inheritance Plan ===\n");

    let owner = Wallet::new();
    let heir = Wallet::new();
    let lawyer = Wallet::new();

    println!("Participants:");
    println!("  Owner: {}", &owner.address[..16]);
    println!("  Heir: {}", &heir.address[..16]);
    println!("  Lawyer: {}\n", &lawyer.address[..16]);

    let current_time = current_timestamp();

    // Plan: after 1 year of inactivity, assets automatically transfer to the heir
    println!("--- Plan Design ---");
    println!("Normal situation:");
    println!("  Requires: owner + heir (2-of-2)");
    println!("  Protects privacy; prevents unilateral transfer\n");

    println!("Emergency situation (after 1 year):");
    println!("  Owner is unreachable or deceased");
    println!("  Time lock has expired");
    println!("  Heir can operate independently\n");

    // Create time lock transaction
    let one_year = 365 * 24 * 3600;
    let inheritance_time = current_time + one_year;
    let timelock = TimeLock::new_time_based(inheritance_time);

    println!("--- Time Lock Configuration ---");
    println!("Trigger time: {}", format_date(inheritance_time));
    println!("Trigger condition: no transaction signed by owner within 1 year\n");

    // Periodic check (performed by lawyer)
    println!("--- Periodic Check ---");
    let last_activity = current_time;
    let inactive_period = current_time - last_activity;

    if inactive_period > one_year {
        if timelock.is_mature(current_time, 0) {
            println!("✓ Time lock triggered");
            println!("✓ Inheritance process started");
            println!("✓ Assets can be transferred to heir");
        }
    } else {
        let days_remaining = (one_year - inactive_period) / 86400;
        println!("🔒 Normal status");
        println!("{} days until inheritance trigger", days_remaining);
    }

    Ok(())
}
```

---

### Use Case 3: Salary Payment

```rust
fn salary_payment_system() -> Result<(), String> {
    println!("=== Salary Payment System ===\n");

    let company = Wallet::new();
    let employees: Vec<_> = (0..5)
        .map(|i| (format!("Employee {}", i+1), Wallet::new()))
        .collect();

    let current_time = current_timestamp();

    println!("Company address: {}", &company.address[..16]);
    println!("Number of employees: {}\n", employees.len());

    // Pay salary on the 1st of each month
    println!("--- Salary Payment Schedule ---");

    for month in 1..=3 {
        // Calculate the timestamp for the 1st of the next month
        let payment_date = calculate_first_day_of_month(current_time, month);
        let timelock = TimeLock::new_time_based(payment_date);

        println!("Month {} salary:", month);
        println!("  Payment date: {}", format_date(payment_date));

        if timelock.is_mature(current_time, 0) {
            println!("  Status: ✓ Ready to pay");

            for (name, wallet) in &employees {
                println!("    {} → {} sat", name, 10000);
            }
        } else {
            let days_until = (payment_date - current_time) / 86400;
            println!("  Status: 🔒 Locked");
            println!("  Countdown: {} days", days_until);
        }
        println!();
    }

    println!("--- Advantages ---");
    println!("✓ Automated payment");
    println!("✓ Cannot be diverted early");
    println!("✓ Employees have predictable income");
    println!("✓ Reduced administrative costs");

    Ok(())
}
```

---

### Use Case 4: Crowdfunding Refund

```rust
fn crowdfunding_refund() -> Result<(), String> {
    println!("=== Crowdfunding Refund Mechanism ===\n");

    let project_owner = Wallet::new();
    let backers: Vec<_> = (0..10).map(|_| Wallet::new()).collect();

    let current_time = current_timestamp();

    println!("--- Crowdfunding Project ---");
    println!("Target amount: 1,000,000 sat");
    println!("Currently raised: 500,000 sat");
    println!("Deadline: 30 days from now\n");

    // If the target is not met after 30 days, automatically refund
    let deadline = current_time + 30 * 86400;
    let refund_timelock = TimeLock::new_time_based(deadline);

    println!("--- Refund Time Lock ---");
    println!("Trigger condition: target not met after 30 days");
    println!("Trigger time: {}", format_date(deadline));
    println!("Refund method: automatically returned to backers\n");

    // Check status
    if refund_timelock.is_mature(current_time, 0) {
        println!("--- Project failed; executing refund ---");
        for (i, backer) in backers.iter().enumerate() {
            println!("✓ Refund to backer #{}: {} sat", i+1, 50000);
        }
    } else {
        let days_left = (deadline - current_time) / 86400;
        println!("--- Crowdfunding in progress ---");
        println!("Time remaining: {} days", days_left);
        println!("Still needed: 500,000 sat");
    }

    Ok(())
}
```

---

## Advanced Usage

### TimeLock + MultiSig

Combining multi-sig for more complex logic:

```rust
fn timelock_multisig_combination() -> Result<(), String> {
    let owner = Wallet::new();
    let heir = Wallet::new();
    let lawyer = Wallet::new();

    // Normal: 2-of-2 (owner + heir)
    let normal_multisig = MultiSigAddress::new(
        2,
        vec![owner.public_key.clone(), heir.public_key.clone()]
    )?;

    // After 1 year: 2-of-3 (any two parties)
    let emergency_multisig = MultiSigAddress::new(
        2,
        vec![owner.public_key, heir.public_key, lawyer.public_key]
    )?;

    let one_year = 365 * 24 * 3600;
    let timelock = TimeLock::new_time_based(current_timestamp() + one_year);

    println!("=== TimeLock + MultiSig Combination ===");
    println!("\nNormal period (first year):");
    println!("  Multi-sig address: {}", &normal_multisig.address[..16]);
    println!("  Requirement: owner + heir (2-of-2)");

    println!("\nEmergency period (after one year):");
    println!("  Multi-sig address: {}", &emergency_multisig.address[..16]);
    println!("  Requirement: any two parties (2-of-3)");
    println!("  Possible combinations:");
    println!("    - owner + heir");
    println!("    - owner + lawyer");
    println!("    - heir + lawyer");

    Ok(())
}
```

---

## Technical Details

### nLockTime Field

In actual Bitcoin transactions:

```rust
struct Transaction {
    version: u32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,  // Time lock field
}
```

**Rules**:
```
if locktime < 500,000,000:
    # Block height mode
    if current_block_height >= locktime:
        can confirm
    else:
        reject

else:
    # Timestamp mode
    if current_timestamp >= locktime:
        can confirm
    else:
        reject
```

### nSequence and Time Locks

For a time lock to be enabled, nSequence must be < 0xFFFFFFFF:

```rust
// Enable time lock
input.sequence = 0xFFFFFFFD;

// Disable time lock (final transaction)
input.sequence = 0xFFFFFFFF;
```

---

## Security Considerations

### 1. Timestamp Manipulation

**Problem**: Miners may manipulate block timestamps

**Constraints**:
- Timestamp cannot be earlier than the median of the previous 11 blocks
- Cannot be more than 2 hours later than the current time

**Recommendation**: Using block height is more reliable

### 2. Emergencies

**Problem**: A time lock cannot be cancelled

**Solutions**:
```rust
// Solution 1: Use RBF to replace before expiry
if !timelock.is_mature(...) && need_cancel {
    replace_with_non_locked_tx();
}

// Solution 2: Double spend (before expiry)
create_alternative_tx_without_timelock();
```

### 3. Key Loss

**Problem**: Key lost before expiry

**Recommendation**:
- Use multi-sig to reduce risk
- Back up keys
- Set up recovery mechanisms

---

## Relationship to CLTV/CSV

### CheckLockTimeVerify (CLTV)

**Introduced in BIP65**:
```
OP_CLTV opcode
Locks a single UTXO
More flexible
```

**nLockTime vs CLTV**:
```
nLockTime: locks the entire transaction
CLTV: locks a single output (more flexible)
```

### CheckSequenceVerify (CSV)

**Introduced in BIP112**:
```
OP_CSV opcode
Relative time lock
Calculated from the time the UTXO was created
```

---

## Best Practices

### 1. Choose the Right Type

```rust
// Exact date: use timestamp
let birthday = to_timestamp("2025-01-01");
let timelock = TimeLock::new_time_based(birthday);

// Relative delay: use block height
let blocks_1week = 1008;  // About 1 week
let timelock = TimeLock::new_block_based(current_height + blocks_1week);
```

### 2. User-Friendly Time Display

```rust
fn display_timelock_status(timelock: &TimeLock, current_time: u64, current_height: u64) {
    if timelock.is_mature(current_time, current_height) {
        println!("✓ Unlocked");
    } else {
        if timelock.is_block_height {
            let blocks_left = timelock.locktime - current_height;
            let hours = blocks_left * 10 / 60;  // Approximately 10 minutes per block
            println!("🔒 Locked; {} blocks remaining (approximately {} hours)", blocks_left, hours);
        } else {
            let seconds_left = timelock.locktime - current_time;
            let days = seconds_left / 86400;
            println!("🔒 Locked; {} days remaining", days);
        }
    }
}
```

### 3. Testing Time Locks

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_timelock() {
        let current = 1000000;
        let future = 2000000;

        let timelock = TimeLock::new_time_based(future);

        // Not yet expired
        assert!(!timelock.is_mature(current, 0));

        // Expired
        assert!(timelock.is_mature(future + 1, 0));
    }
}
```

---

## References

- [BIP65 - CLTV](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
- [BIP112 - CSV](https://github.com/bitcoin/bips/blob/master/bip-0112.mediawiki)
- [Term Deposit Example](../examples/timelock-savings.md)
- [MultiSig Tutorial](./multisig.md)

---

**Summary**: Time locks are a key technology for implementing deferred payments and smart contracts. Used appropriately, they enable a wide range of applications such as term deposits, inheritance planning, and salary payments.

[Back to Advanced Features](./README.md)
