# REST API

SimpleBTC provides a complete RESTful API, allowing interaction with the blockchain system over HTTP. This page describes all endpoints, request and response formats, and cURL call examples in detail.

---

## Basic Information

| Item | Description |
|------|-------------|
| Base URL | `http://localhost:3000` |
| Protocol | HTTP/1.1 |
| Data Format | JSON |
| CORS | All origins allowed (`*`) |
| Authentication | None (demo version) |

### Starting the Server

```bash
# Development mode
cargo run --bin server

# Release mode (better performance)
cargo run --release --bin server
```

Server output on startup:

```
  SimpleBTC Server v1.0
  =====================

  Web UI:   http://localhost:3000
  API:      http://localhost:3000/api/blockchain/info

  Genesis:  <genesis_address> (pre-funded with 100 BTC)

  Crypto:   secp256k1 ECDSA (real Bitcoin signatures)
```

---

## Common Response Format

All endpoints use a unified JSON response structure:

### Success Response

```json
{
  "success": true,
  "data": { },
  "error": null
}
```

### Error Response

```json
{
  "success": false,
  "data": null,
  "error": "Error description"
}
```

### HTTP Status Codes

| Status Code | Description |
|------------|-------------|
| 200 | Request successful |
| 400 | Parameter error or business logic failure |

---

## Endpoint Summary

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Returns embedded Web UI |
| GET | `/api/blockchain/info` | Get blockchain status information |
| GET | `/api/blockchain/chain` | Get full blockchain data |
| GET | `/api/blockchain/validate` | Validate blockchain integrity |
| POST | `/api/wallet/create` | Create a new wallet |
| GET | `/api/wallet/balance/:address` | Query address balance |
| POST | `/api/transaction/create` | Create a transfer transaction |
| POST | `/api/mine` | Mine (pack pending transactions) |

---

## Endpoint Details

### GET /

Returns the embedded Web UI page (HTML). Suitable for opening directly in a browser.

**Example**

```bash
curl http://localhost:3000/
```

---

### GET /api/blockchain/info

Gets the current blockchain state, including height, difficulty, number of pending transactions, mining reward, and genesis address.

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `height` | number | Blockchain height (total blocks included) |
| `difficulty` | number | Current mining difficulty (number of leading zeros in hash) |
| `pending_transactions` | number | Number of unconfirmed transactions in the mempool |
| `mining_reward` | number | Block mining reward (satoshi) |
| `genesis_address` | string | Genesis wallet address (pre-funded with 100 BTC) |

**Response Example**

```json
{
  "success": true,
  "data": {
    "height": 3,
    "difficulty": 4,
    "pending_transactions": 1,
    "mining_reward": 5000,
    "genesis_address": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
  },
  "error": null
}
```

**cURL Example**

```bash
curl http://localhost:3000/api/blockchain/info
```

---

### GET /api/blockchain/chain

Gets the full blockchain data, including all fields and transaction lists for each block.

**Block Fields**

| Field | Type | Description |
|-------|------|-------------|
| `index` | number | Block index (starting from 0) |
| `timestamp` | number | Block timestamp (milliseconds) |
| `transactions` | array | Transaction list |
| `previous_hash` | string | Previous block hash |
| `hash` | string | This block's hash |
| `nonce` | number | Proof-of-work nonce |
| `merkle_root` | string | Merkle root of transactions |
| `difficulty` | number | Block difficulty |

**Transaction Fields**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Transaction ID (hash) |
| `inputs` | array | Transaction input list (UTXO references) |
| `outputs` | array | Transaction output list (recipients and amounts) |
| `timestamp` | number | Transaction timestamp |

**Response Example**

```json
{
  "success": true,
  "data": [
    {
      "index": 0,
      "timestamp": 1700000000000,
      "transactions": [],
      "previous_hash": "0",
      "hash": "0000abcdef...",
      "nonce": 12345,
      "merkle_root": "abc123...",
      "difficulty": 4
    },
    {
      "index": 1,
      "timestamp": 1700000060000,
      "transactions": [
        {
          "id": "txabc123...",
          "inputs": [],
          "outputs": [
            { "address": "a1b2c3...", "amount": 5000 }
          ],
          "timestamp": 1700000059000
        }
      ],
      "previous_hash": "0000abcdef...",
      "hash": "0000fedcba...",
      "nonce": 67890,
      "merkle_root": "def456...",
      "difficulty": 4
    }
  ],
  "error": null
}
```

