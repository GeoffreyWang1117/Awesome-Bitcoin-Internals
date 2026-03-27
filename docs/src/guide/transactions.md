# 交易处理

比特币的核心创新之一是 **UTXO（Unspent Transaction Output）模型**。与银行账户余额不同，比特币系统中不存在"账户余额"这一概念——所有资金都以未花费交易输出的形式分散存在于区块链中。本章深入介绍 SimpleBTC 的交易结构、UTXO 模型、交易创建和验证机制。

---

## UTXO 模型基础

### 什么是 UTXO

UTXO 是"Unspent Transaction Output"（未花费交易输出）的缩写。每笔比特币交易消费若干现有 UTXO（作为输入），同时创造若干新 UTXO（作为输出）。

```
┌──────────────────────────────────────────────────────────────────┐
│ 传统账户模型（如银行）                                            │
│  Alice 余额: 100        Bob 余额: 0                              │
│  转账 30 → Alice: 70, Bob: 30                                    │
├──────────────────────────────────────────────────────────────────┤
│ UTXO 模型（比特币）                                               │
│  区块链上存在：UTXO_A (属于 Alice, 值 100)                        │
│  转账 30：                                                        │
│    消费：UTXO_A (100)  ← 必须整体消费，不能部分花费              │
│    创造：UTXO_B (30, 属于 Bob)   ← 转账金额                      │
│          UTXO_C (60, 属于 Alice) ← 找零（100 - 30 - 10 手续费）  │
└──────────────────────────────────────────────────────────────────┘
```

**UTXO 模型的关键特性：**
- 每个 UTXO 只能被花费一次（花费后从 UTXO 集合中移除）
- 花费时必须消费完整的 UTXO，多余的部分以"找零"输出返还给发送者
- "余额"= 某地址拥有的所有 UTXO 价值之和（由 `UTXOSet` 计算）
- 没有被花费的 UTXO 形成"UTXO 集"（比特币全节点需要维护约 5-10 GB 的 UTXO 集）

---

## 交易数据结构

### TxInput（交易输入）

交易输入引用一个现有的 UTXO，并提供花费它的授权证明（数字签名）：

```rust
// src/transaction.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: String,       // 被引用交易的 ID（32 字节哈希，64 hex 字符）
    pub vout: usize,        // 该交易中第几个输出（从 0 开始）
    pub signature: String,  // ECDSA 签名（DER 编码，hex 字符串）
    pub pub_key: String,    // 发送者的压缩公钥（33 字节，66 hex 字符）
}
```

`txid + vout` 的组合唯一定位区块链上的某个 UTXO。`signature` 由发送者的私钥生成，证明其对该 UTXO 的所有权。

### TxOutput（交易输出）

交易输出定义了接收方可获得的金额，是新 UTXO 的载体：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u64,           // 金额（单位：satoshi，1 BTC = 10⁸ satoshi）
    pub pub_key_hash: String, // 接收者地址（P2PKH）= 锁定脚本
}

impl TxOutput {
    pub fn new(value: u64, address: String) -> Self {
        TxOutput { value, pub_key_hash: address }
    }

    /// 检查是否可以被某地址解锁（即该地址是否为此输出的接收者）
    pub fn can_be_unlocked_with(&self, address: &str) -> bool {
        self.pub_key_hash == address
    }
}
```

`pub_key_hash` 在真实比特币中是锁定脚本（locking script / scriptPubKey），这里简化为接收者地址。

### Transaction（交易）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,              // 交易 ID = SHA-256(交易内容)
    pub inputs: Vec<TxInput>,    // 输入列表（消费哪些 UTXO）
    pub outputs: Vec<TxOutput>,  // 输出列表（创建哪些新 UTXO）
    pub timestamp: u64,          // Unix 时间戳（秒）
    pub fee: u64,                // 手续费 = 输入总额 - 输出总额
}
```

**不变量：** `输入总额 = 输出总额 + 手续费`

---

## 创建普通交易

普通交易通过 `Blockchain::create_transaction()` 创建，该方法会自动处理 UTXO 选择、找零计算和签名：

```rust
pub fn create_transaction(
    &self,
    from_wallet: &Wallet,   // 发送者钱包（需要私钥签名）
    to_address: String,     // 接收者地址
    amount: u64,            // 转账金额（satoshi）
    fee: u64,               // 手续费（satoshi）
) -> Result<Transaction, String>
```

**内部流程：**

