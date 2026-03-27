# 区块链操作

区块链是 SimpleBTC 的核心数据结构——一个以密码学方式链接的区块序列，每个区块包含一批经过验证的交易。本章介绍 `Block` 结构体、`Blockchain` 的创建与管理、工作量证明挖矿机制，以及链的验证与查询接口。

---

## 区块结构

### Block 数据结构

```rust
// src/block.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,                     // 区块高度（创世区块为 0）
    pub timestamp: u64,                 // Unix 时间戳（秒）
    pub transactions: Vec<Transaction>, // 交易列表（第一笔必须是 Coinbase）
    pub previous_hash: String,          // 父区块哈希（64 字符 SHA-256 hex）
    pub hash: String,                   // 本区块哈希（挖矿找到的有效值）
    pub nonce: u64,                     // 工作量证明计数器
    pub merkle_root: String,            // 所有交易 ID 的 Merkle 树根哈希
}
```

**区块头字段详解：**

| 字段 | 比特币对应字段 | 说明 |
|------|-------------|------|
| `index` | Block Height | 区块高度，创世区块为 0，每新增一块加 1 |
| `timestamp` | nTime | 区块创建时间（Unix 时间戳） |
| `previous_hash` | hashPrevBlock | 父区块哈希，形成链式结构 |
| `merkle_root` | hashMerkleRoot | 所有交易的 Merkle 树根，代表区块内容的指纹 |
| `nonce` | nNonce | 挖矿时不断调整的随机数 |
| `hash` | Block Hash | 区块头所有字段的 SHA-256 哈希 |

### 链式结构示意图

```
创世区块 (index=0)          区块 1                区块 2
┌──────────────────┐     ┌──────────────────┐  ┌──────────────────┐
│ prev: "0"        │◄────│ prev: abc...hash │◄─│ prev: def...hash │
│ hash: abc...     │     │ hash: def...     │  │ hash: ghi...     │
│ nonce: 38291     │     │ nonce: 72481     │  │ nonce: 19374     │
│ merkle: xyz...   │     │ merkle: pqr...   │  │ merkle: stu...   │
│ [Coinbase TX]    │     │ [Coinbase TX]    │  │ [Coinbase TX]    │
│                  │     │ [TX_1]           │  │ [TX_3]           │
│                  │     │ [TX_2]           │  │ [TX_4]           │
└──────────────────┘     └──────────────────┘  └──────────────────┘
```

**为什么链式结构保证不可篡改？**

1. 修改区块 1 中的任意交易 → `merkle_root` 变化
2. `merkle_root` 变化 → 区块 1 的 `hash` 完全不同
3. 区块 2 记录了区块 1 的旧 `hash` → 区块 2 的 `previous_hash` 不再匹配
4. 修复区块 2 需要重新挖矿（重算 PoW），区块 3、4... 同理
5. 攻击者需要掌握全网 51% 以上算力才能追上诚实链

---

## 区块哈希计算

区块哈希由区块头的关键字段计算得出（注意：不直接哈希交易列表，而是使用 Merkle 根）：

```rust
// src/block.rs
pub fn calculate_hash(&self) -> String {
    use sha2::{Digest, Sha256};

    // 将区块头字段拼接为字符串
    let data = format!(
        "{}{}{}{}{}",
        self.index,
        self.timestamp,
        self.merkle_root,    // ← 代表全部交易内容
        self.previous_hash,
        self.nonce           // ← 挖矿时不断改变这个值
    );

    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**Merkle 根的作用：**
- 任何单笔交易的改动都会导致 Merkle 根完全变化
- 验证交易是否在区块中只需 O(log n) 次哈希（Merkle 证明），而非下载全部交易

---

## 创建区块链

### Blockchain 结构体

```rust
// src/blockchain.rs
pub struct Blockchain {
    pub chain: Vec<Block>,           // 区块列表（链）
    pub difficulty: usize,           // 挖矿难度（前导 0 的个数，默认 3）
    pub mempool: Mempool,            // 内存池（待确认交易，按费率排序）
    pub utxo_set: UTXOSet,           // UTXO 集合（所有未花费输出）
    pub mining_reward: u64,          // 挖矿奖励（默认 50 satoshi）
    pub indexer: TransactionIndexer, // 交易索引（地址→交易，加速查询）
    miner: ParallelMiner,            // 并行 PoW 挖矿器（私有）
    pending_spent: HashSet<String>,  // 已被待确认交易花费的 UTXO（防双花）
}
```

### 初始化区块链

```rust
use bitcoin_simulation::blockchain::Blockchain;

// 创建区块链（自动包含创世区块）
let mut blockchain = Blockchain::new();

