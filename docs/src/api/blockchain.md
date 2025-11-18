# Blockchain API

区块链模块是SimpleBTC的核心，管理整个区块链的状态和操作。

## 数据结构

### `Blockchain`

```rust
pub struct Blockchain {
    pub chain: Vec<Block>,                      // 区块链（区块列表）
    pub difficulty: usize,                      // 挖矿难度
    pub pending_transactions: Vec<Transaction>, // 待处理交易池
    pub utxo_set: UTXOSet,                     // UTXO集合
    pub mining_reward: u64,                    // 挖矿奖励
    pub indexer: TransactionIndexer,           // 交易索引器
}
```

## 方法

### 初始化

#### `new`

```rust
pub fn new() -> Blockchain
```

创建新的区块链，自动创建创世区块。

**初始参数**:
- `difficulty: 3` - 挖矿难度（3个前导0）
- `mining_reward: 50` - 区块奖励（50 satoshi）
- 创世区块包含100 satoshi发送给`genesis_address`

**返回值**: 新的区块链实例

**示例**:
```rust
let mut blockchain = Blockchain::new();
println!("区块链已初始化，当前高度: {}", blockchain.chain.len());
```

---

### 交易管理

#### `create_transaction`

```rust
pub fn create_transaction(
    &self,
    from_wallet: &Wallet,
    to_address: String,
    amount: u64,
    fee: u64,
) -> Result<Transaction, String>
```

创建新交易。自动选择UTXO、构建输入输出、添加签名。

**参数**:
- `from_wallet` - 发送者钱包（需要私钥签名）
- `to_address` - 接收者地址
- `amount` - 转账金额（satoshi）
- `fee` - 手续费（satoshi）

**返回值**:
- `Ok(Transaction)` - 交易创建成功
- `Err(String)` - 错误信息

**错误情况**:
- `"余额不足（包括手续费）"` - 没有足够的UTXO
- `"UTXO不存在"` - 引用的UTXO已被花费
- `"引用的交易不存在"` - 数据不一致

**工作流程**:
1. 查找发送者的可用UTXO
2. 选择足够的UTXO（贪心算法）
3. 创建交易输入（包含签名）
4. 创建交易输出（接收者 + 找零）
5. 计算交易ID

**示例**:
```rust
// 基本用法
let tx = blockchain.create_transaction(
    &alice,
    bob.address.clone(),
    5000,  // 转5000 satoshi
    10,    // 手续费10 satoshi
)?;

blockchain.add_transaction(tx)?;

// 检查余额
let balance = blockchain.get_balance(&alice.address);
if balance < amount + fee {
    return Err("余额不足".to_string());
}

// 批量创建
for i in 1..=10 {
    let tx = blockchain.create_transaction(
        &alice,
        recipients[i].clone(),
        1000,
        i as u64,  // 不同的手续费
    )?;
    blockchain.add_transaction(tx)?;
}
```

#### `add_transaction`

```rust
pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String>
```

将交易添加到待处理池，等待被打包。

**参数**:
- `transaction` - 要添加的交易

**验证项**:
1. ✅ 交易格式正确（verify()）
2. ✅ 输入引用的UTXO存在
3. ✅ 签名有效
4. ✅ 输入总额 ≥ 输出总额

**返回值**:
- `Ok(())` - 添加成功
- `Err(String)` - 验证失败原因

**示例**:
```rust
let tx = blockchain.create_transaction(&alice, bob.address, 1000, 5)?;

match blockchain.add_transaction(tx) {
    Ok(_) => println!("✓ 交易已添加到待处理池"),
    Err(e) => eprintln!("✗ 交易无效: {}", e),
}

// 查看待处理交易数量
println!("待处理: {} 笔", blockchain.pending_transactions.len());
```

---

### 挖矿

#### `mine_pending_transactions`

```rust
pub fn mine_pending_transactions(
    &mut self,
    miner_address: String
) -> Result<(), String>
```

挖矿：将待处理交易打包成新区块。

**参数**:
- `miner_address` - 矿工地址（接收奖励）

**挖矿流程**:
1. 检查是否有待处理交易
2. 按手续费率从高到低排序
3. 计算总手续费
4. 创建Coinbase交易（奖励 + 手续费）
5. 构建Merkle树
6. 工作量证明（调整nonce找到有效哈希）
7. 验证区块中所有交易
8. 更新UTXO集合（原子操作）
9. 将区块添加到链上
10. 清空待处理池

**返回值**:
- `Ok(())` - 挖矿成功
- `Err(String)` - 错误信息

**错误情况**:
- `"没有待处理的交易"` - 待处理池为空
- `"区块包含无效交易"` - 交易验证失败
- `"UTXO更新失败"` - 数据不一致

