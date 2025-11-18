# 快速入门

这是一个5分钟的快速教程，带您体验SimpleBTC的核心功能。

## 第一个区块链程序

创建一个新的Rust项目并添加SimpleBTC依赖：

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn main() {
    println!("🚀 SimpleBTC 快速入门\n");

    // 1. 创建区块链
    let mut blockchain = Blockchain::new();
    println!("✓ 区块链已初始化");

    // 2. 创建钱包
    let alice = Wallet::new();
    let bob = Wallet::new();
    println!("✓ 创建了两个钱包");
    println!("  Alice: {}", alice.address);
    println!("  Bob:   {}\n", bob.address);

    // 3. Alice获得初始资金（从创世区块）
    let tx1 = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        alice.address.clone(),
        10000,  // 1万 satoshi
        0,      // 无手续费（创世交易）
    ).unwrap();
    blockchain.add_transaction(tx1).unwrap();
    blockchain.mine_pending_transactions(alice.address.clone()).unwrap();

    println!("💰 Alice的余额: {} satoshi", blockchain.get_balance(&alice.address));

    // 4. Alice向Bob转账
    let tx2 = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        3000,   // 转账3000
        10,     // 手续费10
    ).unwrap();
    blockchain.add_transaction(tx2).unwrap();
    blockchain.mine_pending_transactions(bob.address.clone()).unwrap();

    // 5. 查看最终余额
    println!("\n💼 最终余额:");
    println!("  Alice: {} satoshi", blockchain.get_balance(&alice.address));
    println!("  Bob:   {} satoshi\n", blockchain.get_balance(&bob.address));

    // 6. 验证区块链
    if blockchain.is_valid() {
        println!("✅ 区块链验证通过！");
    }

    // 7. 打印区块链信息
    blockchain.print_chain();
}
```

## 运行结果

```
🚀 SimpleBTC 快速入门

✓ 区块链已初始化
✓ 创建了两个钱包
  Alice: a3f2d8c9e4b7...
  Bob:   b9e4c7d2a3f1...

区块已挖出: 0003ab4f9c2d...
💰 Alice的余额: 10050 satoshi

区块已挖出: 0007c3e8d1a9...

💼 最终余额:
  Alice: 6990 satoshi
  Bob:   3060 satoshi

✅ 区块链验证通过！
```

## 核心概念速览

### 1. 区块链（Blockchain）

区块链是区块的链式数据结构，每个区块包含多笔交易。

```rust
let mut blockchain = Blockchain::new();
```

**关键方法**:
- `create_transaction()` - 创建交易
- `add_transaction()` - 添加到待处理池
- `mine_pending_transactions()` - 挖矿打包交易
- `get_balance()` - 查询余额
- `is_valid()` - 验证区块链

### 2. 钱包（Wallet）

钱包管理公钥、私钥和地址。

```rust
let wallet = Wallet::new();
println!("地址: {}", wallet.address);
println!("公钥: {}", wallet.public_key);
// 私钥应保密！
```

**关键方法**:
- `new()` - 创建新钱包
- `sign()` - 签名数据
- `verify_signature()` - 验证签名

### 3. 交易（Transaction）

交易是价值转移的基本单位，使用UTXO模型。

```rust
let tx = blockchain.create_transaction(
    &sender,         // 发送者钱包
    receiver_addr,   // 接收者地址
    amount,          // 金额（satoshi）
    fee,             // 手续费（satoshi）
)?;
```

**交易包含**:
- 输入（Inputs）: 花费的UTXO
- 输出（Outputs）: 创建的新UTXO
- 手续费: 输入总额 - 输出总额

### 4. 挖矿（Mining）

挖矿是通过工作量证明（PoW）将交易打包成区块。

```rust
blockchain.mine_pending_transactions(miner_address)?;
```

**挖矿过程**:
1. 收集待处理交易
2. 创建Coinbase交易（奖励+手续费）
3. 计算Merkle根
4. 找到满足难度的哈希（调整nonce）
5. 将区块添加到链上
6. 更新UTXO集合

## 进阶示例

### 多笔交易

```rust
// 创建多笔交易
for i in 1..=5 {
    let tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        100 * i,
        i,  // 不同的手续费
    )?;
    blockchain.add_transaction(tx)?;
}