println!("链长度: {}", blockchain.chain.len());     // 1（仅创世区块）
println!("挖矿难度: {}", blockchain.difficulty);     // 3
println!("挖矿奖励: {} satoshi", blockchain.mining_reward); // 50
```

`Blockchain::new()` 内部流程：

```rust
pub fn new() -> Blockchain {
    let mempool = Mempool::new_permissive();

    let mut blockchain = Blockchain {
        chain: vec![],
        difficulty: 3,
        mempool,
        utxo_set: UTXOSet::new(),
        mining_reward: 50,
        indexer: TransactionIndexer::new(),
        miner: ParallelMiner::default(),
        pending_spent: HashSet::new(),
    };

    // 创建并添加创世区块
    let genesis_block = blockchain.create_genesis_block();
    blockchain.indexer.index_block(&genesis_block);
    blockchain.chain.push(genesis_block);

    blockchain
}
```

---

## 创世区块

创世区块（Genesis Block）是区块链的第一个区块（index = 0）。它的特殊之处：

- `previous_hash` = `"0"`（不引用任何父区块）
- 包含一个 Coinbase 交易，向**创世钱包**发放 10,000,000 satoshi 初始资金
- 使用**确定性创世钱包**（固定私钥 `0x01`），保证每次启动地址一致

```rust
// src/blockchain.rs
fn create_genesis_block(&mut self) -> Block {
    let timestamp = /* 当前 Unix 时间 */;

    // 确定性创世钱包（固定私钥，可被签名花费）
    let genesis_wallet = Wallet::genesis();
    let coinbase_tx = Transaction::new_coinbase(
        genesis_wallet.address,
        10_000_000,  // 创世区块奖励：10M satoshi
        timestamp,
        0,           // 无手续费
    );

    // 将创世 UTXO 加入 UTXO 集合
    self.utxo_set.add_transaction(&coinbase_tx);

    // 创世区块的 previous_hash 固定为 "0"
    Block::new(0, vec![coinbase_tx], "0".to_string())
}
```

获取创世钱包的两种等价方式：

```rust
let genesis = Blockchain::genesis_wallet();  // Blockchain 的静态方法
let genesis2 = Wallet::genesis();            // 直接从 wallet 模块获取
assert_eq!(genesis.address, genesis2.address);
```

---

## 工作量证明（PoW）挖矿

### 原理

工作量证明要求矿工找到一个 `nonce` 值，使得区块哈希满足"前 N 位为 0"的条件：

```
difficulty = 3，目标哈希格式：000xxxxxxxxx...
```

由于 SHA-256 的输出完全不可预测，矿工只能穷举 nonce：

```
nonce=0: hash = "a7f3b2..." → 不满足（不以 "000" 开头）
nonce=1: hash = "2c91d4..." → 不满足
...
nonce=38291: hash = "000a4b7c9..." → 满足！区块挖出
```

平均需要尝试 16³ = 4096 次（难度 3）。真实比特币难度相当于约 20 个前导 0，需要约 2⁸⁰ 次尝试。

### Block::mine_block()（单线程）

```rust
// src/block.rs
pub fn mine_block(&mut self, difficulty: usize) {
    let target = "0".repeat(difficulty);

    while self.hash[..difficulty] != target {
        self.nonce += 1;
        self.hash = self.calculate_hash();
    }

    println!("✓ 区块已挖出: {}", self.hash);
}
```

### ParallelMiner（多线程）

`Blockchain::mine_pending_transactions()` 使用 `ParallelMiner` 而非单线程 `mine_block()`，充分利用多核 CPU：

```rust
// src/blockchain.rs 节选
self.miner
    .mine_block(&mut block, self.difficulty)
    .map_err(|e| format!("挖矿失败: {}", e))?;
```

`ParallelMiner` 将 nonce 空间分割给多个线程并行搜索，第一个找到有效哈希的线程获胜。

### 难度与调整

| 难度值 | 前导零数 | 平均尝试次数 | 适用场景 |
|--------|---------|------------|---------|
| 1 | 1 个 0 | 16 次 | 极快测试 |
| 2 | 2 个 0 | 256 次 | 快速演示 |
| 3 | 3 个 0 | 4,096 次 | 默认配置 |
| 4 | 4 个 0 | 65,536 次 | 性能测试 |
| 6 | 6 个 0 | 16,777,216 次 | 接近真实 |

---

## 添加交易与挖矿

### 完整流程

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

let mut blockchain = Blockchain::new();
let genesis = Blockchain::genesis_wallet();
let alice = Wallet::new();

// 1. 创建交易
let tx = blockchain.create_transaction(
    &genesis,
    alice.address.clone(),
    5000,   // 转账 5000 satoshi
    50,     // 手续费 50 satoshi
)?;

// 2. 添加到内存池（验证签名 + UTXO）
blockchain.add_transaction(tx)?;

println!("内存池交易数: {}", blockchain.mempool.len()); // 1

// 3. 挖矿（alice 作为矿工接收奖励）
blockchain.mine_pending_transactions(alice.address.clone())?;

println!("链长度: {}", blockchain.chain.len()); // 2（创世 + 新区块）
println!("内存池交易数: {}", blockchain.mempool.len()); // 0（已清空）
```

