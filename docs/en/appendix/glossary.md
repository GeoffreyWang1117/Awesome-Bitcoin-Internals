# Glossary

Core terminology related to Bitcoin and blockchain.

## A

### Address
A unique identifier used to receive bitcoin. Generated from a public key through hashing and encoding.

**Example**: `1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa`

**Types**:
- P2PKH (starts with 1): legacy address
- P2SH (starts with 3): script/multisig address
- Bech32 (starts with bc1): SegWit address

---

### ACID
Four properties of database transactions:
- **Atomicity**: all-or-nothing execution
- **Consistency**: maintains a consistent data state
- **Isolation**: concurrent transactions do not interfere with each other
- **Durability**: committed changes are persisted permanently

SimpleBTC transaction processing conforms to ACID properties.

---

## B

### Block
A data structure containing a list of transactions, linked by hashes to form a blockchain.

**Contains**:
- Block header (index, timestamp, hash, previous_hash, merkle_root, nonce)
- Transaction list (the first is the Coinbase transaction)

**Size**: approximately 1–4 MB in Bitcoin

---

### Blockchain
A chronologically linked sequence of blocks, made tamper-resistant through cryptography.

**Properties**:
- Decentralized
- Tamper-resistant
- Transparent and verifiable
- Trustless (no third party required)

---

### Block Height
The position of a block in the chain. The genesis block has height 0.

**Example**: If the blockchain contains 100 blocks, the latest block has height 99.

---

### BIP (Bitcoin Improvement Proposal)
A proposal for improving the Bitcoin protocol.

**Important BIPs**:
- BIP11: M-of-N multisig
- BIP16: P2SH (Pay-to-Script-Hash)
- BIP32: Hierarchical Deterministic Wallets
- BIP39: Mnemonic phrases
- BIP125: RBF (Replace-By-Fee)

---

## C

### Coinbase Transaction
The first transaction in a block, used to pay the mining reward to the miner.

**Characteristics**:
- No valid inputs (does not spend UTXOs)
- Creates new bitcoin
- Includes block reward + transaction fees

**Example**:
```rust
Transaction::new_coinbase(
    miner_address,
    50,           // block reward
    timestamp,
    total_fees,   // sum of fees
)
```

---

### Cold Wallet
A wallet that stores private keys offline, not connected to the internet.

**Types**:
- Hardware wallets (Ledger, Trezor)
- Paper wallets
- Air-gapped computers

**Advantage**: Extremely high security
**Disadvantage**: Inconvenient to use

---

### Confirmation
The number of times a transaction has been included in a block and subsequently extended by additional blocks.

**Confirmation counts**:
- 0 confirmations: in the mempool, not yet mined
- 1 confirmation: included in a block
- 6 confirmations: very secure (Bitcoin standard)

**Time**: approximately 10 minutes per confirmation in Bitcoin

---

## D

### Difficulty
The computational difficulty of mining, which determines how hard it is to find a valid block hash.

**SimpleBTC**:
```rust
blockchain.difficulty = 3;  // 3 leading zeros
```

**Bitcoin**: dynamically adjusted every 2016 blocks (approximately 2 weeks), targeting a 10-minute block time.

---

### Double Spending
An attack that attempts to spend the same bitcoin twice.

**Defense mechanisms**:
1. UTXO model (each UTXO can only be spent once)
2. Block confirmations (almost impossible after 6 confirmations)
3. Proof of Work (requires 51% hash power to rewrite history)

---

## E

### ECDSA (Elliptic Curve Digital Signature Algorithm)
The digital signature scheme used by Bitcoin.

**Curve**: secp256k1

**Flow**:
```
Private key → ECDSA → Public key → Hash → Address
```

SimpleBTC uses a simplified SHA256-based signature.

---

## F

### Fee
The amount paid to miners as an incentive to include transactions in a block.

**Calculation**:
```
Fee = total inputs − total outputs
```

