# Replace-By-Fee (RBF) 机制

RBF（替换手续费）是BIP125提出的机制，允许用户替换未确认的交易。

## 概述

### 什么是RBF？

RBF允许发送者在交易未确认前，用更高手续费的交易替换原交易。

**场景**:
```
1. Alice发送交易A：1 BTC → Bob，手续费 1 sat/byte
2. 网络拥堵，交易A长时间未确认
3. Alice发送交易B：1 BTC → Bob，手续费 50 sat/byte
4. 矿工优先打包交易B（手续费更高）
5. 交易A被丢弃
```

### 为什么需要RBF？

1. **加速确认**
   - 初始手续费估计不准
   - 网络突然拥堵
   - 紧急交易需要快速确认

2. **取消交易**
   - 发送到错误地址
   - 改变主意
   - 通过发送给自己实现"取消"

3. **批量优化**
   - 初始交易包含部分收款人
   - 后续添加更多收款人
   - 节省总手续费

---

## 技术实现

### RBF标记

**nSequence字段**:
```rust
// 启用RBF
input.sequence = 0xFFFFFFFD;  // < 0xFFFFFFFE

// 禁用RBF（最终交易）
input.sequence = 0xFFFFFFFF;
```

### BIP125规则

替换交易必须满足：

1. **更高手续费**
   ```rust
   new_tx.fee > original_tx.fee
   ```

2. **花费相同UTXO**
   ```rust
   new_tx.inputs == original_tx.inputs
   ```

3. **费率增量**
   ```rust
   new_tx.fee >= original_tx.fee + min_relay_fee
   ```

4. **不引入新的未确认UTXO**

---

## RBFManager实现

### 数据结构

```rust
pub struct RBFManager {
    replaceable_txs: Vec<String>,  // 可替换交易ID列表
}
```

### 方法

#### `new`

```rust
pub fn new() -> Self
```

创建新的RBF管理器。

#### `mark_replaceable`

```rust
pub fn mark_replaceable(&mut self, txid: String)
```

标记交易为可替换。

**示例**:
```rust
let mut rbf = RBFManager::new();
rbf.mark_replaceable(tx.id.clone());
```

#### `is_replaceable`

```rust
pub fn is_replaceable(&self, txid: &str) -> bool
```

检查交易是否可替换。

#### `replace_transaction`

```rust
pub fn replace_transaction(
    &mut self,
    original_txid: &str,
    new_tx: Transaction
) -> Result<(), String>
```

用新交易替换原交易。

**验证**:
1. 原交易必须可替换
2. 新交易手续费更高
3. 新交易有效

---

## 使用场景

### 场景1: 加速确认

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    advanced_tx::RBFManager,
};

fn speed_up_transaction() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let bob = Wallet::new();

    // 初始化余额
    setup_balance(&mut blockchain, &alice, 10000)?;

    println!("=== RBF加速交易演示 ===\n");

    // 1. 创建低手续费交易
    println!("--- 步骤1: 发送低手续费交易 ---");
    let slow_tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        1000,
        1,  // 低手续费：1 sat
    )?;

    println!("原始交易:");
    println!("  ID: {}", &slow_tx.id[..16]);
    println!("  金额: 1000 sat");
    println!("  手续费: 1 sat");
    println!("  费率: {:.2} sat/byte\n", slow_tx.fee_rate());

    blockchain.add_transaction(slow_tx.clone())?;
    rbf.mark_replaceable(slow_tx.id.clone());

    // 2. 网络拥堵，交易长时间未确认
    println!("--- 步骤2: 网络拥堵 ---");
    println!("⏰ 等待确认...");
    println!("⏰ 10分钟后仍未确认");
    println!("⚠️  手续费太低，需要加速\n");

    // 3. 创建高手续费替换交易
    println!("--- 步骤3: 创建替换交易（更高手续费）---");
    let fast_tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        1000,
        50,  // 高手续费：50 sat
    )?;

    println!("替换交易:");
    println!("  ID: {}", &fast_tx.id[..16]);
    println!("  金额: 1000 sat");
    println!("  手续费: 50 sat (50x)");
    println!("  费率: {:.2} sat/byte\n", fast_tx.fee_rate());

    // 4. 验证并替换
    if rbf.is_replaceable(&slow_tx.id) {
        if fast_tx.fee > slow_tx.fee {
            println!("✓ 满足RBF条件:");
            println!("  新手续费({}) > 原手续费({})", fast_tx.fee, slow_tx.fee);

            // 从待处理池移除原交易
            blockchain.pending_transactions.retain(|tx| tx.id != slow_tx.id);

            // 添加新交易
            blockchain.add_transaction(fast_tx)?;

            println!("✓ 交易已替换\n");
        }
    }

    // 5. 挖矿确认
    println!("--- 步骤4: 矿工打包（优先高费率）---");
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("✓ 交易已确认");
    println!("  Bob余额: {} sat", blockchain.get_balance(&bob.address));

    Ok(())
}
```

**输出**:
```
=== RBF加速交易演示 ===