### mine_pending_transactions() 详细流程

```rust
pub fn mine_pending_transactions(&mut self, miner_address: String) -> Result<(), String> {
    if self.mempool.is_empty() {
        return Err("没有待处理的交易".to_string());
    }

    // 1. 从内存池取出高费率交易（已排序）
    let pending_txs = self.mempool.get_top_transactions(usize::MAX);

    // 2. 计算总手续费
    let total_fees: u64 = pending_txs.iter().map(|tx| tx.fee).sum();

    // 3. 创建 Coinbase 交易（矿工奖励 = 区块奖励 + 总手续费）
    let coinbase_tx = Transaction::new_coinbase(
        miner_address,
        self.mining_reward,  // 50 satoshi
        timestamp,
        total_fees,
    );

    // 4. 组装区块（Coinbase 必须是第一笔交易）
    let mut transactions = vec![coinbase_tx];
    transactions.extend(pending_txs.iter().cloned());

    let previous_hash = self.chain.last().unwrap().hash.clone();
    let mut block = Block::new(self.chain.len() as u32, transactions, previous_hash);

    // 5. 并行 PoW 挖矿
    self.miner.mine_block(&mut block, self.difficulty)?;

    // 6. 验证区块中所有交易签名
    if !block.validate_transactions() {
        return Err("区块包含无效交易".to_string());
    }

    // 7. 更新 UTXO 集合（消费输入 UTXO，创建输出 UTXO）
    for tx in &block.transactions {
        if !self.utxo_set.process_transaction(tx) {
            return Err("UTXO 更新失败".to_string());
        }
    }

    // 8. 区块上链 + 建索引
    self.indexer.index_block(&block);
    self.chain.push(block);

    // 9. 清理内存池和 pending_spent
    for tx in &pending_txs {
        let _ = self.mempool.remove_transaction(&tx.id);
    }
    self.pending_spent.clear();

    Ok(())
}
```

---

## Merkle 树与交易验证

`Block::new()` 在创建时自动构建 Merkle 树并计算 Merkle 根：

```rust
pub fn new(index: u32, transactions: Vec<Transaction>, previous_hash: String) -> Block {
    // 收集所有交易 ID
    let tx_ids: Vec<String> = transactions.iter().map(|tx| tx.id.clone()).collect();

    // 构建 Merkle 树，计算根哈希
    let merkle_tree = MerkleTree::new(&tx_ids);
    let merkle_root = merkle_tree.get_root_hash();

    let mut block = Block {
        index, timestamp, transactions, previous_hash,
        hash: String::new(), nonce: 0, merkle_root,
    };
    block.hash = block.calculate_hash();
    block
}
```

验证某笔交易是否包含在区块中（SPV 使用场景）：

```rust
// src/block.rs
pub fn verify_transaction_inclusion(&self, tx_id: &str, index: usize) -> bool {
    let tx_ids: Vec<String> = self.transactions.iter().map(|tx| tx.id.clone()).collect();
    let merkle_tree = MerkleTree::new(&tx_ids);

    if let Some(proof) = merkle_tree.get_proof(tx_id) {
        MerkleTree::verify_proof(tx_id, &proof, &self.merkle_root, index)
    } else {
        false
    }
}
```

---

## 链验证

`Blockchain::is_valid()` 从第 1 块（跳过创世块）开始逐块验证链的完整性：

```rust
pub fn is_valid(&self) -> bool {
    for i in 1..self.chain.len() {
        let current = &self.chain[i];
        let previous = &self.chain[i - 1];

        // 1. 验证区块自身哈希正确性（防止数据被静默篡改）
        if current.hash != current.calculate_hash() {
            println!("区块 {} 哈希无效", i);
            return false;
        }

        // 2. 验证前向引用（链式连接完整性）
        if current.previous_hash != previous.hash {
            println!("区块 {} 的前向引用无效", i);
            return false;
        }

        // 3. 验证工作量证明（哈希前导零满足难度要求）
        let target = "0".repeat(self.difficulty);
        if current.hash[..self.difficulty] != target {
            println!("区块 {} 工作量证明无效", i);
            return false;
        }

        // 4. 验证区块中所有交易的 ECDSA 签名
        if !current.validate_transactions() {
            println!("区块 {} 包含无效交易", i);
            return false;
        }
    }
    true
}
```