**Fee rate**:
```
Fee rate = fee / transaction size (sat/byte)
```

**Recommendations**:
- Low: 1–5 sat/byte
- Medium: 10–20 sat/byte
- High: 50+ sat/byte

---

### Fork
A situation where the blockchain has multiple valid branches.

**Types**:
- **Temporary fork**: two miners produce a block simultaneously; resolved by the longest-chain rule
- **Hard fork**: protocol-incompatible upgrade (e.g., BCH)
- **Soft fork**: backward-compatible upgrade (e.g., SegWit)

---

## G

### Genesis Block
The first block in the blockchain, with index 0.

**Bitcoin genesis block**:
- Date: January 3, 2009
- Reward: 50 BTC (unspendable)
- Message: "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"

**SimpleBTC**:
```rust
fn create_genesis_block() {
    // Create the block at index 0
    // previous_hash = "0"
}
```

---

## H

### Hash
A function that converts arbitrary data into a fixed-length string.

**Bitcoin uses**:
- SHA256 (transaction IDs, block hashes)
- RIPEMD160 (address generation)

**Properties**:
- Deterministic
- One-way (preimage resistant)
- Collision resistant
- Avalanche effect

**Example**:
```
SHA256("hello") = 2cf24dba5fb0a30e...
```

---

### Hash Rate
The number of hash computations performed per second.

**Units**:
- H/s (hashes per second)
- KH/s = 1,000 H/s
- MH/s = 1,000,000 H/s
- GH/s = 1,000,000,000 H/s
- TH/s = 1,000,000,000,000 H/s
- EH/s = 1,000,000,000,000,000,000 H/s

**Bitcoin network**: approximately 300+ EH/s

---

### Hot Wallet
A wallet connected to the internet, convenient for everyday use.

**Types**:
- Mobile wallets
- Desktop wallets
- Web wallets

**Advantage**: Convenient to use
**Disadvantage**: Lower security

---

## M

### Merkle Tree
A binary hash tree of transactions; the root hash is stored in the block header.

**Structure**:
```
        Root
       /    \
     H12    H34
    /  \   /  \
   H1  H2 H3  H4
```

**Uses**:
- SPV lightweight verification
- Proving a transaction exists in a block
- O(log n) verification complexity

---

### Mining
The process of creating new blocks through proof of work.

**Steps**:
1. Collect pending transactions
2. Create the Coinbase transaction
3. Compute the Merkle root
4. Adjust the nonce to find a valid hash
5. Broadcast the block

**Reward**: block reward + transaction fees

---

### Multisig (M-of-N)
An address that requires M signatures (out of N total keys) to spend funds.

**Examples**:
- 2-of-3: CEO + CFO + CTO, any two suffice
- 3-of-5: a board of 5, requiring 3 to agree

**Address**: starts with "3" (P2SH)

---

## N

### Node
A computer running Bitcoin client software.

**Types**:
- **Full node**: stores the complete blockchain, validates all transactions
- **Light node**: stores only block headers, uses SPV verification
- **Miner node**: a full node that participates in mining

---

### Nonce
A number adjusted during mining to change the block hash.

**Purpose**: proof of work
```rust
while hash(block_data + nonce) >= target {
    nonce++;  // keep trying
}
```

---

## P

### P2P (Peer-to-Peer)
A network where nodes communicate directly with each other, without a central server.

**Bitcoin network**:
- Decentralized
- Censorship-resistant
- No single point of failure

---

### P2PKH (Pay-to-Public-Key-Hash)
The traditional Bitcoin address type, starting with "1".

**Flow**:
```
Public key → SHA256 → RIPEMD160 → Base58 → Address
```

---

### P2SH (Pay-to-Script-Hash)
Pay-to-script-hash, used for advanced features like multisig; addresses start with "3".

**Advantages**:
- Supports complex scripts
- Hides script details
- Fee is borne by the recipient

