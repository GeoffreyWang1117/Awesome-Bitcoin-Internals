# REST API 参考文档

SimpleBTC提供完整的RESTful API，用于通过HTTP与区块链系统交互。本文档详细介绍所有API端点、请求/响应格式和使用示例。

## 快速开始

### 启动API服务器

```bash
# 编译并运行
cargo run --bin server

# 或使用release模式（性能更好）
cargo run --release --bin server
```

**服务器信息**：
```
========================================
   SimpleBTC API Server
========================================
🚀 服务器运行在: http://127.0.0.1:3000

API 端点:
  GET  /api/blockchain/info           - 获取区块链信息
  GET  /api/blockchain/chain          - 获取完整区块链
  POST /api/wallet/create             - 创建新钱包
  GET  /api/wallet/balance/:address   - 查询余额
  POST /api/transaction/create        - 创建交易
  POST /api/mine                      - 挖矿
  GET  /api/blockchain/validate       - 验证区块链
========================================
```

### 基本配置

- **基础URL**: `http://127.0.0.1:3000`
- **协议**: HTTP/1.1
- **格式**: JSON
- **CORS**: 允许所有来源（开发模式）
- **认证**: 无（演示版本）

## 通用响应格式

所有API端点使用统一的响应格式：

### 成功响应

```json
{
  "success": true,
  "data": { /* 响应数据 */ },
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

### HTTP状态码

| 状态码 | 说明 | 场景 |
|--------|------|------|
| 200 | OK | 请求成功 |
| 400 | Bad Request | 参数错误、交易失败 |
| 404 | Not Found | 资源不存在 |
| 500 | Internal Server Error | 服务器内部错误 |

---

## API端点详解

### 1. 根路径

获取API服务器信息。

#### 请求

```http
GET / HTTP/1.1
Host: 127.0.0.1:3000
```

#### 响应

```
SimpleBTC API Server v1.0 - Bitcoin Banking System
```

#### 示例

**cURL**:
```bash
curl http://127.0.0.1:3000/
```

**JavaScript**:
```javascript
fetch('http://127.0.0.1:3000/')
  .then(res => res.text())
  .then(console.log);
```

**Python**:
```python
import requests
response = requests.get('http://127.0.0.1:3000/')
print(response.text)
```

---

### 2. 获取区块链信息

获取区块链的当前状态和统计信息。

#### 请求

```http
GET /api/blockchain/info HTTP/1.1
Host: 127.0.0.1:3000
```

#### 响应

```json
{
  "success": true,
  "data": {
    "height": 15,
    "difficulty": 4,
    "pending_transactions": 3,
    "mining_reward": 5000
  },
  "error": null
}
```

#### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| height | number | 区块链高度（区块数量） |
| difficulty | number | 挖矿难度（前导零个数） |
| pending_transactions | number | 待确认交易数量 |
| mining_reward | number | 挖矿奖励（satoshi） |

#### 示例

**cURL**:
```bash
curl http://127.0.0.1:3000/api/blockchain/info
```

**JavaScript**:
```javascript
async function getBlockchainInfo() {
  const res = await fetch('http://127.0.0.1:3000/api/blockchain/info');
  const data = await res.json();

  if (data.success) {
    console.log('区块链高度:', data.data.height);
    console.log('挖矿难度:', data.data.difficulty);
    console.log('待确认交易:', data.data.pending_transactions);
    console.log('挖矿奖励:', data.data.mining_reward);
  }
}

getBlockchainInfo();
```

**Python**:
```python
import requests

def get_blockchain_info():
    url = 'http://127.0.0.1:3000/api/blockchain/info'
    response = requests.get(url)
    data = response.json()

    if data['success']:
        info = data['data']
        print(f"区块链高度: {info['height']}")
        print(f"挖矿难度: {info['difficulty']}")
        print(f"待确认交易: {info['pending_transactions']}")
        print(f"挖矿奖励: {info['mining_reward']}")

