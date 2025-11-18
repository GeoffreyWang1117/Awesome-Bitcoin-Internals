# 核心模块 API

SimpleBTC核心模块提供了比特币区块链的基本功能。

## 模块列表

### 交易模块
- [Transaction API](./transaction.md) - 交易结构、UTXO模型、手续费计算

### 区块模块
- [Block API](./block.md) - 区块结构、工作量证明、Merkle根

### 区块链模块
- [Blockchain API](./blockchain.md) - 区块链管理、挖矿、验证

### 钱包模块
- [Wallet API](./wallet.md) - 密钥生成、签名、地址

### UTXO模块
- [UTXO API](./utxo.md) - UTXO集合、余额查询、双花防护

## 快速索引

### 常用函数

#### 创建钱包
```rust
use bitcoin_simulation::wallet::Wallet;
let wallet = Wallet::new();
```

#### 创建交易
```rust
let tx = blockchain.create_transaction(
    &from_wallet,
    to_address,
    amount,
    fee
)?;
```

#### 挖矿
```rust
blockchain.mine_pending_transactions(miner_address)?;
```

#### 查询余额
```rust
let balance = blockchain.get_balance(&address);
```

## 数据流程

```
1. 创建钱包
   Wallet::new() → 生成密钥对 → 得到地址

2. 创建交易
   选择UTXO → 构建输入输出 → 签名 → 验证

3. 添加交易
   验证交易 → 加入待处理池 → 等待打包

4. 挖矿
   收集交易 → 创建Coinbase → 计算Merkle根 → PoW → 更新UTXO

5. 查询
   遍历UTXO集合 → 累加余额
```

## 类型定义

### 核心类型

```rust
// 金额单位：satoshi
type Amount = u64;  // 1 BTC = 100,000,000 satoshi

// 地址：40字符十六进制
type Address = String;

// 哈希：64字符十六进制
type Hash = String;

// Unix时间戳（秒）
type Timestamp = u64;
```

### 错误类型

```rust
// 所有API使用 Result<T, String> 返回
type ApiResult<T> = Result<T, String>;

// 常见错误消息
"余额不足（包括手续费）"
"UTXO不存在"
"交易验证失败"
"引用的交易不存在"
"没有待处理的交易"
```

## 使用模式

### 基础模式

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn main() -> Result<(), String> {
    // 1. 初始化
    let mut blockchain = Blockchain::new();
    let wallet = Wallet::new();

    // 2. 操作
    let tx = blockchain.create_transaction(...)?;
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(...)?;

    // 3. 查询
    let balance = blockchain.get_balance(&wallet.address);

    Ok(())
}
```

### 错误处理模式

```rust
match blockchain.create_transaction(&alice, bob_addr, 1000, 10) {
    Ok(tx) => {
        blockchain.add_transaction(tx)?;
        println!("✓ 交易成功");
    }
    Err(e) => {
        eprintln!("✗ 错误: {}", e);
        // 处理错误...
    }
}
```

## 性能考虑

### UTXO查询
- 时间复杂度：O(n)，n为UTXO总数
- 建议：使用索引优化（见 `indexer.rs`）

### 挖矿
- 时间复杂度：O(2^difficulty)
- 建议：难度3-4适合demo，实际应用需更高

### 区块链验证
- 时间复杂度：O(n*m)，n为区块数，m为平均交易数
- 建议：定期验证，而非每次操作后验证

## 线程安全

⚠️ **注意**：当前实现不是线程安全的。

如需并发访问：

```rust
use std::sync::{Arc, Mutex};

let blockchain = Arc::new(Mutex::new(Blockchain::new()));

// 在不同线程中
let blockchain = blockchain.clone();
let mut bc = blockchain.lock().unwrap();
bc.create_transaction(...)?;
```

## 下一步

- 查看具体模块的详细API文档
- 阅读[高级模块API](./advanced.md)
- 参考[实战案例](../examples/enterprise-multisig.md)

---

[返回文档首页](../introduction/README.md)
