# Core Concepts

This section explains the fundamental concepts of SimpleBTC and Bitcoin in detail.

## The UTXO Model

### What is a UTXO?

UTXO (Unspent Transaction Output) is a core concept in Bitcoin. It represents an output from a previous transaction that has not yet been spent.

**Account Model vs. UTXO Model:**

| Feature | Account Model (Ethereum) | UTXO Model (Bitcoin) |
|---------|--------------------------|----------------------|
| Balance storage | Each account has a balance field | Computed from all UTXOs |
| State | Account state (balance, nonce) | Stateless (only a UTXO set) |
| Transfer | A account -100, B account +100 | Consume A's UTXO, create new UTXO for B |
| Privacy | Weaker (same address reused) | Stronger (new address each time) |
| Parallelism | Weaker (transactions on same account must be sequential) | Stronger (different UTXOs can be processed in parallel) |

### UTXO Example

```
Alice has 3 UTXOs:
  UTXO1: 5 BTC (received from Bob)
  UTXO2: 3 BTC (received from Charlie)
  UTXO3: 2 BTC (mining reward)

Alice's total balance: 5 + 3 + 2 = 10 BTC
```

### UTXO Lifecycle

```
1. Creation
   Transaction output → Added to UTXO set

2. Existence
   UTXO set → Can be queried and spent

3. Spending
   Referenced by a transaction input → Removed from UTXO set

4. New UTXO creation
   Transaction output → New UTXO added to set
```

### Change Mechanism

A UTXO must be spent in full; it cannot be partially spent:

```rust
// Alice wants to send 3 BTC to Bob, but only has one UTXO worth 5 BTC

Inputs:
  - UTXO: 5 BTC (Alice's)

Outputs:
  - Output 1: 3 BTC → Bob
  - Output 2: 1.999 BTC → Alice (change)
  - Fee: 0.001 BTC → Miner (inputs - outputs)
```

## Blocks and the Blockchain

### Block Structure

```
┌─────────────────────────────────┐
│        Block Header              │
├─────────────────────────────────┤
│ index: 123                       │ Block height
│ timestamp: 1703001234            │ Timestamp
│ previous_hash: 0x00012ab...     │ Parent block hash
│ merkle_root: 0xabc123...        │ Merkle tree root
│ nonce: 2847563                  │ Proof of Work
│ hash: 0x000034cd...             │ Current block hash
├─────────────────────────────────┤
│        Block Body                │
├─────────────────────────────────┤
│ Transaction 1 (Coinbase)        │ Mining reward
│ Transaction 2                   │ Regular transaction
│ Transaction 3                   │ Regular transaction
│ ...                             │
└─────────────────────────────────┘
```

### Chain Structure

```
Genesis Block → Block 1 → Block 2 → ... → Latest Block
    ↓              ↓          ↓                  ↓
  hash=A         hash=B     hash=C            hash=Z
  prev=0         prev=A     prev=B            prev=Y
```

Each block points to its parent block via `previous_hash`, forming an immutable chain.

### Why is it Immutable?

1. **Hash linking**: Changing any transaction changes the block's hash
2. **Cascade invalidation**: A changed block hash breaks the `previous_hash` of all subsequent blocks
3. **Computational cost**: Tampering with history requires re-mining all subsequent blocks
4. **Longest chain rule**: An attacker would need to outpace the entire network — nearly impossible (except with a 51% attack)

## Proof of Work (PoW)

### Mining Principle

Find a nonce value such that the block hash satisfies the difficulty requirement:

```rust
target = "000..." // difficulty leading zeros

while hash(block_header + nonce) >= target {
    nonce++;
}
```

### Difficulty Example

```
difficulty = 3 (demo)
target = "000..."

Valid hashes:
  ✅ 0003ab4f9c2d...
  ✅ 000f12e8a3b9...

Invalid hashes:
  ❌ 001a3f2e8d4c...  (only 2 leading zeros)
  ❌ 0123456789ab...  (only 1 leading zero)
```

### Difficulty and Security