**性能**:
- 难度3: 约0.001-0.1秒
- 难度4: 约0.01-1秒
- 难度5: 约0.1-10秒
- 难度6+: 数秒到数分钟

**示例**:
```rust
// 基本挖矿
blockchain.mine_pending_transactions(miner.address.clone())?;

// 挖矿循环（类似真实矿工）
loop {
    if blockchain.pending_transactions.is_empty() {
        println!("等待新交易...");
        std::thread::sleep(Duration::from_secs(1));
        continue;
    }

    println!("开始挖矿...");
    let start = Instant::now();

    blockchain.mine_pending_transactions(miner.address.clone())?;

    let duration = start.elapsed();
    println!("✓ 挖矿成功! 耗时: {:?}", duration);

    // 查看奖励
    let reward = blockchain.get_balance(&miner.address);
    println!("矿工余额: {} satoshi", reward);
}
```

---

### 查询操作

#### `get_balance`

```rust
pub fn get_balance(&self, address: &str) -> u64
```

查询地址余额。

**参数**:
- `address` - 要查询的地址

**返回值**: 余额（satoshi）

**计算方式**: 遍历UTXO集合，累加该地址的所有UTXO

**示例**:
```rust
let balance = blockchain.get_balance(&alice.address);
println!("余额: {} satoshi", balance);
println!("余额: {:.8} BTC", balance as f64 / 100_000_000.0);

// 批量查询
let addresses = vec![alice.address, bob.address, charlie.address];
for addr in addresses {
    let bal = blockchain.get_balance(&addr);
    println!("{}: {}", &addr[..10], bal);
}
```

#### `is_valid`

```rust
pub fn is_valid(&self) -> bool
```

验证整个区块链的完整性。

**验证项**:
1. ✅ 每个区块的哈希正确
2. ✅ 前向引用正确（previous_hash链接）
3. ✅ 工作量证明有效（哈希满足难度）
4. ✅ 所有交易有效

**返回值**:
- `true` - 区块链完整有效
- `false` - 发现篡改或错误

**用途**:
- 定期完整性检查
- 同步节点后验证
- 检测篡改攻击

**示例**:
```rust
// 定期验证
if !blockchain.is_valid() {
    panic!("❌ 区块链已被篡改！");
}

// 详细验证日志
for (i, block) in blockchain.chain.iter().enumerate() {
    if block.hash != block.calculate_hash() {
        eprintln!("区块 {} 哈希无效", i);
    }
    if !block.validate_transactions() {
        eprintln!("区块 {} 包含无效交易", i);
    }
}

if blockchain.is_valid() {
    println!("✅ 区块链验证通过");
}
```

#### `print_chain`

```rust
pub fn print_chain(&self)
```

打印区块链详细信息（调试用）。

**输出内容**:
- 区块索引、时间戳、哈希
- 前一个区块哈希
- Nonce值
- 交易列表（ID、类型、手续费、输入输出）

**示例**:
```rust
blockchain.print_chain();

// 输出示例:
// ========== 区块链信息 ==========
//
// --- 区块 #0 ---
// 时间戳: 1703001234
// 哈希: 0003ab4f9c2d...
// 前一个哈希: 0
// Nonce: 1247
// 交易数量: 1
//   交易 #0: abc123...
//     类型: Coinbase（挖矿奖励）
//     输入数: 1
//     输出数: 1
//       输出 0: 100 -> genesis_address
// ...
```

---

## 使用示例