```rust
// src/blockchain.rs 节选（简化展示）

// 1. 计算总需求
let total_needed = amount + fee;

// 2. 从 UTXO 集合中查找足够的 UTXO（排除已被待确认交易花费的）
let spendable = self.utxo_set.find_spendable_outputs_excluding(
    &from_wallet.address,
    total_needed,
    &self.pending_spent,
);
let (accumulated, utxos) = spendable.ok_or_else(|| "余额不足（包括交易费）".to_string())?;

// 3. 对每个选中的 UTXO 创建输入并签名
let mut inputs = Vec::new();
for (txid, vout) in utxos {
    let signature = from_wallet.sign(&format!("{}{}", txid, vout));
    let input = TxInput::new(txid, vout, signature, from_wallet.public_key.clone());
    inputs.push(input);
}

// 4. 创建输出（转账 + 找零）
let mut outputs = Vec::new();
outputs.push(TxOutput::new(amount, to_address));
if accumulated > total_needed {
    // 找零返还给发送者（扣除手续费）
    outputs.push(TxOutput::new(accumulated - total_needed, from_wallet.address.clone()));
}

// 5. 构造交易（自动计算 ID）
Ok(Transaction::new(inputs, outputs, timestamp, fee))
```

完整使用示例：

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

let mut blockchain = Blockchain::new();
let genesis = Blockchain::genesis_wallet(); // 预置资金 10M satoshi
let alice = Wallet::new();

// genesis 向 alice 转账 5000 satoshi，手续费 100
let tx = blockchain.create_transaction(
    &genesis,
    alice.address.clone(),
    5000,   // 转账金额
    100,    // 手续费
)?;

println!("交易 ID: {}", tx.id);
println!("输入数量: {}", tx.inputs.len());
println!("输出数量: {}", tx.outputs.len()); // 通常为 2（转账 + 找零）
println!("手续费: {} satoshi", tx.fee);
println!("费率: {:.2} sat/byte", tx.fee_rate());
```

---

## Coinbase 交易

Coinbase 交易是每个区块的**第一笔交易**，专门用于向矿工发放区块奖励。它与普通交易的关键区别：

| 特性 | 普通交易 | Coinbase 交易 |
|------|---------|--------------|
| 输入 | 引用现有 UTXO | 空 txid（凭空创建） |
| 输出 | 转账 + 找零 | 区块奖励 + 所有手续费 |
| 签名 | ECDSA 签名 | 无需签名（`pub_key = "coinbase"`） |
| 验证 | 验证所有输入签名 | 跳过签名验证（`is_coinbase() = true`） |

```rust
pub fn new_coinbase(to: String, reward: u64, timestamp: u64, total_fees: u64) -> Self {
    // 使用原子计数器保证每个 coinbase 交易 ID 唯一（类似 BIP34 区块高度编码）
    static COINBASE_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = COINBASE_COUNTER.fetch_add(1, Ordering::Relaxed);

    // 输出 = 区块奖励 + 所有交易手续费
    let tx_out = TxOutput::new(reward + total_fees, to);
    let tx_in = TxInput {
        txid: String::new(),              // 空 txid 标识 coinbase
        vout: 0,
        signature: format!("coinbase:{}", nonce),  // 唯一性字段
        pub_key: String::from("coinbase"),
    };
    // ...
}
```

Coinbase 交易识别方式：

```rust
pub fn is_coinbase(&self) -> bool {
    // 只有一个输入，且该输入的 txid 为空
    self.inputs.len() == 1 && self.inputs[0].txid.is_empty()
}
```

---

## 交易验证

`Transaction::verify()` 验证所有输入的 ECDSA 签名：

```rust
pub fn verify(&self) -> bool {
    // Coinbase 交易无需验证签名
    if self.is_coinbase() {
        return true;
    }

    // 必须有输入和输出
    if self.inputs.is_empty() || self.outputs.is_empty() {
        return false;
    }

    // 对每个输入验证 ECDSA 签名
    for input in &self.inputs {
        // 签名的原始数据：被花费 UTXO 的位置标识
        let signed_data = format!("{}{}", input.txid, input.vout);
        if !Wallet::verify_signature(&input.pub_key, &signed_data, &input.signature) {
            return false;
        }
    }

    true
}
```

**验证逻辑说明：**
- 签名数据为 `"{txid}{vout}"`，将签名绑定到具体的 UTXO，防止签名重放攻击
- 使用输入中携带的公钥（`pub_key`）验证签名，全节点无需额外查询
- `Wallet::verify_signature()` 内部调用 secp256k1 执行真实的椭圆曲线数学验证

**添加交易到区块链时的完整验证流程（`Blockchain::add_transaction()`）：**

```rust
// 1. ECDSA 签名验证
if !transaction.verify() {
    return Err("交易验证失败".to_string());
}