get_blockchain_info()
```

---

### 3. 获取完整区块链

获取所有区块的详细信息。

#### 请求

```http
GET /api/blockchain/chain HTTP/1.1
Host: 127.0.0.1:3000
```

#### 响应

```json
{
  "success": true,
  "data": [
    {
      "index": 0,
      "timestamp": 1701234567890,
      "transactions": [],
      "previous_hash": "0",
      "hash": "000abc...",
      "nonce": 12345,
      "merkle_root": "abc123...",
      "difficulty": 4
    },
    {
      "index": 1,
      "timestamp": 1701234578901,
      "transactions": [
        {
          "id": "tx123...",
          "inputs": [...],
          "outputs": [...],
          "timestamp": 1701234578900
        }
      ],
      "previous_hash": "000abc...",
      "hash": "000def...",
      "nonce": 67890,
      "merkle_root": "def456...",
      "difficulty": 4
    }
  ],
  "error": null
}
```

#### 字段说明

**Block字段**:

| 字段 | 类型 | 说明 |
|------|------|------|
| index | number | 区块索引（从0开始） |
| timestamp | number | 区块时间戳（毫秒） |
| transactions | array | 交易列表 |
| previous_hash | string | 前一个区块的哈希 |
| hash | string | 当前区块的哈希 |
| nonce | number | 工作量证明的随机数 |
| merkle_root | string | 交易的Merkle根 |
| difficulty | number | 区块难度 |

**Transaction字段**:

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string | 交易ID（哈希） |
| inputs | array | 交易输入列表 |
| outputs | array | 交易输出列表 |
| timestamp | number | 交易时间戳 |

#### 示例

**cURL**:
```bash
curl http://127.0.0.1:3000/api/blockchain/chain | jq
```

**JavaScript**:
```javascript
async function getChain() {
  const res = await fetch('http://127.0.0.1:3000/api/blockchain/chain');
  const data = await res.json();

  if (data.success) {
    const chain = data.data;
    console.log(`区块链长度: ${chain.length}`);

    // 打印每个区块
    chain.forEach(block => {
      console.log(`\n区块 #${block.index}`);
      console.log(`  哈希: ${block.hash.substring(0, 16)}...`);
      console.log(`  交易数: ${block.transactions.length}`);
      console.log(`  时间: ${new Date(block.timestamp).toLocaleString()}`);
    });
  }
}

getChain();
```

**Python**:
```python
import requests
from datetime import datetime

def get_chain():
    url = 'http://127.0.0.1:3000/api/blockchain/chain'
    response = requests.get(url)
    data = response.json()

    if data['success']:
        chain = data['data']
        print(f"区块链长度: {len(chain)}\n")

        for block in chain:
            print(f"区块 #{block['index']}")
            print(f"  哈希: {block['hash'][:16]}...")
            print(f"  交易数: {len(block['transactions'])}")
            timestamp = datetime.fromtimestamp(block['timestamp'] / 1000)
            print(f"  时间: {timestamp}\n")

get_chain()
```

---

### 4. 创建钱包

创建新的钱包地址和密钥对。

#### 请求

```http
POST /api/wallet/create HTTP/1.1
Host: 127.0.0.1:3000
Content-Type: application/json
```

#### 响应

```json
{
  "success": true,
  "data": {
    "address": "a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c",
    "public_key": "04d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c8f1a3d6e9b4c7f0a3d6e9b4c7f"
  },
  "error": null
}
```

#### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| address | string | 钱包地址（40字符十六进制） |
| public_key | string | 公钥（用于验证签名） |

#### 注意事项

⚠️ **安全警告**：
- 此端点生成的私钥**不会**返回给客户端
- 实际应用中需要实现私钥管理机制
- 演示版本中，私钥存储在服务器内存中

#### 示例

**cURL**:
```bash
curl -X POST http://127.0.0.1:3000/api/wallet/create
```

**JavaScript**:
```javascript
async function createWallet() {
  const res = await fetch('http://127.0.0.1:3000/api/wallet/create', {
    method: 'POST'
  });
  const data = await res.json();

  if (data.success) {
    const wallet = data.data;
    console.log('新钱包已创建:');
    console.log('  地址:', wallet.address);
    console.log('  公钥:', wallet.public_key.substring(0, 32) + '...');

    // 保存地址供后续使用
    localStorage.setItem('myAddress', wallet.address);
  }
}

createWallet();
```

**Python**:
```python
import requests
import json