### 完整的区块链演示

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn main() -> Result<(), String> {
    println!("=== SimpleBTC区块链演示 ===\n");

    // 1. 初始化
    let mut blockchain = Blockchain::new();
    println!("✓ 区块链已创建（创世区块）\n");

    // 2. 创建参与者
    let alice = Wallet::new();
    let bob = Wallet::new();
    let miner = Wallet::new();

    println!("✓ 创建了3个钱包");
    println!("  Alice: {}", &alice.address[..16]);
    println!("  Bob:   {}", &bob.address[..16]);
    println!("  Miner: {}\n", &miner.address[..16]);

    // 3. Alice获得初始资金
    let init_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        alice.address.clone(),
        10000,
        0,
    )?;
    blockchain.add_transaction(init_tx)?;
    blockchain.mine_pending_transactions(miner.address.clone())?;

    println!("✓ 区块 #1 已挖出");
    println!("  Alice余额: {}\n", blockchain.get_balance(&alice.address));

    // 4. 多笔交易
    println!("创建5笔交易（不同手续费）...");
    for i in 1..=5 {
        let tx = blockchain.create_transaction(
            &alice,
            bob.address.clone(),
            100 * i,
            i as u64,  // 手续费递增
        )?;
        blockchain.add_transaction(tx)?;
        println!("  交易 #{}: {} sat, 费率: {} sat/byte",
            i, 100 * i, i);
    }

    // 5. 挖矿（交易按费率排序）
    println!("\n开始挖矿...");
    blockchain.mine_pending_transactions(miner.address.clone())?;
    println!("✓ 区块 #2 已挖出\n");

    // 6. 最终余额
    println!("=== 最终余额 ===");
    println!("Alice: {} satoshi", blockchain.get_balance(&alice.address));
    println!("Bob:   {} satoshi", blockchain.get_balance(&bob.address));
    println!("Miner: {} satoshi", blockchain.get_balance(&miner.address));

    // 7. 验证区块链
    println!("\n=== 验证区块链 ===");
    if blockchain.is_valid() {
        println!("✅ 区块链完整性验证通过");
    } else {
        println!("❌ 区块链验证失败");
    }

    // 8. 打印详细信息
    println!("\n=== 区块链详情 ===");
    blockchain.print_chain();

    Ok(())
}
```

### 手续费优先级演示

```rust
fn fee_priority_demo() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let alice = Wallet::new();
    let recipients: Vec<_> = (0..3).map(|_| Wallet::new()).collect();

    // 初始化Alice余额
    setup_balance(&mut blockchain, &alice, 10000)?;

    // 创建不同费率的交易
    let txs = vec![
        ("慢速", 1000, 1),   // 1 sat/byte
        ("快速", 1000, 50),  // 50 sat/byte
        ("中速", 1000, 10),  // 10 sat/byte
    ];

    println!("添加交易:");
    for (i, (name, amount, fee)) in txs.iter().enumerate() {
        let tx = blockchain.create_transaction(
            &alice,
            recipients[i].address.clone(),
            *amount,
            *fee,
        )?;
        println!("  {}: {} sat, 费率 {} sat/byte", name, amount, fee);
        blockchain.add_transaction(tx)?;
    }

    println!("\n挖矿（自动按费率排序）...");
    blockchain.mine_pending_transactions(recipients[0].address.clone())?;

    // 查看最新区块的交易顺序
    let latest_block = blockchain.chain.last().unwrap();
    println!("\n区块中的交易顺序:");
    for (i, tx) in latest_block.transactions.iter().skip(1).enumerate() {
        println!("  #{}: 费率 {:.2} sat/byte",
            i + 1, tx.fee_rate());
    }

    Ok(())
}
```

### 监控区块链状态

```rust
fn blockchain_monitor(blockchain: &Blockchain) {
    println!("=== 区块链状态 ===");
    println!("区块高度: {}", blockchain.chain.len());
    println!("难度: {} ({}个前导0)",
        blockchain.difficulty, blockchain.difficulty);
    println!("挖矿奖励: {} satoshi", blockchain.mining_reward);
    println!("待处理交易: {} 笔", blockchain.pending_transactions.len());

    // UTXO统计
    let total_utxos = blockchain.utxo_set.utxos
        .values()
        .map(|v| v.len())
        .sum::<usize>();
    println!("UTXO总数: {}", total_utxos);

    // 最新区块信息
    if let Some(latest) = blockchain.chain.last() {
        println!("\n最新区块:");
        println!("  哈希: {}", &latest.hash[..16]);
        println!("  交易数: {}", latest.transactions.len());
        println!("  Merkle根: {}", &latest.merkle_root[..16]);
    }
}
```

## 配置建议

### 挖矿难度

```rust
// 演示环境
blockchain.difficulty = 3;  // 快速（毫秒级）

// 测试环境
blockchain.difficulty = 4;  // 适中（秒级）

// 生产环境
blockchain.difficulty = 6;  // 安全（分钟级）
```

### 区块奖励

```rust
// 比特币风格（逐步减半）
let halving_interval = 210000;
let halvings = blockchain.chain.len() / halving_interval;
blockchain.mining_reward = 50 >> halvings;  // 50, 25, 12.5, ...
```

## 性能优化

### 1. UTXO索引

使用`indexer`加速查询：

```rust
// 查找地址的所有交易
let txs = blockchain.indexer.get_transactions_by_address(&address);

// 查找特定交易
let tx = blockchain.indexer.get_transaction(&txid);
```

### 2. 批量操作

```rust
// 批量添加交易
for tx in transactions {
    blockchain.add_transaction(tx)?;
}
// 一次性挖矿
blockchain.mine_pending_transactions(miner.address)?;
```

### 3. 并行验证

```rust
use rayon::prelude::*;

// 并行验证所有交易（需要添加rayon依赖）
let all_valid = blockchain.pending_transactions
    .par_iter()
    .all(|tx| tx.verify());
```

## 参考

- [Transaction API](./transaction.md)
- [Block API](./block.md)
- [UTXO API](./utxo.md)
- [实战案例](../examples/enterprise-multisig.md)

---

[返回API目录](./core.md)
