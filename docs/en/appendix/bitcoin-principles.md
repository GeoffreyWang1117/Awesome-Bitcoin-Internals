# Bitcoin Principles

This appendix is a standalone educational article for readers who want to understand the underlying principles of Bitcoin. No programming background is required, though some familiarity with cryptography and distributed systems will be helpful.

---

## Introduction

On October 31, 2008, a person using the pseudonym "Satoshi Nakamoto" published a 9-page paper on a cryptography mailing list: [*Bitcoin: A Peer-to-Peer Electronic Cash System*](https://bitcoin.org/bitcoin.pdf). The paper proposed a revolutionary answer to the question: **How can two strangers transfer value without a trusted third party (such as a bank)?**

Bitcoin's answer rests on four core technical pillars: hash functions, public-key cryptography, proof of work, and the blockchain data structure. This chapter introduces each of these principles in turn.

---

## I. Hash Functions and SHA-256

### What Is a Hash Function?

A hash function is a mathematical function that maps data of arbitrary length to a fixed-length "digest." Bitcoin uses **SHA-256** (Secure Hash Algorithm 256-bit), which always produces a 256-bit (32-byte) output, typically represented as 64 hexadecimal characters.

```
SHA-256("Hello, Bitcoin!") =
  a3b5c7d2e1f0... (64 hexadecimal characters)

SHA-256("Hello, Bitcoin.")  =
  f9e8d7c6b5a4... (completely different hash)
```

### Four Key Properties of Hash Functions

**1. Deterministic**
The same input always produces the same output. There is no randomness.

**2. Avalanche Effect**
A tiny change in the input (even a single bit) causes a large, unpredictable change in the output. This ensures the hash is highly sensitive to even minor modifications.

**3. One-Way (Preimage Resistance)**
It is computationally infeasible to reverse-engineer the original input from a hash output. Even knowing the SHA-256 output, recovering the input by brute force within the lifetime of the universe is impossible (the search space is 2²⁵⁶).

**4. Collision Resistance**
It is computationally infeasible to find two different inputs that produce the same output (a hash collision). This is the foundation of the Bitcoin blockchain's tamper resistance.

### SHA-256 Applications in Bitcoin

Bitcoin uses SHA-256 (or double SHA-256, i.e., SHA-256(SHA-256(data))) in several places:

| Application | Hash Method | Purpose |
|-------------|------------|---------|
| Block hash | SHA-256(SHA-256(block header)) | Unique block identifier, links the blockchain |
| Transaction ID | SHA-256(SHA-256(transaction data)) | Unique transaction identifier |
| Address generation | RIPEMD-160(SHA-256(public key)) | Derives a Bitcoin address from a public key |
| Merkle tree | SHA-256(SHA-256(node concatenation)) | Efficient verification of a transaction set |
| Mining | SHA-256(SHA-256(block header)) | Finds a nonce satisfying the difficulty target |

### Why Double SHA-256?

Bitcoin uses SHA-256 twice rather than once, primarily to defend against **length extension attacks**. SHA-256's mathematical structure has a weakness: knowing `SHA-256(M)`, one can compute `SHA-256(M || X)` (for arbitrary appended data X) without knowing M. Double SHA-256 eliminates this security concern.

---

## II. Public-Key Cryptography

### Symmetric vs. Asymmetric Encryption

Traditional **symmetric encryption** (such as AES) uses the same key for both encryption and decryption. The problem is that if Alice wants to send an encrypted message to Bob, she first needs to securely transmit the key to Bob — but if a secure channel already exists for that, why not use it directly for the message?

**Asymmetric encryption** (public-key cryptography) solves this "key distribution problem." Each user has two keys:
- **Public Key**: can be shared openly with anyone
- **Private Key**: must be kept strictly secret and never shared

Their relationship is: a public key can be derived from a private key, but a private key cannot be reverse-engineered from a public key.

```
Private key (a random 256-bit number)
    │
    ▼ (one-way, irreversible)
Public key (a point on an elliptic curve)
    │
    ▼ (one-way, irreversible)
Bitcoin address (hash of the public key)
```

### Elliptic Curve Cryptography (ECC) and secp256k1

Bitcoin uses the **Elliptic Curve Digital Signature Algorithm (ECDSA)** with a specific curve called `secp256k1`. This curve is defined by the equation:

```
y² = x³ + 7  (over the finite field Fp, where p = 2²⁵⁶ − 2³² − 977)
```

The security of elliptic curve cryptography is based on the **Elliptic Curve Discrete Logarithm Problem (ECDLP)**: given a generator point G on the curve and a point P = k·G, it is computationally infeasible to recover the integer k from P.

**Why secp256k1 rather than the more common secp256r1 (NIST P-256)?**

Satoshi Nakamoto chose secp256k1 parameters that were derived from a deterministic formula rather than generated randomly, making it less likely to contain a backdoor inserted by the NSA — a concern that is debated but worth considering in the cryptography community. The coefficients of secp256k1 are very simple (a=0, b=7), with no complex "seemingly random" parameters, providing greater transparency.

**Key generation process**:

```
1. Generate a private key: randomly select a 256-bit integer k (1 ≤ k ≤ n−1, where n is the curve order)
   Private key = random number k (typically from a cryptographically secure random number generator)

2. Generate a public key: compute elliptic curve point multiplication
   Public key = k × G (G is the standard base point of secp256k1)
   Note: point multiplication is a special operation defined on the elliptic curve, not ordinary multiplication

3. Generate an address (simplified):
   Address = RIPEMD-160(SHA-256(public key)) + checksum
```

The randomness of the private key is critical. Documented theft cases show that private keys generated using weak random number generators (such as timestamps) have been brute-forced. Truly secure private keys come from the operating system's cryptographic random number interface (e.g., `/dev/urandom` on Linux).

---

## III. Digital Signatures

### The Role of Signatures

Digital signatures solve the most fundamental problem in Bitcoin: **How do you prove you have the right to spend some funds without revealing your private key?**

By analogy with the real world: you sign a cheque and the bank verifies the signature is yours. A digital signature is the cryptographic equivalent of this process, but more secure — it does not rely on the visual appearance of the signature (which can be forged), but on a cryptographic proof that is mathematically impossible to forge.

### The ECDSA Signing Process

**Signing**:
```
Input: message M (hash of transaction data), private key k
Output: signature (r, s)

Steps:
1. Generate a random number r_rand (must be different for each signature!)
2. Compute the curve point R = r_rand × G
3. r = R.x mod n (take the x-coordinate of R)
4. s = r_rand⁻¹ × (hash(M) + k × r) mod n
```

**Verification**:
```
Input: message M, signature (r, s), public key P = k × G
Output: valid / invalid

Steps:
1. u1 = hash(M) × s⁻¹ mod n
2. u2 = r × s⁻¹ mod n
3. Compute point Q = u1 × G + u2 × P
4. Verify Q.x mod n == r
```

The verification process uses only the **public key** and does not require the private key. This means anyone can verify a signature, but only the holder of the private key can create a valid signature.

### Critical Security Requirement: The Random Number Must Not Be Reused

The random number `r_rand` in the signing algorithm **must be unique and unpredictable for each signature**. In 2013, the ECDSA implementation in the PlayStation 3 was broken because it used a fixed random number, leading to private key exposure. Similar cases have occurred in Bitcoin history.

Modern implementations (including Bitcoin Core) use **RFC 6979**, which deterministically generates the random number from the private key and message, completely eliminating the risk of random number reuse.

---

## IV. Proof-of-Work Consensus

### The Byzantine Generals Problem

In a distributed system, how can consensus be reached if nodes may send incorrect information (whether due to malice or failure)? This is known as the **Byzantine Generals Problem**, formally introduced by Lamport, Shostak, and Pease in 1982.

The classical conclusion: in a traditional message-passing model, if there are f Byzantine nodes, at least 3f+1 total nodes are required to tolerate the faults. However, this result has a prerequisite: communication costs are negligible.

Satoshi Nakamoto's insight was that in Bitcoin, speaking on the network requires a **real physical cost** (electricity), which fundamentally changes the game-theoretic equilibrium.

### Proof of Work

Proof of Work requires miners to find a special number (nonce) such that the hash of the block header satisfies a specific condition (beginning with a certain number of zeros):

```
Target: SHA-256(SHA-256(block header)) < target value

Equivalently: the block header hash begins with `difficulty` leading zeros

Example (difficulty=4):
0000a3f7d2e1b5c8...  ← valid (starts with 4 zeros)
0001a3f7d2e1b5c8...  ← invalid (4th character is not 0)
```

Miners repeatedly modify the nonce and recompute the hash until a valid value is found:

```
while SHA-256(SHA-256(block header || nonce)) >= target:
    nonce += 1  // try the next number

Once found, broadcast this block to the entire network
```

**Difficulty adjustment**: Bitcoin automatically adjusts the difficulty every 2016 blocks (approximately two weeks), targeting an average block time of 10 minutes. If hash power increases, the difficulty rises; if hash power decreases, the difficulty falls.

### Why Does PoW Prevent Double Spending?

Suppose an attacker attempts to double-spend:
1. Sends transaction A to a merchant (paying 10 BTC)
2. The merchant waits for N block confirmations before shipping
3. The attacker secretly mines on a separate chain, creating blocks containing transaction B (sending the 10 BTC back to themselves)

The attacker needs to secretly outpace the honest miners who have already mined N blocks, producing a longer chain. If the attacker controls a fraction α of the hash power (α < 0.5), their probability of success decreases exponentially as N increases. Satoshi Nakamoto proved in the whitepaper that:

```
P(success) ≈ (α / (1−α))^N
```

When α = 0.3 (30% hash power) and N = 6 (6 confirmations, approximately 1 hour):
```
P(success) ≈ (0.3/0.7)^6 = (0.4286)^6 ≈ 0.0006 = 0.06%
```

This is the mathematical basis for Bitcoin's "6 confirmations" rule.

### PoW vs. Other Consensus Mechanisms

| Consensus Mechanism | Representative Projects | Advantages | Disadvantages |
|--------------------|-----------------------|-----------|--------------|
| Proof of Work (PoW) | Bitcoin, Litecoin | No need to trust participants, Sybil attack resistant | High energy consumption, slow block time |
| Proof of Stake (PoS) | Ethereum 2.0, Cardano | Low energy consumption, good scalability | Initial distribution fairness issues, "nothing-at-stake" problem |
| Delegated Proof of Stake (DPoS) | EOS, Tron | High throughput | Centralization risk, 21 nodes |
| Practical Byzantine Fault Tolerance (PBFT) | Hyperledger | Efficient, has finality | Suitable only for consortium chains with known participants |

---

## V. Blockchain Data Structure

### Block Structure

Each block consists of two parts:

**Block header (80 bytes)**:
```
Version      (4 bytes)  - protocol version
Previous hash(32 bytes) - links this block to the previous one
Merkle root  (32 bytes) - root hash of the Merkle tree of all transactions
Timestamp    (4 bytes)  - Unix timestamp
Difficulty   (4 bytes)  - current mining difficulty (compact format)
Nonce        (4 bytes)  - value adjusted by the miner
```

**Block body (variable size)**:
```
Transaction count (varint)
Transaction list  [Transaction...]
  ├── Transaction 1 (coinbase, miner reward)
  ├── Transaction 2
  └── ...
```

### Chain Structure and Tamper Resistance

The "chain" in blockchain comes from **each block header containing the hash of the previous block**:

```
Block 0 (genesis)              Block 1                     Block 2
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ prev_hash: 0000  │    │ prev_hash: H(B0) │    │ prev_hash: H(B1) │
│ merkle_root: ... │◄───│ merkle_root: ... │◄───│ merkle_root: ... │
│ nonce: 2083236893│    │ nonce: 12345678  │    │ nonce: 87654321  │
│ hash: H(B0)      │    │ hash: H(B1)      │    │ hash: H(B2)      │
└──────────────────┘    └──────────────────┘    └──────────────────┘
```

If an attacker modifies a transaction in Block 1:
1. Block 1's Merkle root changes
2. Block 1's block header hash changes
3. Block 2's `prev_hash` field no longer matches Block 1's new hash
4. The attacker must redo the proof of work to compute Block 2's nonce
5. This invalidates Block 3, requiring it to be mined again...
6. The attacker must recompute the PoW for every block from the tampered one to the chain tip

Because honest miners are continuously extending the chain, the attacker would need to complete this work faster than the entire network's hash power. Under the assumption of 51% honest hash power, this is an impossible task.

### The UTXO Set: The "State" of the Blockchain

The full blockchain records all historical transactions, but validating new transactions only requires knowing "which outputs are currently unspent" — the UTXO set (Unspent Transaction Output Set).

The UTXO set is the "ledger state" obtained by sequentially processing all transactions starting from the genesis block. As of 2024, the Bitcoin UTXO set contains approximately 110 million entries, occupying about 5–6 GB of memory, far smaller than the full blockchain at 600+ GB.

```
Blockchain (historical record, ~600 GB)
    ↓ Full node processes sequentially
UTXO set (current state, ~5 GB)
    ↓ Query
Validate whether new transactions are valid
```

---

## VI. Decentralization and Network Security

### P2P Network

Bitcoin nodes connect to each other through a **peer-to-peer (P2P) network**, with no central server. Each node establishes connections to dozens of peers, forming a small-world network.

New transactions propagate through the network via a **Gossip protocol**:
1. Node A creates a transaction and broadcasts it to connected nodes
2. Each node that receives the transaction validates it, then forwards it to its own connected nodes
3. The transaction spreads to most nodes worldwide within seconds

### Economic Analysis of a 51% Attack

Attacking the Bitcoin network requires controlling more than 50% of the global hash rate. As of 2024, Bitcoin's global hash rate is approximately 600 EH/s. Purchasing or renting 50% of that hash power would require billions of dollars in hardware investment, plus ongoing electricity costs.

More critically, there is the **economic incentive problem of the attack**:
- Potential gain from a successful attack: double-spending in a large transaction, possibly defrauding an exchange
- Cost of the attack: Bitcoin's price collapses, and the attacker's bitcoin holdings and mining equipment drop sharply in value
- Conclusion: a rational economic actor would prefer to use their hash power to mine honestly (approximately $20 million per day in revenue) rather than launch an attack with limited upside and extreme risk

This **economically self-reinforcing security** is the elegance of Bitcoin's design — security automatically strengthens as the network's value grows.

### Node Types and Network Roles

| Node Type | Description | Typical Use Case |
|-----------|-------------|-----------------|
| Full node | Stores the full blockchain, independently validates all rules | Exchanges, node operators |
| Pruned node | Retains only recent blocks, saves disk space | Home users |
| SPV node | Downloads only block headers, lightweight verification | Mobile wallets |
| Mining pool node | Coordinates large numbers of miners, allocates hash power | Mining farm operators |
| Lightning Network node | Manages payment channels, enables instant micropayments | Merchants, everyday payments |

---

## Further Reading and References

The following are important references for a deeper understanding of Bitcoin's technical principles:

### Foundational Papers

1. **Nakamoto, S. (2008).** *Bitcoin: A Peer-to-Peer Electronic Cash System.*
   [https://bitcoin.org/bitcoin.pdf](https://bitcoin.org/bitcoin.pdf)
   The Bitcoin whitepaper, 9 pages, covering all core concepts. Essential reading.

2. **Merkle, R. C. (1979).** *Secrecy, Authentication, and Public Key Systems.*
   Stanford University doctoral dissertation, the original paper on Merkle trees.

3. **Lamport, L., Shostak, R., & Pease, M. (1982).** *The Byzantine Generals Problem.*
   ACM Transactions on Programming Languages and Systems.
   The classical formal description of the distributed consensus problem.

4. **Back, A. (2002).** *Hashcash – A Denial of Service Counter-Measure.*
   [http://www.hashcash.org/papers/hashcash.pdf](http://www.hashcash.org/papers/hashcash.pdf)
   The predecessor to Bitcoin's PoW mechanism, originally designed to prevent email spam.

5. **Dai, W. (1998).** *b-money.*
   [http://www.weidai.com/bmoney.txt](http://www.weidai.com/bmoney.txt)
   An early decentralized digital currency proposal cited by Satoshi Nakamoto.

### Cryptography Fundamentals

6. **Johnson, D., Menezes, A., & Vanstone, S. (2001).** *The Elliptic Curve Digital Signature Algorithm (ECDSA).*
   International Journal of Information Security.
   The authoritative technical specification for ECDSA.

7. **Pornin, T. (2013).** *RFC 6979: Deterministic Usage of the Digital Signature Algorithm (DSA) and Elliptic Curve Digital Signature Algorithm (ECDSA).*
   IETF Request for Comments.
   The standard for deterministic signature nonce generation, eliminating the nonce reuse vulnerability.

8. **National Institute of Standards and Technology. (2015).** *FIPS PUB 180-4: Secure Hash Standard.*
   The official specification document for SHA-256.

### Books

9. **Antonopoulos, A. M. (2017).** *Mastering Bitcoin: Programming the Open Blockchain (2nd ed.).* O'Reilly Media.
   The most comprehensive introductory book on Bitcoin technology; the [open-source version](https://github.com/bitcoinbook/bitcoinbook) is freely available.

10. **Song, J. (2019).** *Programming Bitcoin.* O'Reilly Media.
    Implements the Bitcoin protocol from scratch in Python; ideal for hands-on learning.

11. **Narayanan, A., Bonneau, J., Felten, E., Miller, A., & Goldfeder, S. (2016).**
    *Bitcoin and Cryptocurrency Technologies.* Princeton University Press.
    [Free PDF available](https://bitcoinbook.cs.princeton.edu/), a comprehensive academic analysis.

### Advanced Resources

12. **Bitcoin Improvement Proposals (BIPs).**
    [https://github.com/bitcoin/bips](https://github.com/bitcoin/bips)
    All improvement proposals for the Bitcoin protocol, including SegWit (BIP141), RBF (BIP125), HD Wallets (BIP32), and more.

13. **Bitcoin Core source code.**
    [https://github.com/bitcoin/bitcoin](https://github.com/bitcoin/bitcoin)
    The reference implementation of Bitcoin, written in C++, approximately 150,000 lines of code.

---

## Summary: Bitcoin's Five-Layer Architecture

```
Layer 5: Economic Incentive Layer
         PoW rewards (block reward + fees) → drives honest miner behavior
              ↑
Layer 4: Consensus Layer
         Longest-chain rule + PoW difficulty adjustment → network-wide agreement on "which chain is correct"
              ↑
Layer 3: Data Structure Layer
         Blockchain (chained blocks) + Merkle tree → tamper-resistant transaction history
              ↑
Layer 2: Transaction Layer
         UTXO model + Script system → defines the rules for value transfer
              ↑
Layer 1: Cryptography Layer
         SHA-256 (integrity) + ECDSA (authentication) → trustless mathematical guarantees
```

Bitcoin's revolution does not lie in any single technical innovation — SHA-256, elliptic curve cryptography, P2P networks, hash chains, and PoW all existed before. Satoshi Nakamoto's genius was in **combining these known technologies in a specific way** to create a self-consistent, game-theoretically stable decentralized monetary system.

The SimpleBTC project implements a teaching version of this system, helping you understand how each layer works through runnable code. Reading this article alongside the source code is recommended, combining theory with practice.