// 2. 验证 UTXO 存在且余额充足
let mut input_sum = 0u64;
for input in &transaction.inputs {
    if let Some(outputs) = self.find_transaction_outputs(&input.txid) {
        if let Some((_, output)) = outputs.iter().find(|(idx, _)| *idx == input.vout) {
            input_sum += output.value;
        } else {
            return Err("UTXO 不存在".to_string());
        }
    } else {
        return Err("引用的交易不存在".to_string());
    }
}

let output_sum: u64 = transaction.outputs.iter().map(|o| o.value).sum();
if input_sum < output_sum {
    return Err("余额不足，交易无效".to_string());
}

// 3. 记录待确认 UTXO（防止同一 UTXO 被两笔待确认交易双花）
for input in &transaction.inputs {
    self.pending_spent.insert(format!("{}:{}", input.txid, input.vout));
}
```

---

## 手续费与费率

### 手续费计算

```
手续费 = 输入总额 - 输出总额
```

例如：花费 UTXO (100 satoshi)，转账 85，找零 5，手续费 = 100 - 85 - 5 = 10 satoshi

### 费率

```rust
pub fn fee_rate(&self) -> f64 {
    let size = self.size();  // 交易序列化后的字节大小
    if size == 0 { return 0.0; }
    self.fee as f64 / size as f64  // 单位：satoshi/byte
}
```

典型费率参考（真实比特币，随网络拥堵波动）：

| 优先级 | 费率 | 确认时间 |
|--------|------|---------|
| 低 | 1–5 sat/byte | 数小时甚至更长 |
| 中 | 5–20 sat/byte | 30–60 分钟 |
| 高 | 20–50 sat/byte | 10–20 分钟 |
| 紧急 | 50+ sat/byte | 下一个区块（约 10 分钟） |

SimpleBTC 的内存池按费率排序，高费率交易优先被打包：

```rust
// 获取按费率排序的顶部交易（Mempool 内部逻辑）
let pending_txs = self.mempool.get_top_transactions(usize::MAX);
```

---

## 交易生命周期

```
用户发起转账请求
       │
       ▼
blockchain.create_transaction(&wallet, to, amount, fee)
  → 选择 UTXO，生成签名，构造 Transaction
       │
       ▼
blockchain.add_transaction(tx)
  → 验证签名（ECDSA）
  → 验证 UTXO 存在 + 余额充足
  → 加入 Mempool（按费率排序）
  → 标记已花费 UTXO（pending_spent）
       │
       ▼  等待矿工打包
       │
       ▼
blockchain.mine_pending_transactions(miner_address)
  → 从 Mempool 取出高优先级交易
  → 创建 Coinbase 交易（奖励 + 手续费）
  → 并行 PoW 挖矿
  → 更新 UTXO 集合（原子操作）
  → 区块上链，清空 pending_spent
       │
       ▼
交易获得 1 次确认
（每增加一个后续区块 = +1 次确认）
```

---

## 交易哈希计算

```rust
pub fn calculate_hash(&self) -> String {
    // 将交易序列化为 JSON，计算 SHA-256
    let tx_data = serde_json::to_string(&self).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(tx_data.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

真实比特币使用双重 SHA-256（SHA256d），并且序列化格式为紧凑二进制格式；这里为了教学简化为 JSON + 单次 SHA-256。

---

## 输出总额查询

```rust
// 获取所有输出的金额之和
let output_sum = tx.output_sum();

// 交易大小（字节数，影响手续费计算）
let size = tx.size();

// 是否为 Coinbase 交易
let is_cb = tx.is_coinbase();
```

---

## 完整交易示例

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn main() -> Result<(), String> {
    let mut blockchain = Blockchain::new();

    let genesis = Blockchain::genesis_wallet();
    let alice = Wallet::new();
    let bob = Wallet::new();

    // 第一笔：genesis → alice，转账 10000 satoshi
    let tx1 = blockchain.create_transaction(&genesis, alice.address.clone(), 10000, 50)?;
    println!("tx1 id: {}", tx1.id);
    println!("tx1 输出数: {}", tx1.outputs.len()); // 2（转账 + 找零）
    println!("tx1 费率: {:.2} sat/byte", tx1.fee_rate());
    blockchain.add_transaction(tx1)?;

    // 挖矿确认（alice 收矿工奖励）
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("Alice 余额: {} satoshi", blockchain.get_balance(&alice.address));

    // 第二笔：alice → bob，转账 3000 satoshi
    let tx2 = blockchain.create_transaction(&alice, bob.address.clone(), 3000, 100)?;
    blockchain.add_transaction(tx2)?;

    blockchain.mine_pending_transactions(bob.address.clone())?;

    println!("Alice 余额: {} satoshi", blockchain.get_balance(&alice.address));
    println!("Bob 余额:   {} satoshi", blockchain.get_balance(&bob.address));

    // 验证链完整性
    assert!(blockchain.is_valid());

    Ok(())
}
```