| Difficulty | Average Attempts | Use Case |
|------------|-----------------|----------|
| 1 | 16 | Testing |
| 3 | 4,096 | Demo |
| 5 | 1,048,576 | Small network |
| 10 | ~10¹² | Private chain |
| 20 | ~10²⁴ | Bitcoin-level |

Bitcoin's actual difficulty is approximately 70-80 bits, with the global hash rate in the hundreds of EH/s.

### Why is PoW Needed?

1. **Prevent spam attacks**: Creating a block requires computational cost
2. **Fair competition**: Higher hash power means higher probability of winning
3. **Decentralization**: Anyone can participate in mining
4. **Economic incentive**: Miners receive rewards (Coinbase + fees)

## Merkle Tree

### Structure Example

Merkle tree for 4 transactions:

```
              Root Hash
             /         \
          H(AB)       H(CD)
         /    \       /    \
       H(A)  H(B)  H(C)  H(D)
        ↑     ↑     ↑     ↑
       Tx1   Tx2   Tx3   Tx4
```

Construction process:
1. Compute the hash of each transaction (leaf nodes)
2. Pair them up and compute the parent node hash
3. Repeat until only one root hash remains
4. The root hash is stored in the block header

### SPV Verification (Lightweight Verification)

No need to download the entire block — only the block header and Merkle proof are needed:

```
Verify that Tx2 is in the block:

Required:
  - Hash of Tx2
  - Merkle proof: [H(A), H(CD)]
  - Root Hash from the block header

Verification:
  1. Compute H(B) = hash(Tx2)
  2. Compute H(AB) = hash(H(A) + H(B))
  3. Compute Root = hash(H(AB) + H(CD))
  4. Compare computed Root with the Root in the block header

✅ Match → Tx2 is indeed in the block
❌ No match → Tx2 is absent or has been tampered with
```

### Advantages of SPV

- **Lightweight**: Only needs the block header (~80 bytes), not the full block (1-2 MB)
- **Fast**: O(log n) verification complexity
- **Mobile-friendly**: Can run on phone wallets
- **Secure**: Protected by PoW; no need to trust a third party

## Cryptographic Foundations

### Hash Function (SHA256)

**Properties**:
- Deterministic: the same input always produces the same output
- Fast to compute: millisecond-level
- Irreversible: cannot reverse-engineer the input from the hash
- Collision-resistant: finding two inputs with the same hash is practically impossible
- Avalanche effect: a tiny change in input produces a completely different hash

**Example**:
```
hash("hello") = 2cf24dba5fb0a30e...
hash("hallo") = d3751d33f9cd5049...  (completely different!)
```

### Digital Signatures (Simplified ECDSA)

**Real Bitcoin**:
```
1. Private key (256-bit random number)
   ↓ Elliptic curve operation
2. Public key (elliptic curve point)
   ↓ SHA256 + RIPEMD160
3. Address (Base58 encoded)
```

**SimpleBTC Simplified**:
```
1. Private key (random string)
   ↓ SHA256
2. Public key (hash value)
   ↓ SHA256, first 20 bytes
3. Address (hexadecimal string)
```

### Signature Verification

```rust
// Signing
signature = hash(private_key + data)

// Verification (simplified)
verify(public_key, data, signature) -> bool
```

Real Bitcoin uses the ECDSA algorithm, which is mathematically provably secure.

## Transaction Structure

### Transaction Anatomy

```rust
Transaction {
    id: "abc123...",           // Transaction hash
    inputs: [                  // Inputs (which UTXOs to spend)
        TxInput {
            txid: "prev_tx",   // Referenced transaction ID
            vout: 0,           // Output index
            signature: "...",  // Signature
            pub_key: "...",   // Public key
        }
    ],
    outputs: [                 // Outputs (which new UTXOs to create)
        TxOutput {
            value: 3000,       // Amount (satoshi)
            pub_key_hash: "bob_address",
        },
        TxOutput {
            value: 6990,       // Change
            pub_key_hash: "alice_address",
        }
    ],
    timestamp: 1703001234,
    fee: 10,                   // Transaction fee
}
```