---

### Private Key
A secret number used to sign transactions.

**Properties**:
- A 256-bit random number
- Owning the private key = owning the bitcoin
- Cannot be recovered if lost

**Protection**:
- Never share it
- Store it encrypted
- Keep multiple backups

---

### Proof of Work (PoW)
Bitcoin's consensus mechanism.

**Principle**: Finding a hash that satisfies the difficulty target requires a large amount of computation.

**Purpose**:
- Prevent spam attacks
- Decentralized consensus
- Extremely high cost for a 51% attack

---

### Public Key
A publicly shareable number derived from a private key, used to generate addresses and verify signatures.

**Derivation**:
```
Private key → elliptic curve operation → Public key → Hash → Address
```

---

## R

### RBF (Replace-By-Fee)
A mechanism that allows replacing an unconfirmed transaction (BIP125).

**Uses**:
- Speed up a transaction (by increasing the fee)
- Cancel a transaction
- Batch optimization

**Marker**: nSequence < 0xFFFFFFFE

---

## S

### Satoshi (sat)
The smallest unit of bitcoin.

```
1 BTC = 100,000,000 satoshi
1 sat = 0.00000001 BTC
```

**Named after**: Bitcoin's creator, Satoshi Nakamoto

---

### Script
Bitcoin's scripting language, which defines spending conditions.

**Opcodes**:
- OP_DUP
- OP_HASH160
- OP_EQUALVERIFY
- OP_CHECKSIG
- OP_CHECKMULTISIG

SimpleBTC uses a simplified version.

---

### SPV (Simplified Payment Verification)
Lightweight verification that does not require downloading the full blockchain.

**Principle**: Uses Merkle proofs to verify transactions.

**Advantages**:
- Only needs block headers (~80 bytes each)
- Suitable for mobile wallets
- O(log n) verification

---

## T

### Timelock (nLockTime)
Restricts a transaction from being confirmed before a specific time.

**Types**:
- Timestamp-based (≥ 500,000,000)
- Block height-based (< 500,000,000)

**Applications**:
- Time deposits
- Inheritance
- Payroll disbursement

---

### Transaction (TX)
The basic unit of value transfer.

**Contains**:
- Inputs (which UTXOs to spend)
- Outputs (which new UTXOs to create)
- Timestamp
- Fee

---

## U

### UTXO (Unspent Transaction Output)
An unspent transaction output, representing bitcoin that can be spent.

**Lifecycle**:
1. Created (as a transaction output)
2. Exists (in the UTXO set)
3. Spent (referenced by a transaction input)
4. Removed (deleted from the UTXO set)

**Balance**: the sum of all UTXOs

---

## W

### Wallet
Software that manages private keys, public keys, and addresses.

**Types**:
- Hot wallet (online)
- Cold wallet (offline)
- Hardware wallet
- Paper wallet

**Functions**:
- Generate key pairs
- Create addresses
- Sign transactions
- Query balances

---

## Numbers

### 51% Attack
An attack in which an attacker controls more than 50% of the network's hash power, enabling rewriting of blockchain history.

**Consequences**:
- Double-spend attacks
- Blocking transaction confirmations

**Defense**: Bitcoin's hash rate is so large that the cost of such an attack is prohibitively high.

---

### 6 Confirmations
The standard number of confirmations for a Bitcoin transaction to be considered secure.

**Time**: approximately 60 minutes (6 blocks × 10 minutes)

**Reason**: After 6 blocks, rewriting history is virtually impossible.

---

## Reference Resources

- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- [Bitcoin Wiki](https://en.bitcoin.it/wiki/Main_Page)
- [Mastering Bitcoin](https://github.com/bitcoinbook/bitcoinbook)
- [BIP List](https://github.com/bitcoin/bips)

---

[Back to Documentation Home](../introduction/README.md) | [Basic Concepts](../guide/concepts.md)