def create_wallet():
    url = 'http://127.0.0.1:3000/api/wallet/create'
    response = requests.post(url)
    data = response.json()

    if data['success']:
        wallet = data['data']
        print('新钱包已创建:')
        print(f"  地址: {wallet['address']}")
        print(f"  公钥: {wallet['public_key'][:32]}...")

        # 保存到文件
        with open('wallet.json', 'w') as f:
            json.dump(wallet, f)

create_wallet()
```

---

### 5. 查询余额

查询指定地址的账户余额。

#### 请求

```http
GET /api/wallet/balance/:address HTTP/1.1
Host: 127.0.0.1:3000
```

#### 路径参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| address | string | 是 | 钱包地址 |

#### 响应

```json
{
  "success": true,
  "data": {
    "address": "a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c",
    "balance": 15000
  },
  "error": null
}
```

#### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| address | string | 查询的地址 |
| balance | number | 余额（satoshi） |

#### 示例

**cURL**:
```bash
ADDRESS="a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c"
curl http://127.0.0.1:3000/api/wallet/balance/$ADDRESS
```

**JavaScript**:
```javascript
async function getBalance(address) {
  const url = `http://127.0.0.1:3000/api/wallet/balance/${address}`;
  const res = await fetch(url);
  const data = await res.json();

  if (data.success) {
    const { address, balance } = data.data;
    console.log(`地址: ${address.substring(0, 16)}...`);
    console.log(`余额: ${balance} satoshi`);
    console.log(`     ${(balance / 100000000).toFixed(8)} BTC`);
  }
}

const myAddress = 'a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c';
getBalance(myAddress);
```

**Python**:
```python
import requests

def get_balance(address):
    url = f'http://127.0.0.1:3000/api/wallet/balance/{address}'
    response = requests.get(url)
    data = response.json()

    if data['success']:
        balance_info = data['data']
        print(f"地址: {balance_info['address'][:16]}...")
        print(f"余额: {balance_info['balance']} satoshi")
        print(f"     {balance_info['balance'] / 100000000:.8f} BTC")

my_address = 'a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c'
get_balance(my_address)
```

---

### 6. 创建交易

创建新的转账交易并加入待确认池。

#### 请求

```http
POST /api/transaction/create HTTP/1.1
Host: 127.0.0.1:3000
Content-Type: application/json

{
  "from_address": "a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c",
  "to_address": "b4g3e9d0f5c8g2b0ad3e6f9c4d7b2e5f8e0c3b6d",
  "amount": 5000,
  "fee": 10
}
```

#### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| from_address | string | 是 | 发送方地址 |
| to_address | string | 是 | 接收方地址 |
| amount | number | 是 | 转账金额（satoshi） |
| fee | number | 是 | 交易手续费（satoshi） |

#### 成功响应

```json
{
  "success": true,
  "data": "交易已创建: tx_abc123...",
  "error": null
}
```

#### 错误响应

```json
{
  "success": false,
  "data": null,
  "error": "余额不足"
}
```

#### 常见错误

| 错误信息 | 原因 | 解决方法 |
|---------|------|---------|
| 余额不足 | 发送方余额 < 金额 + 手续费 | 减少金额或充值 |
| 无效的地址 | 地址格式错误 | 检查地址格式 |
| 金额必须大于0 | 金额为0或负数 | 输入正确金额 |

#### 示例

**cURL**:
```bash
curl -X POST http://127.0.0.1:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{
    "from_address": "a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c",
    "to_address": "b4g3e9d0f5c8g2b0ad3e6f9c4d7b2e5f8e0c3b6d",
    "amount": 5000,
    "fee": 10
  }'
```

**JavaScript**:
```javascript
async function createTransaction(from, to, amount, fee) {
  const res = await fetch('http://127.0.0.1:3000/api/transaction/create', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      from_address: from,
      to_address: to,
      amount: amount,
      fee: fee
    })
  });

  const data = await res.json();

  if (data.success) {
    console.log('✓', data.data);
  } else {
    console.error('✗ 交易失败:', data.error);
  }

  return data;
}