--- 步骤1: 发送低手续费交易 ---
原始交易:
  ID: abc123...
  金额: 1000 sat
  手续费: 1 sat
  费率: 0.01 sat/byte

--- 步骤2: 网络拥堵 ---
⏰ 等待确认...
⏰ 10分钟后仍未确认
⚠️  手续费太低，需要加速

--- 步骤3: 创建替换交易（更高手续费）---
替换交易:
  ID: def456...
  金额: 1000 sat
  手续费: 50 sat (50x)
  费率: 0.50 sat/byte

✓ 满足RBF条件:
  新手续费(50) > 原手续费(1)
✓ 交易已替换

--- 步骤4: 矿工打包（优先高费率）---
✓ 交易已确认
  Bob余额: 1000 sat
```

---

### 场景2: 取消交易

```rust
fn cancel_transaction() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let wrong_addr = Wallet::new().address;  // 错误地址

    setup_balance(&mut blockchain, &alice, 10000)?;

    println!("=== RBF取消交易演示 ===\n");

    // 1. 发送到错误地址
    println!("--- 错误：发送到错误地址 ---");
    let wrong_tx = blockchain.create_transaction(
        &alice,
        wrong_addr.clone(),
        5000,
        10,
    )?;

    println!("错误交易:");
    println!("  收款人: {} (错误!)", &wrong_addr[..16]);
    println!("  金额: 5000 sat\n");

    blockchain.add_transaction(wrong_tx.clone())?;
    rbf.mark_replaceable(wrong_tx.id.clone());

    // 2. 发现错误，取消交易
    println!("--- 发现错误，尝试取消 ---");
    println!("策略: 用更高手续费发送给自己\n");

    // 3. 创建"取消"交易（发送给自己）
    let cancel_tx = blockchain.create_transaction(
        &alice,
        alice.address.clone(),  // 发给自己
        4950,  // 金额略少（扣除手续费）
        50,    // 更高手续费
    )?;

    println!("取消交易:");
    println!("  收款人: {} (自己)", &alice.address[..16]);
    println!("  金额: 4950 sat");
    println!("  手续费: 50 sat (5x)\n");

    // 4. 替换
    if cancel_tx.fee > wrong_tx.fee {
        blockchain.pending_transactions.retain(|tx| tx.id != wrong_tx.id);
        blockchain.add_transaction(cancel_tx)?;
        println!("✓ 交易已取消（实际是替换）\n");
    }

    // 5. 确认
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("✓ 资金已返回");
    println!("  Alice余额: {} sat", blockchain.get_balance(&alice.address));
    println!("  错误地址余额: {} sat", blockchain.get_balance(&wrong_addr));

    Ok(())
}
```

---

### 场景3: 批量支付优化

```rust
fn batch_payment_optimization() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let recipients: Vec<_> = (0..5).map(|_| Wallet::new()).collect();

    setup_balance(&mut blockchain, &alice, 100000)?;

    println!("=== RBF批量支付优化 ===\n");

    // 1. 初始支付（2个收款人）
    println!("--- 初始批量支付（2个收款人）---");
    let mut outputs = vec![
        TxOutput::new(1000, recipients[0].address.clone()),
        TxOutput::new(2000, recipients[1].address.clone()),
    ];

    // 创建交易...（简化）
    println!("支付:");
    println!("  收款人1: 1000 sat");
    println!("  收款人2: 2000 sat");
    println!("  手续费: 10 sat\n");

    // 2. 添加更多收款人
    println!("--- 添加更多收款人（RBF扩展）---");
    outputs.push(TxOutput::new(3000, recipients[2].address.clone()));
    outputs.push(TxOutput::new(4000, recipients[3].address.clone()));

    println!("新增:");
    println!("  收款人3: 3000 sat");
    println!("  收款人4: 4000 sat");
    println!("  手续费: 15 sat (只增加5 sat!)\n");

    println!("优势:");
    println!("  ✓ 4笔交易合并为1笔");
    println!("  ✓ 节省手续费 (4×10 - 15 = 25 sat)");
    println!("  ✓ 节省区块空间");

    Ok(())
}
```

---

## 安全考虑

### ⚠️ 零确认交易风险

**问题**: RBF使零确认交易不安全

```rust
// 攻击场景
// 1. 攻击者：Alice → 商家Bob (1 BTC, 低手续费)
//    商家看到交易，发货

