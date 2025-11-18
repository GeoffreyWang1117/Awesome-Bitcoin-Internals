# Transaction API

交易模块提供了比特币UTXO模型的完整实现。

## 数据结构

### `TxInput`

交易输入，引用之前的未花费输出（UTXO）。

```rust
pub struct TxInput {
    pub txid: String,        // 被引用的交易ID
    pub vout: usize,         // 输出索引
    pub signature: String,   // 数字签名
    pub pub_key: String,     // 公钥
}
```

**方法**:

#### `new`

```rust
pub fn new(
    txid: String,
    vout: usize,
    signature: String,
    pub_key: String
) -> Self
```

创建新的交易输入。

**参数**:
- `txid` - 被引用的交易ID
- `vout` - 输出索引号
- `signature` - 使用私钥生成的签名
- `pub_key` - 对应的公钥

**示例**:
```rust
let input = TxInput::new(
    "abc123...".to_string(),
    0,
    wallet.sign("data"),
    wallet.public_key.clone()
);
```

---

### `TxOutput`

交易输出，代表一笔未花费的金额（UTXO）。

```rust
pub struct TxOutput {
    pub value: u64,              // 金额（satoshi）
    pub pub_key_hash: String,    // 接收者地址
}
```

**方法**:

#### `new`

```rust
pub fn new(value: u64, address: String) -> Self
```

创建新的交易输出。

**参数**:
- `value` - 输出金额（satoshi）
- `address` - 接收者地址

**示例**:
```rust
let output = TxOutput::new(5000, bob_address);
```

#### `can_be_unlocked_with`

```rust
pub fn can_be_unlocked_with(&self, address: &str) -> bool
```

检查是否可以被指定地址解锁。

**参数**:
- `address` - 要检查的地址

**返回值**:
- `true` - 地址匹配
- `false` - 地址不匹配

**示例**:
```rust
if output.can_be_unlocked_with(&alice.address) {
    println!("Alice可以花费这个输出");
}
```

---

### `Transaction`

完整的交易结构。

```rust
pub struct Transaction {
    pub id: String,                 // 交易ID
    pub inputs: Vec<TxInput>,       // 输入列表
    pub outputs: Vec<TxOutput>,     // 输出列表
    pub timestamp: u64,             // Unix时间戳
    pub fee: u64,                   // 手续费
}
```

**方法**:

#### `new`

```rust
pub fn new(
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    timestamp: u64,
    fee: u64
) -> Self
```

创建新交易。

**参数**:
- `inputs` - 交易输入列表
- `outputs` - 交易输出列表
- `timestamp` - Unix时间戳
- `fee` - 手续费（satoshi）

**返回值**:
- 新创建的交易实例，ID已自动计算

**示例**:
```rust
let tx = Transaction::new(
    vec![input1, input2],
    vec![output1, output2],
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    10
);
```

#### `new_coinbase`

```rust
pub fn new_coinbase(
    to: String,
    reward: u64,
    timestamp: u64,
    total_fees: u64
) -> Self
```

创建Coinbase交易（挖矿奖励）。

**参数**:
- `to` - 矿工地址
- `reward` - 区块奖励（不含手续费）
- `timestamp` - Unix时间戳
- `total_fees` - 区块内所有交易的手续费总和

**返回值**:
- Coinbase交易实例

**示例**:
```rust
let coinbase = Transaction::new_coinbase(
    miner.address,
    50,
    timestamp,
    total_fees
);
```

#### `calculate_hash`

```rust
pub fn calculate_hash(&self) -> String
```

计算交易哈希（交易ID）。

**返回值**:
- 64字符的十六进制哈希字符串

**说明**:
- 使用SHA256算法
- 包含所有交易数据（输入、输出、时间戳、手续费）
- 任何数据改变都会导致完全不同的哈希

**示例**:
```rust
let tx_id = tx.calculate_hash();
println!("交易ID: {}", tx_id);
```

#### `is_coinbase`

```rust
pub fn is_coinbase(&self) -> bool
```

检查是否为Coinbase交易。

**返回值**:
- `true` - Coinbase交易
- `false` - 普通交易

**判断标准**:
- 只有一个输入
- 该输入的txid为空

**示例**:
```rust
if tx.is_coinbase() {
    println!("这是挖矿奖励交易");
} else {
    println!("这是普通交易");
}
```

#### `verify`

```rust
pub fn verify(&self) -> bool
```

验证交易有效性（简化版）。

**验证项**:
1. Coinbase交易总是有效
2. 检查是否有输入和输出
3. 检查签名和公钥非空

**返回值**:
- `true` - 交易有效
- `false` - 交易无效

**注意**:
实际比特币还需验证：
- ECDSA签名正确性
- UTXO存在性
- 金额平衡
- 脚本执行

**示例**:
```rust
if tx.verify() {
    blockchain.add_transaction(tx)?;
} else {
    return Err("无效交易".to_string());
}
```

#### `size`

```rust
pub fn size(&self) -> usize
```

计算交易大小（字节）。

