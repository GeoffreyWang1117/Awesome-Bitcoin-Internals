# SimpleBTC - 基于BTC的银行系统Demo

一个完整的基于比特币（BTC）原理的银行系统演示项目，使用Rust实现，符合事务处理的ACID特性。

## 项目特性

### 核心功能

- **UTXO模型**：采用比特币的未花费交易输出（UTXO）模型
- **区块链技术**：完整的区块链实现，包括创世区块、挖矿、区块验证
- **钱包系统**：地址生成、密钥管理、交易签名
- **交易处理**：支持转账、余额查询、交易验证
- **工作量证明**：POW挖矿机制，可配置难度

### 事务处理特性（ACID）

✅ **原子性 (Atomicity)**
- 交易要么全部执行，要么全部不执行
- 余额不足时，交易被完全拒绝，不会部分执行

✅ **一致性 (Consistency)**
- 交易前后，所有账户余额总和保持一致
- 每笔交易都经过验证，确保输入 ≥ 输出
- 区块链完整性验证机制

✅ **隔离性 (Isolation)**
- 交易在待处理池中等待，不影响当前状态
- 只有在挖矿成功后，交易才会被确认并更新UTXO集合

✅ **持久性 (Durability)**
- 一旦交易被打包进区块并挖矿成功
- 交易记录永久保存在区块链中，不可篡改

## 项目结构

```
SimpleBTC/
├── src/
│   ├── main.rs         # 主程序和演示代码
│   ├── block.rs        # 区块结构和实现
│   ├── blockchain.rs   # 区块链核心逻辑
│   ├── transaction.rs  # 交易和UTXO模型
│   ├── wallet.rs       # 钱包和密钥管理
│   └── utxo.rs         # UTXO集合管理
├── Cargo.toml          # 项目配置和依赖
└── README.md           # 项目文档
```

## 技术实现

### 1. UTXO模型

```rust
// 交易输入 - 引用之前的交易输出
pub struct TxInput {
    pub txid: String,       // 被引用的交易ID
    pub vout: usize,        // 输出索引
    pub signature: String,  // 签名
    pub pub_key: String,    // 公钥
}

// 交易输出 - 未花费的交易输出
pub struct TxOutput {
    pub value: u64,         // 金额
    pub pub_key_hash: String, // 接收者地址
}
```

### 2. 交易验证流程

1. 验证交易格式和签名
2. 检查引用的UTXO是否存在
3. 验证输入总额 ≥ 输出总额
4. 添加到待处理交易池
5. 挖矿打包后更新UTXO集合

### 3. 挖矿机制

- 使用SHA256哈希算法
- 工作量证明（POW）
- 可配置的挖矿难度
- Coinbase交易作为挖矿奖励

## 快速开始

### 环境要求

- Rust 1.70 或更高版本
- Cargo 包管理器

### 编译和运行

```bash
# 编译项目
cargo build --release

# 运行演示
cargo run --release
```

### 运行结果

程序将演示以下场景：

1. 初始化区块链和创世区块
2. 创建用户钱包（Alice, Bob, Charlie）
3. 为用户发放初始余额
4. 执行转账交易
5. 挖矿打包交易
6. 查询账户余额
7. 测试余额不足情况
8. 验证区块链完整性
9. 展示完整的区块链信息
10. 演示ACID事务特性

## 演示输出示例

```
========================================
   SimpleBTC - 基于BTC的银行系统演示
========================================

>>> 步骤 1: 初始化区块链
✓ 区块链已创建，创世区块已生成

>>> 步骤 2: 创建用户钱包
✓ Alice 的钱包地址: fa43ca15f5b3f715edbed52327a385643bfd4b2d
✓ Bob 的钱包地址: 7e8f1f8cc81f471923e90c106d3bd3169b54703f
✓ Charlie 的钱包地址: 88b461d25f7bf373d183a5bb151c2e5f76f1d538

>>> 步骤 5: Alice 向 Bob 转账 30 BTC
✓ 交易已创建
✓ 交易已验证并添加到待处理池

>>> 步骤 9: 测试余额不足情况
✓ 正确拒绝: 余额不足

>>> 步骤 10: 验证区块链完整性
✓ 区块链验证通过，所有区块和交易都有效
```

## 核心API

### 创建区块链

```rust
let mut blockchain = Blockchain::new();
```

### 创建钱包

```rust
let wallet = Wallet::new();
println!("地址: {}", wallet.address);
```

### 创建交易

```rust
let tx = blockchain.create_transaction(
    &from_wallet,
    to_address,
    amount
)?;
```

### 添加交易

```rust
blockchain.add_transaction(tx)?;
```

### 挖矿

```rust
blockchain.mine_pending_transactions(miner_address)?;
```

### 查询余额

```rust
let balance = blockchain.get_balance(&address);
```

### 验证区块链

```rust
let is_valid = blockchain.is_valid();
```

## 技术栈

- **Rust**: 系统编程语言
- **SHA2**: 哈希算法
- **Serde**: 序列化/反序列化
- **MD5**: 辅助哈希（用于简化演示）

## 学习要点

1. **区块链基础**：理解区块、链、共识机制
2. **UTXO模型**：比特币的核心数据结构
3. **交易处理**：如何构建和验证交易
4. **工作量证明**：POW挖矿原理
5. **数字签名**：简化的签名验证机制
6. **ACID特性**：数据库事务处理原则在区块链中的应用

## 注意事项

⚠️ **这是一个教育演示项目，不适用于生产环境**

- 使用简化的密钥生成和签名机制
- 没有实现网络通信和节点同步
- 未实现持久化存储
- 挖矿难度较低，仅用于演示

## 扩展建议

如果要进一步完善此项目，可以考虑：

1. 实现真正的椭圆曲线数字签名算法（ECDSA）
2. 添加P2P网络通信
3. 实现数据持久化（SQLite/RocksDB）
4. 添加交易费用机制
5. 实现Merkle树优化交易验证
6. 添加智能合约功能
7. 实现更多共识算法（POS、DPOS等）

## 许可证

MIT License

## 作者

SimpleBTC Banking System Demo

---

**Happy Coding! 🚀**