**cURL Example**

```bash
# Get full blockchain (with jq for formatted output)
curl http://localhost:3000/api/blockchain/chain | jq
```

---

### GET /api/blockchain/validate

Validates the integrity of the blockchain, checking whether the hash chain and proof of work for all blocks are valid.

**Response Example (Validation Passed)**

```json
{
  "success": true,
  "data": "Blockchain is valid",
  "error": null
}
```

**Response Example (Validation Failed)**

```json
{
  "success": false,
  "data": "Blockchain is invalid",
  "error": null
}
```

**cURL Example**

```bash
curl http://localhost:3000/api/blockchain/validate
```

---

### POST /api/wallet/create

Generates a new secp256k1 key pair on the server side, returning the wallet address and public key. The private key is saved in server memory for subsequent ECDSA signing of transactions.

> **Note**: The private key is not returned via the API. The wallet address created this way can be directly used to receive transfers and initiate transactions.

**Request Body**

None required.

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | Wallet address (40-character hex) |
| `public_key` | string | Compressed public key (secp256k1) |

**Response Example**

```json
{
  "success": true,
  "data": {
    "address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a",
    "public_key": "04d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c8f1a3d6e9b4c7f0a3d6e9b4c7f0"
  },
  "error": null
}
```

**cURL Example**

```bash
curl -X POST http://localhost:3000/api/wallet/create
```

---

### GET /api/wallet/balance/:address

Queries the current balance of a specified wallet address; balance is computed from the UTXO set.

**Path Parameters**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | string | Yes | Wallet address (40-character hex) |

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | The queried wallet address |
| `balance` | number | Balance (satoshi; 1 BTC = 100,000,000 satoshi) |

**Response Example**

```json
{
  "success": true,
  "data": {
    "address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a",
    "balance": 10000000000
  },
  "error": null
}
```

**cURL Example**

```bash
ADDRESS="3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a"
curl http://localhost:3000/api/wallet/balance/$ADDRESS
```

---

### POST /api/transaction/create

Creates a transfer transaction, signs it with the sender's private key using secp256k1 ECDSA, and adds it to the mempool to await mining confirmation.

> **Prerequisite**: The sender address must be a wallet created via `/api/wallet/create` (the server must hold its private key to sign).

**Request Body**

```json
{
  "from_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a",
  "to_address":   "7c1e5b9f4a2d8e3c6f0b5a9d2e7f4c1b8a3d6e9f",
  "amount": 5000,
  "fee": 10
}
```

**Request Parameters**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `from_address` | string | Yes | Sender wallet address |
| `to_address` | string | Yes | Recipient wallet address |
| `amount` | number | Yes | Transfer amount (satoshi) |
| `fee` | number | Yes | Transaction fee (satoshi, goes to the miner) |

**Success Response**

```json
{
  "success": true,
  "data": "Transaction created: txabc123def456...",
  "error": null
}
```

**Error Response Example**

```json
{
  "success": false,
  "data": null,
  "error": "Wallet not found: 3f8a.... Please first create a wallet via /api/wallet/create."
}
```

**Common Errors**

| Error | Cause | Resolution |
|-------|-------|-----------|
| Wallet not found | Sender address not created on this server | Call `/api/wallet/create` first |
| Insufficient balance | Balance < amount + fee | Reduce amount, or mine to receive a reward first |

**cURL Example**

```bash
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{
    "from_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a",
    "to_address":   "7c1e5b9f4a2d8e3c6f0b5a9d2e7f4c1b8a3d6e9f",
    "amount": 5000,
    "fee": 10
  }'
```

---

### POST /api/mine

Performs proof-of-work mining, packing all pending transactions in the mempool into a new block, and distributing the block reward to the miner address.

