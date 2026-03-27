# REST API

SimpleBTC 提供完整的 RESTful API，允许通过 HTTP 与区块链系统进行交互。本页面详细描述所有端点、请求与响应格式，以及 cURL 调用示例。

---

## 基本信息

| 项目 | 说明 |
|------|------|
| 基础 URL | `http://localhost:3000` |
| 协议 | HTTP/1.1 |
| 数据格式 | JSON |
| CORS | 允许所有来源（`*`） |
| 认证 | 无（演示版本） |

### 启动服务器

```bash
# 开发模式
cargo run --bin server

# Release 模式（性能更好）
cargo run --release --bin server
```

服务器启动后输出：

```
  SimpleBTC Server v1.0
  =====================

  Web UI:   http://localhost:3000
  API:      http://localhost:3000/api/blockchain/info

  Genesis:  <genesis_address> (pre-funded with 100 BTC)

  Crypto:   secp256k1 ECDSA (real Bitcoin signatures)
```

---

## 通用响应格式

所有端点均使用统一的 JSON 响应结构：

### 成功响应

```json
{
  "success": true,
  "data": { },
  "error": null
}
```

### 错误响应

```json
{
  "success": false,
  "data": null,
  "error": "错误描述信息"
}
```

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 请求成功 |
| 400 | 参数错误或业务逻辑失败 |

---

## 端点一览

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 返回内嵌 Web UI |
| GET | `/api/blockchain/info` | 获取区块链状态信息 |
| GET | `/api/blockchain/chain` | 获取完整区块链数据 |
| GET | `/api/blockchain/validate` | 验证区块链完整性 |
| POST | `/api/wallet/create` | 创建新钱包 |
| GET | `/api/wallet/balance/:address` | 查询地址余额 |
| POST | `/api/transaction/create` | 创建转账交易 |
| POST | `/api/mine` | 挖矿（打包待确认交易） |

---

## 端点详解

### GET /

返回内嵌的 Web UI 页面（HTML）。适合在浏览器中直接打开。

**示例**

```bash
curl http://localhost:3000/
```

---

### GET /api/blockchain/info

获取区块链当前状态，包括高度、难度、待确认交易数量、挖矿奖励和创世地址。

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `height` | number | 区块链高度（已包含区块总数） |
| `difficulty` | number | 当前挖矿难度（哈希前导零个数） |
| `pending_transactions` | number | 内存池中待确认交易数 |
| `mining_reward` | number | 挖矿区块奖励（satoshi） |
| `genesis_address` | string | 创世钱包地址（预置 100 BTC） |

**响应示例**

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

**cURL 示例**

```bash
curl http://localhost:3000/api/blockchain/info
```

---

### GET /api/blockchain/chain

获取完整的区块链数据，包含每个区块的所有字段和交易列表。

**区块字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `index` | number | 区块索引（从 0 开始） |
| `timestamp` | number | 区块时间戳（毫秒） |
| `transactions` | array | 交易列表 |
| `previous_hash` | string | 前一区块哈希 |
| `hash` | string | 本区块哈希 |
| `nonce` | number | 工作量证明随机数 |
| `merkle_root` | string | 交易的 Merkle 根 |
| `difficulty` | number | 区块难度 |

**交易字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 交易 ID（哈希） |
| `inputs` | array | 交易输入列表（UTXO 引用） |
| `outputs` | array | 交易输出列表（接收方与金额） |
| `timestamp` | number | 交易时间戳 |

**响应示例**

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

**cURL 示例**

```bash
# 获取完整区块链（配合 jq 格式化输出）
curl http://localhost:3000/api/blockchain/chain | jq
```

---

### GET /api/blockchain/validate

验证区块链的完整性，检查所有区块的哈希链与工作量证明是否有效。

**响应示例（验证通过）**

```json
{
  "success": true,
  "data": "Blockchain is valid",
  "error": null
}
```

**响应示例（验证失败）**

```json
{
  "success": false,
  "data": "Blockchain is invalid",
  "error": null
}
```

**cURL 示例**

```bash
curl http://localhost:3000/api/blockchain/validate
```

---

### POST /api/wallet/create

在服务器端生成新的 secp256k1 密钥对，返回钱包地址和公钥。私钥保存在服务器内存中，用于后续对交易进行 ECDSA 签名。

> **注意**：私钥不会通过 API 返回。创建后的钱包地址可直接用于接收转账和发起交易。

**请求体**

无需请求体。

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `address` | string | 钱包地址（40 字符十六进制） |
| `public_key` | string | 压缩公钥（secp256k1） |

**响应示例**

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

**cURL 示例**

```bash
curl -X POST http://localhost:3000/api/wallet/create
```

---

### GET /api/wallet/balance/:address

查询指定钱包地址的当前余额，余额由 UTXO 集合计算得出。

**路径参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `address` | string | 是 | 钱包地址（40 字符十六进制） |

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `address` | string | 查询的钱包地址 |
| `balance` | number | 余额（satoshi，1 BTC = 100,000,000 satoshi） |

**响应示例**

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

**cURL 示例**

```bash
ADDRESS="3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a"
curl http://localhost:3000/api/wallet/balance/$ADDRESS
```

---

### POST /api/transaction/create

创建一笔转账交易，使用发送方的私钥进行 secp256k1 ECDSA 签名，然后加入内存池等待挖矿确认。

> **前提**：发送方地址必须是通过 `/api/wallet/create` 创建的钱包（服务器持有其私钥才能签名）。

**请求体**