**返回值**:
- 交易的字节大小

**用途**:
- 计算手续费率（sat/byte）
- 评估区块空间占用
- 手续费估算

**示例**:
```rust
let size = tx.size();
println!("交易大小: {} 字节", size);
```

#### `fee_rate`

```rust
pub fn fee_rate(&self) -> f64
```

计算交易费率（satoshi/byte）。

**返回值**:
- 费率（sat/byte）

**公式**:
```
fee_rate = fee / size
```

**费率参考**:
- 1-5 sat/byte: 低优先级
- 5-20 sat/byte: 中优先级
- 20-50 sat/byte: 高优先级
- 50+ sat/byte: 紧急

**示例**:
```rust
let rate = tx.fee_rate();
println!("费率: {:.2} sat/byte", rate);

if rate < 5.0 {
    println!("警告：费率较低，确认可能较慢");
}
```

#### `output_sum`

```rust
pub fn output_sum(&self) -> u64
```

获取所有输出的总金额。

**返回值**:
- 输出总额（satoshi）

**用途**:
- 验证交易平衡
- 计算实际手续费

**公式**:
```
fee = input_sum - output_sum
```

**示例**:
```rust
let output_total = tx.output_sum();
let fee = input_total - output_total;
println!("手续费: {}", fee);
```

## 使用示例

### 创建简单交易

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn main() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let alice = Wallet::new();
    let bob = Wallet::new();

    // Alice获得初始资金
    let init_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis".to_string()),
        alice.address.clone(),
        10000,
        0,
    )?;
    blockchain.add_transaction(init_tx)?;
    blockchain.mine_pending_transactions(alice.address.clone())?;

    // Alice向Bob转账
    let tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        3000,  // 金额
        10,    // 手续费
    )?;

    // 查看交易详情
    println!("交易ID: {}", tx.id);
    println!("输入数: {}", tx.inputs.len());
    println!("输出数: {}", tx.outputs.len());
    println!("手续费: {}", tx.fee);
    println!("费率: {:.2} sat/byte", tx.fee_rate());

    // 添加到区块链
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(bob.address)?;

    Ok(())
}
```

### 批量交易

```rust
// 创建多笔交易，测试手续费优先级
let transactions = vec![
    (bob.address.clone(), 1000, 1),   // 低手续费
    (charlie.address.clone(), 2000, 50), // 高手续费
    (david.address.clone(), 3000, 5), // 中等手续费
];

for (to, amount, fee) in transactions {
    let tx = blockchain.create_transaction(&alice, to, amount, fee)?;
    blockchain.add_transaction(tx)?;
}

// 挖矿时会按费率从高到低排序
blockchain.mine_pending_transactions(miner.address)?;
```

### 手动构建交易

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use bitcoin_simulation::transaction::{Transaction, TxInput, TxOutput};

// 1. 创建输入（需要知道之前的UTXO）
let input = TxInput::new(
    "previous_tx_id".to_string(),
    0,  // vout
    alice.sign("tx_data"),
    alice.public_key.clone(),
);

// 2. 创建输出
let output1 = TxOutput::new(3000, bob.address);     // 给Bob
let output2 = TxOutput::new(6990, alice.address);   // 找零

// 3. 组装交易
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

let tx = Transaction::new(
    vec![input],
    vec![output1, output2],
    timestamp,
    10,  // 手续费
);

// 4. 验证和添加
if tx.verify() {
    blockchain.add_transaction(tx)?;
}
```

## 错误处理

```rust
match blockchain.create_transaction(&alice, bob.address, 1000, 10) {
    Ok(tx) => {
        println!("✓ 交易创建成功");
        blockchain.add_transaction(tx)?;
    }
    Err(e) => {
        eprintln!("❌ 交易创建失败: {}", e);
        // 常见错误：
        // - "余额不足（包括手续费）"
        // - "UTXO不存在"
        // - "引用的交易不存在"
    }
}
```

## 最佳实践

### 1. 手续费设置

```rust
// 根据紧急程度设置手续费
let size = estimate_tx_size(inputs_count, outputs_count);

let fee = match urgency {
    Urgency::Low => size * 1,      // 1 sat/byte
    Urgency::Medium => size * 10,  // 10 sat/byte
    Urgency::High => size * 50,    // 50 sat/byte
};
```

### 2. UTXO选择

```rust
// 优先使用小额UTXO，避免碎片化
let utxos = blockchain.utxo_set.find_spendable_outputs(&address, amount)?;
println!("使用了 {} 个UTXO", utxos.1.len());
```

### 3. 交易验证

```rust
// 创建交易后立即验证
let tx = Transaction::new(...);
assert!(tx.verify(), "交易验证失败");
assert!(tx.fee_rate() >= 1.0, "手续费率过低");
```

## 参考

- [Blockchain API](./blockchain.md) - 区块链交易管理
- [UTXO API](./utxo.md) - UTXO集合操作
- [Wallet API](./wallet.md) - 钱包和签名

---

[返回API目录](./core.md)