// 使用示例
createTransaction(
  'a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c',
  'b4g3e9d0f5c8g2b0ad3e6f9c4d7b2e5f8e0c3b6d',
  5000,
  10
);
```

**Python**:
```python
import requests

def create_transaction(from_addr, to_addr, amount, fee):
    url = 'http://127.0.0.1:3000/api/transaction/create'
    payload = {
        'from_address': from_addr,
        'to_address': to_addr,
        'amount': amount,
        'fee': fee
    }

    response = requests.post(url, json=payload)
    data = response.json()

    if data['success']:
        print('✓', data['data'])
    else:
        print('✗ 交易失败:', data['error'])

    return data

# 使用示例
create_transaction(
    from_addr='a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c',
    to_addr='b4g3e9d0f5c8g2b0ad3e6f9c4d7b2e5f8e0c3b6d',
    amount=5000,
    fee=10
)
```

---

### 7. 挖矿

挖掘新区块，将待确认交易打包。

#### 请求

```http
POST /api/mine HTTP/1.1
Host: 127.0.0.1:3000
Content-Type: application/json

{
  "miner_address": "a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c"
}
```

#### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| miner_address | string | 是 | 矿工地址（接收挖矿奖励） |

#### 成功响应

```json
{
  "success": true,
  "data": "区块已挖出，当前高度: 16",
  "error": null
}
```

#### 错误响应

```json
{
  "success": false,
  "data": null,
  "error": "没有待确认交易"
}
```

#### 注意事项

- 挖矿需要计算工作量证明（PoW），可能需要数秒到数分钟
- 挖矿难度由区块链的`difficulty`字段决定
- 矿工奖励会自动添加到矿工地址
- 交易手续费也会归矿工所有

#### 示例

**cURL**:
```bash
curl -X POST http://127.0.0.1:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address": "a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c"}'
```

**JavaScript**:
```javascript
async function mine(minerAddress) {
  console.log('开始挖矿...');
  const startTime = Date.now();

  const res = await fetch('http://127.0.0.1:3000/api/mine', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      miner_address: minerAddress
    })
  });

  const data = await res.json();
  const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);

  if (data.success) {
    console.log(`✓ ${data.data}`);
    console.log(`  耗时: ${elapsed}秒`);
  } else {
    console.error('✗ 挖矿失败:', data.error);
  }

  return data;
}

// 使用示例
const minerAddr = 'a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c';
mine(minerAddr);
```

**Python**:
```python
import requests
import time

def mine(miner_address):
    print('开始挖矿...')
    start_time = time.time()

    url = 'http://127.0.0.1:3000/api/mine'
    payload = {'miner_address': miner_address}

    response = requests.post(url, json=payload)
    data = response.json()

    elapsed = time.time() - start_time

    if data['success']:
        print(f"✓ {data['data']}")
        print(f"  耗时: {elapsed:.2f}秒")
    else:
        print(f"✗ 挖矿失败: {data['error']}")

    return data

# 使用示例
miner_addr = 'a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c'
mine(miner_addr)
```

---

### 8. 验证区块链

验证整个区块链的完整性。

#### 请求

```http
GET /api/blockchain/validate HTTP/1.1
Host: 127.0.0.1:3000
```

#### 响应

**有效时**:
```json
{
  "success": true,
  "data": "区块链有效",
  "error": null
}
```

**无效时**:
```json
{
  "success": false,
  "data": "区块链无效",
  "error": null
}
```

#### 验证内容

验证包括以下检查：
1. 创世区块有效性
2. 每个区块的哈希正确性
3. 区块链的连续性（previous_hash匹配）
4. 工作量证明有效性（哈希满足难度要求）
5. 交易签名有效性
6. 余额一致性（防止双花）

#### 示例

**cURL**:
```bash
curl http://127.0.0.1:3000/api/blockchain/validate
```

**JavaScript**:
```javascript
async function validateChain() {
  const res = await fetch('http://127.0.0.1:3000/api/blockchain/validate');
  const data = await res.json();

  if (data.success) {
    console.log('✓ 区块链验证通过');
  } else {
    console.error('✗ 区块链验证失败');
    console.error('  可能原因: 区块被篡改或链断裂');
  }

  return data;
}

validateChain();
```

**Python**:
```python
import requests

