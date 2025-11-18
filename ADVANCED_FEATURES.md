# SimpleBTC - 高级比特币特性

本文档详细介绍SimpleBTC中实现的真实比特币核心特性。

---

## 🌳 1. Merkle 树 (Merkle Tree)

### 什么是Merkle树？

Merkle树是一种哈希树，用于高效验证区块中的交易。比特币中每个区块都包含一个Merkle根，可以快速证明某笔交易是否在区块中。

### 实现特性

- **自动构建**: 每个区块创建时自动构建Merkle树
- **Merkle根**: 存储在区块头中，用于区块哈希计算
- **SPV验证**: 支持生成和验证Merkle证明（轻节点验证）
- **高效存储**: 不需要下载完整区块就能验证交易

### 使用示例

```rust
use bitcoin_simulation::merkle::MerkleTree;

// 创建Merkle树
let tx_ids = vec!["tx1".to_string(), "tx2".to_string(), "tx3".to_string()];
let tree = MerkleTree::new(&tx_ids);

// 获取根哈希
let root = tree.get_root_hash();

// 生成Merkle证明
let proof = tree.get_proof("tx1").unwrap();

// 验证交易是否在区块中
let is_valid = MerkleTree::verify_proof("tx1", &proof, &root, 0);
assert!(is_valid);
```

### 优势

- ✅ **轻节点支持**: SPV钱包可以验证交易而不下载完整区块
- ✅ **高效验证**: O(log n) 复杂度验证交易
- ✅ **节省空间**: 区块头只需32字节存储Merkle根

---

## 🔐 2. 多重签名 (Multisig)

### 什么是多重签名？

多重签名要求多个私钥签署交易才能花费资金，常用于企业钱包、托管服务、安全备份。

### 实现特性

- **m-of-n 多签**: 支持任意配置（如2-of-3, 3-of-5）
- **预设类型**: 提供常用配置（2-of-2, 2-of-3, 3-of-5）
- **地址生成**: 多签地址以"3"开头（与比特币相同）
- **签名验证**: 自动验证签名数量

### 使用示例

```rust
use bitcoin_simulation::{wallet::Wallet, multisig::*};

// 创建3个钱包
let wallet1 = Wallet::new();
let wallet2 = Wallet::new();
let wallet3 = Wallet::new();

// 创建2-of-3多签地址
let public_keys = vec![
    wallet1.public_key.clone(),
    wallet2.public_key.clone(),
    wallet3.public_key.clone(),
];

let multisig = MultiSigAddress::new(2, public_keys).unwrap();
println!("多签地址: {}", multisig.address);

// 构建交易并收集签名
let mut builder = MultiSigTxBuilder::new(multisig);
builder.add_signature(&wallet1, "transaction_data").unwrap();
builder.add_signature(&wallet2, "transaction_data").unwrap();

assert!(builder.is_complete()); // 2个签名已收集
```

### 应用场景

- 💼 **企业钱包**: 需要多人批准才能转账
- 🏦 **托管服务**: 买卖双方 + 仲裁者的2-of-3安排
- 🔒 **安全备份**: 将密钥分散存储，防止单点故障

---

## 🔄 3. Replace-By-Fee (RBF)

### 什么是RBF？

RBF允许用户通过支付更高手续费来替换未确认的交易，用于加速交易或修改交易内容。

### 实现特性

- **可替换标记**: 交易可以标记为支持RBF
- **手续费验证**: 新交易必须支付更高手续费
- **输入验证**: 必须花费相同的UTXO
- **最小增量**: 手续费增量必须足够

### 使用示例

```rust
use bitcoin_simulation::advanced_tx::*;

let mut rbf = RBFManager::new();

// 标记交易为可替换
rbf.mark_replaceable("tx_original");

// 验证是否可以替换
let can_replace = rbf.can_replace(&old_tx, &new_tx);

match can_replace {
    Ok(_) => println!("可以替换！"),
    Err(e) => println!("无法替换: {}", e),
}
```

### 替换规则

1. ✅ 原交易必须支持RBF（未确认状态）
2. ✅ 新交易必须花费相同的输入（UTXO）
3. ✅ 新交易手续费必须更高
4. ✅ 手续费增量至少为交易大小（sat）

### 应用场景

- ⚡ **加速交易**: 手续费设置过低时提高费率
- 📝 **修改收款人**: 在确认前更改接收地址
- 💰 **动态费率**: 根据网络拥堵调整手续费

---

## ⏰ 4. 时间锁 (Timelock)

### 什么是时间锁？

时间锁限制交易只能在特定时间或区块高度之后使用，用于延迟支付、智能合约等场景。

### 实现特性

- **时间锁定**: 基于Unix时间戳
- **区块锁定**: 基于区块高度
- **成熟度检查**: 自动判断是否可用
- **剩余时间**: 计算还需等待的时间/区块数

### 使用示例

```rust
use bitcoin_simulation::advanced_tx::*;

// 创建基于时间的锁（2小时后可用）
let unlock_time = current_time + 7200; // 2小时
let timelock = TimeLock::new_time_based(unlock_time);

// 检查是否可用
if timelock.is_mature(current_time, current_height) {
    println!("时间锁已到期，可以使用");
} else {
    let remaining = timelock.remaining(current_time, current_height);
    println!("还需等待 {} 秒", remaining);
}

// 创建基于区块高度的锁
let block_lock = TimeLock::new_height_based(100);
```