### Transaction Validation

When a miner validates a transaction, it checks:

1. ✅ **Valid signature**: Each input's signature is correct
2. ✅ **UTXO exists**: The referenced UTXO is in the UTXO set
3. ✅ **No double-spend**: The UTXO has not been spent by another transaction
4. ✅ **Sufficient balance**: Total inputs ≥ total outputs
5. ✅ **Correct format**: Conforms to the protocol specification

### Coinbase Transaction

The first transaction in every block, used to distribute the mining reward:

```rust
Transaction {
    id: "coinbase_tx",
    inputs: [
        TxInput {
            txid: "",          // Empty (does not reference a UTXO)
            vout: 0,
            signature: "coinbase",
            pub_key: "coinbase",
        }
    ],
    outputs: [
        TxOutput {
            value: 50 + total_fees,  // Reward + fees
            pub_key_hash: "miner_address",
        }
    ],
    fee: 0,
}
```

## Consensus Mechanism

### Longest Chain Rule

When a fork occurs, the network selects the chain with the most accumulated work:

```
     Block 3a (PoW difficulty 3)
    /
Block 2
    \
     Block 3b → Block 4b (PoW difficulty 3)
```

The chain containing Block 4b has greater total difficulty and becomes the main chain. Block 3a is orphaned.

### Why the Longest Chain?

- **Work**: A longer chain represents more computational investment
- **Majority consensus**: Honest nodes always mine on the longest chain
- **Attack difficulty**: An attacker would need more than 51% of the total network hash rate

### The 6-Confirmation Rule

```
Your Tx → Block N → N+1 → N+2 → N+3 → N+4 → N+5 → N+6
          0 conf   1 conf 2 conf 3 conf 4 conf 5 conf 6 conf
```

- 0 confirmations: May be double-spent (RBF)
- 1 confirmation: Relatively safe (small payments)
- 3 confirmations: Safe (medium amounts)
- 6 confirmations: Very safe (large transfers)

## Fee Market

### Fee Rate Calculation

```
fee_rate = fee / transaction_size (sat/byte)
```

### Priority

Miner transaction selection strategy:

```rust
// Sort by fee rate from high to low
transactions.sort_by(|a, b| {
    b.fee_rate().cmp(&a.fee_rate())
});
```

Higher fee-rate transactions are packaged first.

### Fee Recommendations

| Urgency | Fee Rate | Confirmation Time |
|---------|----------|-------------------|
| Low priority | 1-5 sat/byte | Several hours |
| Medium priority | 5-20 sat/byte | 30-60 minutes |
| High priority | 20-50 sat/byte | 10-20 minutes |
| Urgent | 50+ sat/byte | Next block |

## Network Parameters

### Time-Related

- **Block time**: ~10 minutes (maintained via difficulty adjustment)
- **Difficulty adjustment**: Every 2,016 blocks (~2 weeks)
- **Halving cycle**: Every 210,000 blocks (~4 years)

### Economic Parameters

- **Initial reward**: 50 BTC
- **Current reward**: 3.125 BTC (after the 2024 halving)
- **Total supply**: 21 million BTC (never increases)
- **Smallest unit**: 1 satoshi = 0.00000001 BTC

### Size Limits

- **Block size**: 1 MB (legacy) / 4 MB (SegWit)
- **Transaction size**: ~250-500 bytes on average
- **Transactions per block**: ~2,000-3,000

## Next Steps

Now that you understand the core concepts, continue learning:

- [Wallet Management](./wallet.md) - Deep dive into keys and addresses
- [Transaction Processing](./transactions.md) - Transaction creation and validation
- [Blockchain Operations](./blockchain.md) - Mining and block management
- [UTXO Management](./utxo.md) - UTXO set operations

---

💡 **Self-Quiz**: Try answering the following questions to test your understanding:

1. What is the main difference between the UTXO model and the account model?
2. Why is the blockchain immutable?
3. How does the Merkle tree enable SPV verification?
4. What is the purpose of Proof of Work?
5. What is the change mechanism?