> **Note**: Mining is a CPU-intensive operation that may take several seconds depending on the current difficulty. The mining reward (`mining_reward`) and all transaction fees go to the miner address; the balance change is visible after the next query.

**Request Body**

```json
{
  "miner_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a"
}
```

**Request Parameters**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `miner_address` | string | Yes | Miner wallet address (receives reward) |

**Success Response**

```json
{
  "success": true,
  "data": "Block mined! Height: 4",
  "error": null
}
```

**Error Response Example**

```json
{
  "success": false,
  "data": null,
  "error": "No pending transactions"
}
```

**cURL Example**

```bash
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a"}'
```

---

## Quick Start: Complete Workflow

The following example shows the complete flow from creating a wallet to completing a transfer.

### Step 1: Get the Genesis Address

The genesis wallet is pre-funded with 100 BTC (10,000,000,000 satoshi); its address can be obtained from `blockchain/info`.

```bash
# Get genesis address
curl -s http://localhost:3000/api/blockchain/info | jq '.data.genesis_address'
# Example output: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"

GENESIS="a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
```

### Step 2: Create a New Wallet

```bash
# Create wallet, save address
WALLET=$(curl -s -X POST http://localhost:3000/api/wallet/create)
echo $WALLET | jq

MY_ADDR=$(echo $WALLET | jq -r '.data.address')
echo "My address: $MY_ADDR"
```

### Step 3: Transfer from Genesis Address to New Wallet

```bash
# Genesis address transfers 10,000 satoshi to new wallet
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d "{
    \"from_address\": \"$GENESIS\",
    \"to_address\":   \"$MY_ADDR\",
    \"amount\": 10000,
    \"fee\": 100
  }"
```

### Step 4: Mine to Confirm Transaction

```bash
# Use new wallet as miner address (also receives mining reward)
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d "{\"miner_address\": \"$MY_ADDR\"}"
```

### Step 5: Query Balance

```bash
# Query new wallet balance (should include transfer amount + mining reward)
curl http://localhost:3000/api/wallet/balance/$MY_ADDR | jq
```

### Step 6: Validate the Blockchain

```bash
# Confirm blockchain data is complete
curl http://localhost:3000/api/blockchain/validate | jq
```

### Complete Script

```bash
#!/bin/bash
BASE="http://localhost:3000"

echo "=== SimpleBTC Quick Demo ==="

# 1. Get genesis address
GENESIS=$(curl -s $BASE/api/blockchain/info | jq -r '.data.genesis_address')
echo "Genesis address: $GENESIS"

# 2. Create new wallet
MY_ADDR=$(curl -s -X POST $BASE/api/wallet/create | jq -r '.data.address')
echo "New wallet address: $MY_ADDR"

# 3. Create transaction (genesis address → new wallet, transfer 10000 satoshi)
TX=$(curl -s -X POST $BASE/api/transaction/create \
  -H "Content-Type: application/json" \
  -d "{\"from_address\":\"$GENESIS\",\"to_address\":\"$MY_ADDR\",\"amount\":10000,\"fee\":100}")
echo "Transaction: $(echo $TX | jq -r '.data')"

# 4. Mine to confirm
MINE=$(curl -s -X POST $BASE/api/mine \
  -H "Content-Type: application/json" \
  -d "{\"miner_address\":\"$MY_ADDR\"}")
echo "Mining: $(echo $MINE | jq -r '.data')"

# 5. Query balance
BAL=$(curl -s $BASE/api/wallet/balance/$MY_ADDR | jq '.data.balance')
echo "Balance: $BAL satoshi"

# 6. Validate blockchain
VALID=$(curl -s $BASE/api/blockchain/validate | jq -r '.data')
echo "Validation: $VALID"
```

---

## Unit Notes

All amount fields in SimpleBTC use **satoshi** as the unit (consistent with Bitcoin):

| Unit | Conversion |
|------|-----------|
| 1 BTC | 100,000,000 satoshi |
| 1 mBTC | 100,000 satoshi |
| 1 satoshi | Minimum unit, indivisible |

The genesis wallet's pre-funded balance is `10,000,000,000 satoshi` (100 BTC). The default mining reward is `5000 satoshi`.