// 一次性打包所有交易
blockchain.mine_pending_transactions(miner.address.clone())?;
```

### 交易费率优先级

```rust
// 低手续费交易
let slow_tx = blockchain.create_transaction(&alice, bob.address.clone(), 1000, 1)?;

// 高手续费交易
let fast_tx = blockchain.create_transaction(&alice, charlie.address.clone(), 1000, 50)?;

blockchain.add_transaction(slow_tx)?;
blockchain.add_transaction(fast_tx)?;

// 矿工会优先打包fast_tx（更高费率）
blockchain.mine_pending_transactions(miner.address)?;
```

### 余额查询

```rust
let balance = blockchain.get_balance(&alice.address);
println!("余额: {} satoshi ({:.8} BTC)", balance, balance as f64 / 100_000_000.0);
```

## REST API 使用

启动API服务器：

```bash
cargo run --bin btc-server
```

### 创建钱包

```bash
curl -X POST http://localhost:3000/api/wallet/create
```

响应：
```json
{
  "address": "a3f2d8c9e4b7...",
  "public_key": "04f9a...",
  "private_key": "私钥请妥善保管"
}
```

### 创建交易

```bash
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice_address",
    "to": "bob_address",
    "amount": 5000,
    "fee": 10
  }'
```

### 查询余额

```bash
curl http://localhost:3000/api/balance/alice_address
```

### 挖矿

```bash
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{
    "miner_address": "miner_address"
  }'
```

### 查看区块链信息

```bash
curl http://localhost:3000/api/blockchain/info
```

响应：
```json
{
  "chain_length": 3,
  "difficulty": 3,
  "pending_transactions": 2,
  "latest_block": {
    "index": 2,
    "hash": "0003ab4f...",
    "timestamp": 1703001234,
    "transaction_count": 5
  }
}
```

## Electron GUI

启动图形界面：

```bash
cd frontend
npm install
npm start
```

GUI功能：
- 📊 **区块链浏览器**: 可视化查看所有区块
- 👛 **钱包管理**: 创建、导入钱包
- 💸 **发送交易**: 图形化创建交易
- ⛏️ **挖矿**: 点击按钮开始挖矿
- 🎮 **Demo模式**: 一键运行完整演示

## 实战案例

SimpleBTC提供了三个完整的实战案例：

### 1. 企业多签钱包（2-of-3）

```bash
cargo run --example enterprise_multisig
```

学习内容：
- 创建多签地址
- 收集签名
- 企业资金管理

### 2. 托管服务

```bash
cargo run --example escrow_service
```

学习内容：
- 买卖双方交易
- 仲裁机制
- 争议解决

### 3. 定期存款

```bash
cargo run --example timelock_savings
```

学习内容：
- 时间锁设置
- 到期检查
- 强制储蓄

## 下一步学习

现在您已经掌握了基础使用，可以继续深入学习：

1. **理解原理** - [基本概念](./concepts.md)
   - UTXO模型详解
   - 工作量证明原理
   - Merkle树结构

2. **核心功能** - [核心模块指南](./wallet.md)
   - 钱包深入使用
   - 交易高级特性
   - 区块链操作

3. **高级特性** - [高级功能](../advanced/merkle.md)
   - Merkle树与SPV
   - 多重签名
   - Replace-By-Fee
   - 时间锁

4. **API文档** - [API参考](../api/core.md)
   - 完整的API文档
   - 函数签名
   - 使用示例

## 小贴士

💡 **提示**:
- 挖矿难度3-4适合演示，6+更接近真实
- 手续费越高，交易越快被确认
- 定期调用`is_valid()`验证区块链完整性
- 使用`print_chain()`查看详细信息

⚠️ **注意**:
- 私钥一旦丢失无法恢复
- 本项目仅用于学习，不要用于生产
- 简化的密码学实现不如真实比特币安全

---

准备好深入探索了吗？继续阅读[基本概念](./concepts.md)！