### 应用场景

- 📅 **定时支付**: 工资在特定日期自动发放
- 🔒 **托管**: 资金在一定时间后自动释放
- 💍 **遗产规划**: 长时间未使用后转移资金
- ⚖️ **纠纷解决**: 给予争议解决时间窗口

---

## 📊 5. 交易优先级算法

### 智能优先级计算

SimpleBTC实现了多维度的交易优先级系统，确保重要交易优先处理。

### 实现特性

- **费率计算**: sat/byte 标准
- **年龄权重**: 输入的确认数影响优先级
- **综合评分**: 费率(70%) + 优先级(30%)
- **动态手续费推荐**: 根据紧急程度推荐费率

### 计算公式

```
费率优先级 = 手续费 / 交易大小
传统优先级 = (输入价值 × 输入年龄) / 交易大小
综合评分 = 费率 × 0.7 + 优先级 × 0.001 × 0.3
```

### 使用示例

```rust
use bitcoin_simulation::advanced_tx::*;

// 计算费率
let fee_rate = TxPriorityCalculator::calculate_fee_rate(500, 100);
// 结果: 5.0 sat/byte

// 计算传统优先级
let priority = TxPriorityCalculator::calculate_priority(
    10000,  // 输入价值
    10,     // 确认区块数
    100     // 交易大小
);

// 综合评分
let score = TxPriorityCalculator::calculate_score(fee_rate, priority);

// 推荐手续费
let recommended = TxPriorityCalculator::recommend_fee(250, FeeUrgency::High);
println!("推荐手续费: {} satoshi", recommended);
```

### 手续费等级

| 等级 | 费率 | 预计确认时间 |
|------|------|--------------|
| Low (低) | 1 sat/byte | 几小时 |
| Medium (中) | 5 sat/byte | 30-60分钟 |
| High (高) | 20 sat/byte | 10-20分钟 |
| Urgent (紧急) | 50 sat/byte | 下一个区块 |

---

## 🛠️ 6. 高级交易构建器

### 灵活的交易构建

```rust
use bitcoin_simulation::advanced_tx::*;

// 创建支持RBF的交易
let builder = AdvancedTxBuilder::new()
    .with_rbf();

// 创建带时间锁的交易
let builder = AdvancedTxBuilder::new()
    .with_timelock(TimeLock::new_time_based(future_time));

// 检查配置
if builder.supports_rbf() {
    println!("交易支持RBF");
}
```

---

## 📈 实际应用场景

### 场景1: 企业多签钱包

```rust
// 公司财务需要CEO + CFO批准
let ceo_wallet = Wallet::new();
let cfo_wallet = Wallet::new();

let company_wallet = MultiSigType::TwoOfTwo
    .create_address(&[ceo_wallet, cfo_wallet])
    .unwrap();

// 转账需要两人签名
```

### 场景2: 拥堵时加速交易

```rust
// 原交易手续费太低
let original_fee = 100; // 1 sat/byte

// 使用RBF提高手续费
let new_fee = TxPriorityCalculator::recommend_fee(100, FeeUrgency::Urgent);

// 创建替换交易
if rbf.can_replace(&old_tx, &new_tx).is_ok() {
    // 广播新交易
}
```

### 场景3: 智能合约 - 原子交换

```rust
// Alice和Bob的跨链原子交换
let timelock_alice = TimeLock::new_height_based(current_height + 100);
let timelock_bob = TimeLock::new_height_based(current_height + 50);

// Bob的退款交易有更长的时间锁
// 确保Alice先能退款，防止双重支付
```

---

## 🔬 技术对比

### 与真实比特币的对比

| 特性 | SimpleBTC | 真实比特币 |
|------|-----------|------------|
| Merkle树 | ✅ 完整实现 | ✅ |
| 多重签名 | ✅ M-of-N | ✅ |
| RBF | ✅ BIP125简化版 | ✅ BIP125 |
| 时间锁 | ✅ nLockTime | ✅ nLockTime + CheckLockTimeVerify |
| 优先级 | ✅ 费率优先 | ✅ |
| Script | ⚠️ 简化 | ✅ 完整脚本 |
| SegWit | ❌ 未实现 | ✅ |
| Taproot | ❌ 未实现 | ✅ |

---

## 📚 学习资源

1. **Merkle树**
   - [Bitcoin Wiki: Merkle Tree](https://en.bitcoin.it/wiki/Merkle_tree)
   - SPV验证原理

2. **多重签名**
   - [BIP11: M-of-N Standard Transactions](https://github.com/bitcoin/bips/blob/master/bip-0011.mediawiki)
   - 安全最佳实践

3. **Replace-By-Fee**
   - [BIP125: Opt-in RBF](https://github.com/bitcoin/bips/blob/master/bip-0125.mediawiki)
   - 费用估算策略

4. **时间锁**
   - [BIP65: CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
   - 应用场景解析

---

## 🎯 下一步扩展

考虑添加的特性：

1. **SegWit (隔离见证)**
   - 修复交易延展性
   - 提高区块容量

2. **Script系统**
   - P2SH (Pay-to-Script-Hash)
   - 条件支付脚本

3. **闪电网络**
   - 第二层支付通道
   - 即时微支付

4. **Taproot**
   - 隐私改进
   - 更复杂的智能合约

---

**SimpleBTC - 学习真实比特币的最佳实践平台！** 🚀