验证示例：

```rust
let mut blockchain = Blockchain::new();
// ... 添加交易，挖矿 ...

// 正常情况：应通过
assert!(blockchain.is_valid());

// 模拟篡改（教学用途，实际中 Rust 借用规则会约束直接访问）
// 如果有人修改了历史区块的交易，is_valid() 将返回 false
```

---

## 余额查询

余额通过 UTXO 集合计算，避免扫描全部历史区块：

```rust
pub fn get_balance(&self, address: &str) -> u64 {
    self.utxo_set.get_balance(address)
}
```

使用示例：

```rust
let balance = blockchain.get_balance(&alice.address);
println!("Alice 余额: {} satoshi", balance);
println!("Alice 余额: {:.8} BTC", balance as f64 / 1e8);
```

**UTXOSet 的性能优势：**

不使用 UTXO 集合时，查询余额需要扫描全部区块的全部交易（O(n)，n = 总交易数）。UTXO 集合将当前所有未花费输出缓存在内存中，查询变为 O(1) 的哈希表查找。

---

## 打印区块链信息

`Blockchain::print_chain()` 提供格式化的调试输出：

```rust
blockchain.print_chain();
```

输出示例：

```
========== 区块链信息 ==========

--- 区块 #0 ---
时间戳: 1711497600
哈希: 000a4b7c9d2e1f3a...
前一个哈希: 0
Nonce: 38291
交易数量: 1
  交易 #0: f3a1b2c4...
    类型: Coinbase（挖矿奖励）
    输入数: 1
    输出数: 1
      输出 0: 10000000 -> 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2

--- 区块 #1 ---
时间戳: 1711497615
哈希: 000d2f8a1b9e4c7f...
前一个哈希: 000a4b7c9d2e1f3a...
Nonce: 72481
交易数量: 2
  交易 #0: a1b2c3d4...
    类型: Coinbase（挖矿奖励）
    ...
  交易 #1: e5f6a7b8...
    交易费: 50 satoshi
    费率: 0.23 sat/byte
    输入数: 1
    输出数: 2
      输出 0: 5000 -> 1AliceAddress...
      输出 1: 4994950 -> 1GenesisAddress...

================================
```

---

## 完整操作示例

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn main() -> Result<(), String> {
    // 初始化
    let mut blockchain = Blockchain::new();
    let genesis = Blockchain::genesis_wallet();
    let alice = Wallet::new();
    let bob = Wallet::new();
    let miner = Wallet::new();

    // 第一轮：genesis → alice
    let tx1 = blockchain.create_transaction(&genesis, alice.address.clone(), 100_000, 100)?;
    blockchain.add_transaction(tx1)?;
    blockchain.mine_pending_transactions(miner.address.clone())?;

    println!("区块 1 已挖出");
    println!("Alice 余额: {} sat", blockchain.get_balance(&alice.address));
    println!("矿工余额: {} sat", blockchain.get_balance(&miner.address));

    // 第二轮：alice → bob（两笔交易在同一区块）
    let tx2 = blockchain.create_transaction(&alice, bob.address.clone(), 30_000, 200)?;
    let tx3 = blockchain.create_transaction(&alice, miner.address.clone(), 20_000, 150)?;
    blockchain.add_transaction(tx2)?;
    blockchain.add_transaction(tx3)?;
    blockchain.mine_pending_transactions(miner.address.clone())?;

    println!("\n区块 2 已挖出（含 2 笔交易）");
    println!("链长度: {}", blockchain.chain.len()); // 3
    println!("Alice 余额: {} sat", blockchain.get_balance(&alice.address));
    println!("Bob 余额:   {} sat", blockchain.get_balance(&bob.address));
    println!("矿工余额:   {} sat", blockchain.get_balance(&miner.address));

    // 验证链完整性
    assert!(blockchain.is_valid(), "链验证应通过");
    println!("\n链验证通过！");

    // 打印完整链信息
    blockchain.print_chain();

    Ok(())
}
```

---

## 关键参数参考

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `difficulty` | 3 | 挖矿难度（前导 0 的个数） |
| `mining_reward` | 50 | 区块基础奖励（satoshi） |
| 创世区块奖励 | 10,000,000 | 创世 Coinbase 金额（satoshi） |
| 真实比特币初始奖励 | 5,000,000,000 | 50 BTC（satoshi 单位） |
| 真实比特币减半周期 | 210,000 区块 | 约 4 年 |
| 真实比特币目标出块时间 | 10 分钟 | 约每 2016 块调整一次难度 |