def validate_chain():
    url = 'http://127.0.0.1:3000/api/blockchain/validate'
    response = requests.get(url)
    data = response.json()

    if data['success']:
        print('✓ 区块链验证通过')
    else:
        print('✗ 区块链验证失败')
        print('  可能原因: 区块被篡改或链断裂')

    return data

validate_chain()
```

---

## 完整使用流程

以下是一个完整的使用示例，展示如何通过API完成转账流程。

### JavaScript完整示例

```javascript
const BASE_URL = 'http://127.0.0.1:3000';

async function completeWorkflow() {
  console.log('=== SimpleBTC API 完整流程演示 ===\n');

  // 1. 创建Alice的钱包
  console.log('1. 创建Alice的钱包');
  const aliceRes = await fetch(`${BASE_URL}/api/wallet/create`, {
    method: 'POST'
  });
  const aliceData = await aliceRes.json();
  const alice = aliceData.data;
  console.log(`   Alice地址: ${alice.address.substring(0, 20)}...\n`);

  // 2. 创建Bob的钱包
  console.log('2. 创建Bob的钱包');
  const bobRes = await fetch(`${BASE_URL}/api/wallet/create`, {
    method: 'POST'
  });
  const bobData = await bobRes.json();
  const bob = bobData.data;
  console.log(`   Bob地址: ${bob.address.substring(0, 20)}...\n`);

  // 3. 查看区块链状态
  console.log('3. 查看区块链状态');
  const infoRes = await fetch(`${BASE_URL}/api/blockchain/info`);
  const infoData = await infoRes.json();
  console.log(`   区块高度: ${infoData.data.height}`);
  console.log(`   挖矿奖励: ${infoData.data.mining_reward} satoshi\n`);

  // 4. Alice挖矿获得初始资金
  console.log('4. Alice挖矿获得初始资金');
  const mineRes = await fetch(`${BASE_URL}/api/mine`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ miner_address: alice.address })
  });
  const mineData = await mineRes.json();
  console.log(`   ${mineData.data}\n`);

  // 5. 查看Alice余额
  console.log('5. 查看Alice余额');
  const balanceRes = await fetch(
    `${BASE_URL}/api/wallet/balance/${alice.address}`
  );
  const balanceData = await balanceRes.json();
  console.log(`   余额: ${balanceData.data.balance} satoshi\n`);

  // 6. Alice向Bob转账
  console.log('6. Alice向Bob转账2000 satoshi');
  const txRes = await fetch(`${BASE_URL}/api/transaction/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      from_address: alice.address,
      to_address: bob.address,
      amount: 2000,
      fee: 10
    })
  });
  const txData = await txRes.json();
  console.log(`   ${txData.data}\n`);

  // 7. 挖矿确认交易
  console.log('7. 挖矿确认交易');
  const mine2Res = await fetch(`${BASE_URL}/api/mine`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ miner_address: alice.address })
  });
  const mine2Data = await mine2Res.json();
  console.log(`   ${mine2Data.data}\n`);

  // 8. 查看最终余额
  console.log('8. 查看最终余额');
  const aliceBalance = await fetch(
    `${BASE_URL}/api/wallet/balance/${alice.address}`
  );
  const aliceBalData = await aliceBalance.json();
  console.log(`   Alice: ${aliceBalData.data.balance} satoshi`);

  const bobBalance = await fetch(
    `${BASE_URL}/api/wallet/balance/${bob.address}`
  );
  const bobBalData = await bobBalance.json();
  console.log(`   Bob: ${bobBalData.data.balance} satoshi\n`);

  // 9. 验证区块链
  console.log('9. 验证区块链');
  const validateRes = await fetch(`${BASE_URL}/api/blockchain/validate`);
  const validateData = await validateRes.json();
  console.log(`   ${validateData.data}\n`);

  console.log('=== 演示完成 ===');
}

completeWorkflow();
```

### Python完整示例

```python
import requests
import json

BASE_URL = 'http://127.0.0.1:3000'

def complete_workflow():
    print('=== SimpleBTC API 完整流程演示 ===\n')

    # 1. 创建Alice的钱包
    print('1. 创建Alice的钱包')
    alice_res = requests.post(f'{BASE_URL}/api/wallet/create')
    alice = alice_res.json()['data']
    print(f"   Alice地址: {alice['address'][:20]}...\n")

    # 2. 创建Bob的钱包
    print('2. 创建Bob的钱包')
    bob_res = requests.post(f'{BASE_URL}/api/wallet/create')
    bob = bob_res.json()['data']
    print(f"   Bob地址: {bob['address'][:20]}...\n")

    # 3. 查看区块链状态
    print('3. 查看区块链状态')
    info_res = requests.get(f'{BASE_URL}/api/blockchain/info')
    info = info_res.json()['data']
    print(f"   区块高度: {info['height']}")
    print(f"   挖矿奖励: {info['mining_reward']} satoshi\n")

    # 4. Alice挖矿获得初始资金
    print('4. Alice挖矿获得初始资金')
    mine_res = requests.post(f'{BASE_URL}/api/mine', json={
        'miner_address': alice['address']
    })
    mine_data = mine_res.json()
    print(f"   {mine_data['data']}\n")

    # 5. 查看Alice余额
    print('5. 查看Alice余额')
    balance_res = requests.get(
        f"{BASE_URL}/api/wallet/balance/{alice['address']}"
    )
    balance = balance_res.json()['data']['balance']
    print(f"   余额: {balance} satoshi\n")

    # 6. Alice向Bob转账
    print('6. Alice向Bob转账2000 satoshi')
    tx_res = requests.post(f'{BASE_URL}/api/transaction/create', json={
        'from_address': alice['address'],
        'to_address': bob['address'],
        'amount': 2000,
        'fee': 10
    })
    tx_data = tx_res.json()
    print(f"   {tx_data['data']}\n")

    # 7. 挖矿确认交易
    print('7. 挖矿确认交易')
    mine2_res = requests.post(f'{BASE_URL}/api/mine', json={
        'miner_address': alice['address']
    })
    mine2_data = mine2_res.json()
    print(f"   {mine2_data['data']}\n")

    # 8. 查看最终余额
    print('8. 查看最终余额')
    alice_bal = requests.get(
        f"{BASE_URL}/api/wallet/balance/{alice['address']}"
    ).json()['data']['balance']
    print(f"   Alice: {alice_bal} satoshi")

    bob_bal = requests.get(
        f"{BASE_URL}/api/wallet/balance/{bob['address']}"
    ).json()['data']['balance']
    print(f"   Bob: {bob_bal} satoshi\n")

    # 9. 验证区块链
    print('9. 验证区块链')
    validate_res = requests.get(f'{BASE_URL}/api/blockchain/validate')
    validate_data = validate_res.json()
    print(f"   {validate_data['data']}\n")

    print('=== 演示完成 ===')

if __name__ == '__main__':
    complete_workflow()
```

### 预期输出

```
=== SimpleBTC API 完整流程演示 ===

1. 创建Alice的钱包
   Alice地址: a3f2d8c9e4b7f1a89c2d...

2. 创建Bob的钱包
   Bob地址: b4g3e9d0f5c8g2b0ad3e...

3. 查看区块链状态
   区块高度: 1
   挖矿奖励: 5000 satoshi

4. Alice挖矿获得初始资金
   区块已挖出，当前高度: 2

5. 查看Alice余额
   余额: 5000 satoshi

6. Alice向Bob转账2000 satoshi
   交易已创建: tx_abc123...

7. 挖矿确认交易
   区块已挖出，当前高度: 3

8. 查看最终余额
   Alice: 8000 satoshi
   Bob: 2000 satoshi

9. 验证区块链
   区块链有效

=== 演示完成 ===
```

**余额计算说明**：
```
Alice最终余额 = 初始挖矿奖励 + 第二次挖矿奖励 + 手续费 - 转账金额
             = 5000 + 5000 + 10 - 2000 - 10
             = 8000 satoshi

Bob最终余额 = 转账金额
           = 2000 satoshi
```

---

## 错误处理

### 常见错误及解决方法

#### 1. 连接错误

**错误**:
```
Failed to fetch: Network request failed
```

**原因**: 服务器未启动

**解决**:
```bash
cargo run --bin server
```

#### 2. JSON解析错误

**错误**:
```json
{
  "success": false,
  "error": "Failed to deserialize JSON"
}
```

**原因**: 请求体格式错误

**解决**: 确保Content-Type为`application/json`且JSON格式正确

#### 3. 余额不足

**错误**:
```json
{
  "success": false,
  "error": "余额不足"
}
```

**解决**: 先挖矿获得资金或减少转账金额

#### 4. 无待确认交易

**错误**:
```json
{
  "success": false,
  "error": "没有待确认交易"
}
```

**解决**: 先创建交易再挖矿

---

## 性能优化建议

### 1. 使用连接池

**JavaScript**:
```javascript
const agent = new http.Agent({
  keepAlive: true,
  maxSockets: 50
});

fetch(url, { agent });
```

**Python**:
```python
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry

session = requests.Session()
retry = Retry(total=3, backoff_factor=0.1)
adapter = HTTPAdapter(max_retries=retry, pool_connections=10, pool_maxsize=100)
session.mount('http://', adapter)

session.get(url)
```

### 2. 批量请求

```javascript
// 并发请求多个地址的余额
const addresses = [addr1, addr2, addr3];
const balances = await Promise.all(
  addresses.map(addr =>
    fetch(`${BASE_URL}/api/wallet/balance/${addr}`)
      .then(res => res.json())
  )
);
```

### 3. 缓存区块链信息

```javascript
let cachedInfo = null;
let cacheTime = 0;
const CACHE_TTL = 5000;  // 5秒缓存

async function getBlockchainInfo() {
  const now = Date.now();
  if (cachedInfo && now - cacheTime < CACHE_TTL) {
    return cachedInfo;
  }

  const res = await fetch(`${BASE_URL}/api/blockchain/info`);
  cachedInfo = await res.json();
  cacheTime = now;

  return cachedInfo;
}
```

---

## 安全注意事项

### 1. 生产环境配置

```rust
// ❌ 开发环境（当前）
let cors = CorsLayer::new()
    .allow_origin(Any)           // 允许所有来源
    .allow_methods(Any)          // 允许所有方法
    .allow_headers(Any);         // 允许所有头部

// ✅ 生产环境（推荐）
use tower_http::cors::AllowOrigin;

let cors = CorsLayer::new()
    .allow_origin("https://myapp.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
```

### 2. 添加认证

```rust
// JWT认证示例
use axum::middleware;

async fn auth_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_jwt(token) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

let app = Router::new()
    .route("/api/*", /* ... */)
    .layer(middleware::from_fn(auth_middleware));
```

### 3. 速率限制

```rust
use tower::limit::RateLimitLayer;
use std::time::Duration;

let rate_limit = RateLimitLayer::new(
    100,                           // 100个请求
    Duration::from_secs(60),       // 每分钟
);

let app = Router::new()
    .route("/api/*", /* ... */)
    .layer(rate_limit);
```

### 4. HTTPS加密

```bash
# 生产环境使用HTTPS
# 使用反向代理（Nginx/Caddy）处理TLS
server {
    listen 443 ssl;
    server_name api.mybtc.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
    }
}
```

---

## 扩展功能

### 未来可能的API端点

```
# 交易历史
GET /api/wallet/transactions/:address

# 查询特定交易
GET /api/transaction/:txid

# 查询特定区块
GET /api/block/:index

# WebSocket实时通知
WS /api/subscribe

# 多签创建
POST /api/multisig/create

# RBF交易替换
POST /api/transaction/replace

# 交易统计
GET /api/stats
```

---

## 参考资料

- [Axum Web Framework](https://github.com/tokio-rs/axum) - Rust异步Web框架
- [RESTful API设计指南](https://restfulapi.net/) - REST最佳实践
- [Bitcoin JSON-RPC API](https://developer.bitcoin.org/reference/rpc/) - 比特币官方RPC参考
- [Blockchain API](./blockchain.md) - 区块链核心API
- [Transaction API](./transaction.md) - 交易API
- [Wallet API](./wallet.md) - 钱包API

---

[返回API目录](./core.md) | [查看快速入门](../guide/quickstart.md)