```json
{
  "from_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a",
  "to_address":   "7c1e5b9f4a2d8e3c6f0b5a9d2e7f4c1b8a3d6e9f",
  "amount": 5000,
  "fee": 10
}
```

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `from_address` | string | 是 | 发送方钱包地址 |
| `to_address` | string | 是 | 接收方钱包地址 |
| `amount` | number | 是 | 转账金额（satoshi） |
| `fee` | number | 是 | 交易手续费（satoshi，归矿工所有） |

**成功响应**

```json
{
  "success": true,
  "data": "Transaction created: txabc123def456...",
  "error": null
}
```

**错误响应示例**

```json
{
  "success": false,
  "data": null,
  "error": "钱包未找到: 3f8a...。请先通过 /api/wallet/create 创建钱包。"
}
```

**常见错误**

| 错误 | 原因 | 解决方法 |
|------|------|---------|
| 钱包未找到 | 发送方地址未在此服务器创建 | 先调用 `/api/wallet/create` |
| 余额不足 | 余额 < amount + fee | 减少金额，或先挖矿获得奖励 |

**cURL 示例**

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

执行工作量证明挖矿，将内存池中所有待确认交易打包进新区块，并向矿工地址发放区块奖励。

> **注意**：挖矿是 CPU 密集操作，根据当前难度可能耗时数秒。挖矿奖励（mining_reward）和所有交易手续费均归矿工地址所有，下一次挖矿后可查询到余额变化。

**请求体**

```json
{
  "miner_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a"
}
```

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `miner_address` | string | 是 | 矿工钱包地址（接收奖励） |

**成功响应**

```json
{
  "success": true,
  "data": "Block mined! Height: 4",
  "error": null
}
```

**错误响应示例**

```json
{
  "success": false,
  "data": null,
  "error": "No pending transactions"
}
```

**cURL 示例**

```bash
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address": "3f8a2d1c9e4b7f0a5c8d2e6f1b4a9c3d7e5f2b8a"}'
```

---

## 快速入门：完整工作流

以下示例展示从创建钱包到完成转账的完整流程。

### 第一步：查看创世地址

创世钱包预置了 100 BTC（10,000,000,000 satoshi），可从 `blockchain/info` 中获取其地址。

```bash
# 获取创世地址
curl -s http://localhost:3000/api/blockchain/info | jq '.data.genesis_address'
# 示例输出: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"

GENESIS="a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
```

### 第二步：创建新钱包

```bash
# 创建钱包，保存地址
WALLET=$(curl -s -X POST http://localhost:3000/api/wallet/create)
echo $WALLET | jq

MY_ADDR=$(echo $WALLET | jq -r '.data.address')
echo "我的地址: $MY_ADDR"
```

### 第三步：从创世地址转账给新钱包

```bash
# 创世地址向新钱包转账 10,000 satoshi
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d "{
    \"from_address\": \"$GENESIS\",
    \"to_address\":   \"$MY_ADDR\",
    \"amount\": 10000,
    \"fee\": 100
  }"
```

### 第四步：挖矿以确认交易

```bash
# 使用新钱包作为矿工地址（同时获得挖矿奖励）
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d "{\"miner_address\": \"$MY_ADDR\"}"
```

### 第五步：查询余额

```bash
# 查询新钱包余额（应包含转账金额 + 挖矿奖励）
curl http://localhost:3000/api/wallet/balance/$MY_ADDR | jq
```

### 第六步：验证区块链

```bash
# 确认区块链数据完整
curl http://localhost:3000/api/blockchain/validate | jq
```

### 完整脚本

```bash
#!/bin/bash
BASE="http://localhost:3000"

echo "=== SimpleBTC 快速体验 ==="

# 1. 获取创世地址
GENESIS=$(curl -s $BASE/api/blockchain/info | jq -r '.data.genesis_address')
echo "创世地址: $GENESIS"

# 2. 创建新钱包
MY_ADDR=$(curl -s -X POST $BASE/api/wallet/create | jq -r '.data.address')
echo "新钱包地址: $MY_ADDR"

# 3. 创建交易（创世地址 → 新钱包，转账 10000 satoshi）
TX=$(curl -s -X POST $BASE/api/transaction/create \
  -H "Content-Type: application/json" \
  -d "{\"from_address\":\"$GENESIS\",\"to_address\":\"$MY_ADDR\",\"amount\":10000,\"fee\":100}")
echo "交易: $(echo $TX | jq -r '.data')"

# 4. 挖矿确认
MINE=$(curl -s -X POST $BASE/api/mine \
  -H "Content-Type: application/json" \
  -d "{\"miner_address\":\"$MY_ADDR\"}")
echo "挖矿: $(echo $MINE | jq -r '.data')"

# 5. 查询余额
BAL=$(curl -s $BASE/api/wallet/balance/$MY_ADDR | jq '.data.balance')
echo "余额: $BAL satoshi"

# 6. 验证区块链
VALID=$(curl -s $BASE/api/blockchain/validate | jq -r '.data')
echo "验证: $VALID"
```

---

## 单位说明

SimpleBTC 所有金额字段均以 **satoshi** 为单位（与 Bitcoin 保持一致）：

| 单位 | 换算 |
|------|------|
| 1 BTC | 100,000,000 satoshi |
| 1 mBTC | 100,000 satoshi |
| 1 satoshi | 最小单位，不可再分 |

创世钱包预置余额为 `10,000,000,000 satoshi`（100 BTC）。挖矿奖励默认为 `5000 satoshi`。