// 2. 攻击者替换：Alice → Alice (1 BTC, 高手续费)
//    资金返回自己，商家损失

// 防御：等待确认
if confirmations < 1 {
    println!("⚠️ 警告：零确认交易不安全（RBF风险）");
    println!("建议：等待至少1个确认");
}
```

### 商家建议

```rust
fn accept_payment(tx: &Transaction) -> bool {
    // 1. 检查是否启用RBF
    if is_rbf_enabled(tx) {
        println!("⚠️ 交易启用了RBF");

        // 选项A: 拒绝零确认
        println!("等待确认中...");
        return false;

        // 选项B: 要求更高手续费
        if tx.fee_rate() < 50.0 {
            println!("手续费太低，需要 >= 50 sat/byte");
            return false;
        }
    }

    // 2. 等待足够确认
    let confirmations = get_confirmations(tx);
    if confirmations < 1 {
        return false;
    }

    true
}
```

---

## RBF vs CPFP

### Child-Pays-For-Parent (CPFP)

**CPFP**: 子交易支付父交易的手续费

```
父交易: Alice → Bob (低手续费)
  ↓
子交易: Bob → Charlie (高手续费)

矿工会一起打包以获得高手续费
```

### 对比

| 特性 | RBF | CPFP |
|------|-----|------|
| 操作者 | 发送者 | 接收者 |
| 机制 | 替换交易 | 子交易拉动 |
| 手续费 | 发送者支付 | 接收者支付 |
| 复杂度 | 简单 | 稍复杂 |
| 适用场景 | 发送者加速 | 接收者加速 |

---

## 最佳实践

### 1. 何时使用RBF

```rust
// ✅ 适合RBF的场景
if network_congested && !urgent {
    // 先发低手续费，需要时再加速
    create_rbf_transaction(fee_low);
}

// ❌ 不适合RBF的场景
if urgent || large_amount {
    // 直接发高手续费
    create_transaction(fee_high);
}
```

### 2. 手续费策略

```rust
fn calculate_replacement_fee(original_fee: u64) -> u64 {
    // 至少增加原费用的50%
    let min_increase = original_fee / 2;

    // 或达到当前推荐费率
    let recommended = get_recommended_fee_rate() * tx_size;

    max(original_fee + min_increase, recommended)
}
```

### 3. 用户通知

```rust
fn notify_replacement(original_tx: &Transaction, new_tx: &Transaction) {
    println!("📢 交易已被替换:");
    println!("  原交易: {}", &original_tx.id[..16]);
    println!("  新交易: {}", &new_tx.id[..16]);
    println!("  原手续费: {} sat", original_tx.fee);
    println!("  新手续费: {} sat", new_tx.fee);
    println!("  增加: +{} sat", new_tx.fee - original_tx.fee);
}
```

---

## 实现示例

### 完整的RBF交易流程

```rust
use bitcoin_simulation::advanced_tx::RBFManager;

fn rbf_complete_example() -> Result<(), String> {
    let mut blockchain = Blockchain::new();
    let mut rbf = RBFManager::new();

    let alice = Wallet::new();
    let bob = Wallet::new();

    // 1. 初始化
    setup_balance(&mut blockchain, &alice, 10000)?;

    // 2. 创建可替换交易
    let tx1 = blockchain.create_transaction(&alice, bob.address.clone(), 1000, 5)?;
    blockchain.add_transaction(tx1.clone())?;
    rbf.mark_replaceable(tx1.id.clone());

    println!("✓ 原交易已创建（手续费: 5 sat）");

    // 3. 监控交易状态
    std::thread::sleep(std::time::Duration::from_secs(30));

    if !is_confirmed(&blockchain, &tx1.id) {
        println!("⚠️ 30秒后仍未确认，准备加速...");

        // 4. 创建替换交易
        let tx2 = blockchain.create_transaction(&alice, bob.address, 1000, 50)?;

        // 5. 验证RBF规则
        if rbf.can_replace(&tx1, &tx2) {
            // 6. 执行替换
            blockchain.pending_transactions.retain(|tx| tx.id != tx1.id);
            blockchain.add_transaction(tx2.clone())?;

            println!("✓ 交易已替换（手续费: 50 sat）");

            // 7. 确认
            blockchain.mine_pending_transactions(alice.address)?;
            println!("✓ 新交易已确认");
        }
    }

    Ok(())
}
```

---

## 参考资料

- [BIP125 - Opt-in RBF](https://github.com/bitcoin/bips/blob/master/bip-0125.mediawiki)
- [Transaction API](../api/transaction.md)
- [手续费优化](./priority.md)

---

**总结**: RBF是强大的工具，但要注意零确认交易风险。商家应等待确认，用户应合理使用。

[返回高级特性](./README.md)
