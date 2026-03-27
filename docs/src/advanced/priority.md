# 交易优先级

当网络拥堵时，内存池（Mempool）中可能积压数千笔待确认交易。矿工每次只能打包约1MB数据进入区块，因此需要一套优先级机制来决定哪些交易先被确认。本章介绍SimpleBTC中交易优先级的计算方式、内存池排序逻辑以及手续费推荐策略。

---

## 核心概念：费率（Fee Rate）

**费率**（Fee Rate）是衡量交易优先级最重要的指标：

```
费率（sat/byte）= 手续费（satoshi）/ 交易大小（bytes）
```

矿工优先选择费率高的交易打包进区块，因为这样在相同区块空间下能获得最多手续费收益。

### 为什么用费率而不是绝对手续费？

一笔包含10个输入的复杂交易可能支付1000 sat手续费，但它占用900字节，费率约1.1 sat/byte。一笔只有1个输入的简单交易支付200 sat，仅占用192字节，费率约1.04 sat/byte。两者从矿工利益角度基本相当。若只看绝对手续费，会错误地优先选择前者，浪费区块空间。

---

## 内存池结构

SimpleBTC的`Mempool`使用**双索引结构**来支持高效的优先级排序：

```rust
pub struct Mempool {
    // 主存储：txid → 内存池条目
    transactions: HashMap<String, MempoolEntry>,

    // 费率索引：fee_rate → txid集合（BTreeMap自动有序，高效迭代）
    fee_index: BTreeMap<ordered_float::NotNan<f64>, HashSet<String>>,

    // UTXO索引：用于双花检测
    utxo_index: HashMap<String, String>,

    // 容量控制
    max_size: usize,       // 最大字节数（默认300MB）
    current_size: usize,   // 当前已用字节数
    min_fee_rate: f64,     // 最低接受费率（默认1.0 sat/byte）
    max_age: u64,          // 最长保留时间（默认72小时）
}
```

`BTreeMap`（平衡二叉搜索树）是关键：它按费率自动排序，使得"取费率最高的N笔交易"操作只需从尾部反向迭代，时间复杂度为O(N)。

### 内存池条目：`MempoolEntry`

```rust
pub struct MempoolEntry {
    pub transaction: Transaction,  // 完整交易数据
    pub added_time: u64,           // 加入时间（Unix时间戳）
    pub size: usize,               // 估算的字节大小
    pub fee_rate: f64,             // 计算出的费率（sat/byte）
    pub replaceable: bool,         // 是否支持RBF替换
}
```

费率在创建`MempoolEntry`时立即计算并缓存，避免重复计算：

```rust
impl MempoolEntry {
    pub fn new(transaction: Transaction, size: usize) -> Self {
        let fee_rate = if size > 0 {
            transaction.fee as f64 / size as f64
        } else {
            0.0
        };
        // ...
    }
}
```

---

## 交易大小估算

SimpleBTC使用简化的公式估算交易字节大小：

```rust
fn estimate_tx_size(&self, tx: &Transaction) -> usize {
    let base = 10;               // 固定开销（版本号、锁定时间等）
    let inputs_size = tx.inputs.len() * 148;   // 每个输入约148字节
    let outputs_size = tx.outputs.len() * 34;  // 每个输出约34字节
    base + inputs_size + outputs_size
}
```

**实际比特币交易大小参考**（原生SegWit，P2WPKH格式）：

| 交易类型 | 输入数 | 输出数 | 估算大小 |
|----------|--------|--------|----------|
| 简单转账 | 1 | 2 | 约192字节 |
| 合并多个UTXO | 5 | 2 | 约898字节 |
| 批量付款 | 1 | 10 | 约388字节 |

---

## 交易添加与验证流程

调用`mempool.add_transaction(tx)`时，内部按顺序执行以下检查：

```
交易到达内存池
      │
      ▼
① 是否已存在？ ──是──► 拒绝（重复交易）
      │否
      ▼
② 基本安全验证（格式、签名等）
      │
      ▼
③ 双花检测：是否有输入已被其他内存池交易花费？
      │
      ├─ 是，且旧交易支持RBF且新费用更高 ──► 触发RBF替换，继续
      │
      └─ 是，但不满足RBF条件 ──► 拒绝（双花攻击）
      │否
      ▼
④ 估算大小，计算费率
      │
      ▼
⑤ 费率 ≥ min_fee_rate？ ──否──► 拒绝（费率过低）
      │是
      ▼
⑥ 内存池是否已满？ ──是──► 触发淘汰低费率交易
      │
      ▼
⑦ 添加到 transactions、fee_index、utxo_index
      │
      ▼
     成功
```

```rust
// 示例：添加一笔交易到内存池
let mut mempool = Mempool::default(); // 300MB限制，1 sat/byte最低费率

let tx = Transaction::new(inputs, outputs, 0, 200); // 200 sat手续费
match mempool.add_transaction(tx) {
    Ok(()) => println!("交易已进入内存池"),
    Err(e) => println!("拒绝原因: {}", e),
}
```

---

## 优先级排序与区块打包

### 按费率获取前N笔：`get_top_transactions`

```rust
pub fn get_top_transactions(&self, max_count: usize) -> Vec<Transaction>
```

通过反向迭代`fee_index`（BTreeMap从大到小），快速取出费率最高的交易：

```rust
// 从高费率到低费率遍历
for (_fee_rate, txids) in self.fee_index.iter().rev() {
    for txid in txids {
        if let Some(entry) = self.transactions.get(txid) {
            result.push(entry.transaction.clone());
            if result.len() >= max_count {
                return result;
            }
        }
    }
}
```

```rust
// 用法：矿工想预览最优质的10笔交易
let top_txs = mempool.get_top_transactions(10);
for tx in &top_txs {
    println!("txid: {}, fee: {} sat", tx.id, tx.fee);
}
```

### 按区块大小限制打包：`get_transactions_for_block`

```rust
pub fn get_transactions_for_block(&self, max_size: usize) -> Vec<Transaction>
```

更实用的区块打包函数。同样按费率从高到低选取，但额外检查累积大小不超过`max_size`字节：

```rust
pub fn get_transactions_for_block(&self, max_size: usize) -> Vec<Transaction> {
    let mut result = Vec::new();
    let mut total_size = 0;

    for (_fee_rate, txids) in self.fee_index.iter().rev() {
        for txid in txids {
            if let Some(entry) = self.transactions.get(txid) {
                if total_size + entry.size <= max_size {
                    result.push(entry.transaction.clone());
                    total_size += entry.size;
                }
            }
        }
    }
    result
}
```

```rust
// 用法：为新区块打包交易（比特币区块限制约1MB = 1_000_000字节）
let block_txs = mempool.get_transactions_for_block(1_000_000);
println!("选中 {} 笔交易用于打包", block_txs.len());
```

---

## 综合优先级评分

SimpleBTC在`src/advanced_tx.rs`中提供了`TxPriorityCalculator`，实现了更精细的优先级计算。

### 基础费率计算

```rust
pub fn calculate_fee_rate(fee: u64, size: usize) -> f64 {
    if size == 0 { return 0.0; }
    fee as f64 / size as f64
}
```

```rust
// 200 sat手续费，交易大小192字节
let fee_rate = TxPriorityCalculator::calculate_fee_rate(200, 192);
println!("费率: {:.2} sat/byte", fee_rate); // 约1.04 sat/byte
```

### 硬币年龄优先级

比特币早期（SegWit之前）也考虑"硬币年龄"（Coin Age）：UTXO的价值乘以其等待的区块数，除以交易大小：

```rust
/// 优先级 = (输入价值 × 输入确认数) / 交易大小
pub fn calculate_priority(
    input_value: u64,  // 输入的总价值（satoshi）
    input_age: u32,    // 输入UTXO已确认的区块数
    tx_size: usize,
) -> f64 {
    (input_value as f64 * input_age as f64) / tx_size as f64
}
```

```rust
// 示例：输入价值1 BTC = 100_000_000 sat，已确认100个区块，交易大小200字节
let priority = TxPriorityCalculator::calculate_priority(100_000_000, 100, 200);
println!("硬币年龄优先级: {:.0}", priority); // 50_000_000
```

> **历史背景**：比特币核心在0.12版本（2016年）移除了基于硬币年龄的免费交易优先级，因为低手续费交易严重拖慢区块打包速度。现代网络中，费率是唯一实际起作用的优先级指标。

### 综合评分公式：70% 费率 + 30% 硬币年龄

```rust
/// 综合评分 = 费率 × 0.7 + 优先级 × 0.001 × 0.3
pub fn calculate_score(fee_rate: f64, priority: f64) -> f64 {
    fee_rate * 0.7 + priority * 0.001 * 0.3
}
```

这个加权公式的设计思路：
- **70%的权重给费率**：保证矿工利益最大化，高费率交易仍然优先
- **30%的权重给硬币年龄**（乘以0.001缩放系数）：给长期等待的交易一个"加分"，避免低费率旧UTXO永久无法确认

```rust
// 完整评分示例
let fee_rate = TxPriorityCalculator::calculate_fee_rate(500, 200); // 2.5 sat/byte
let priority = TxPriorityCalculator::calculate_priority(50_000_000, 10, 200); // 2_500_000
let score = TxPriorityCalculator::calculate_score(fee_rate, priority);
println!("综合分数: {:.4}", score);
// score = 2.5 * 0.7 + 2_500_000 * 0.001 * 0.3 = 1.75 + 750 = 751.75
```

---

## 手续费推荐

`TxPriorityCalculator::recommend_fee`根据紧急程度返回建议手续费：

```rust
pub enum FeeUrgency {
    Low,    // 低优先级：几小时内确认
    Medium, // 中优先级：30-60分钟确认
    High,   // 高优先级：10-20分钟（约1-2个区块）
    Urgent, // 紧急：下一个区块（最高优先级）
}

pub fn recommend_fee(tx_size: usize, urgency: FeeUrgency) -> u64 {
    let sat_per_byte = match urgency {
        FeeUrgency::Low    => 1.0,   // 1 sat/byte
        FeeUrgency::Medium => 5.0,   // 5 sat/byte
        FeeUrgency::High   => 20.0,  // 20 sat/byte
        FeeUrgency::Urgent => 50.0,  // 50 sat/byte
    };
    (tx_size as f64 * sat_per_byte) as u64
}
```

```rust
use bitcoin_simulation::advanced_tx::{TxPriorityCalculator, FeeUrgency};

// 估算一笔标准交易（1输入2输出）的建议手续费
let tx_size = 10 + 1 * 148 + 2 * 34; // = 226 字节

let low_fee    = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Low);
let medium_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Medium);
let high_fee   = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::High);
let urgent_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Urgent);

println!("低优先级:  {} sat ({} sat/byte)", low_fee,    1);  // 226 sat
println!("中优先级:  {} sat ({} sat/byte)", medium_fee, 5);  // 1130 sat
println!("高优先级:  {} sat ({} sat/byte)", high_fee,   20); // 4520 sat
println!("紧急:      {} sat ({} sat/byte)", urgent_fee, 50); // 11300 sat
```

**实际比特币网络费率参考**（2024年数据，BTC/USD = 60,000$）：

| 紧急程度 | 典型费率 | 约合美元（226字节交易） |
|----------|---------|------------------------|
| 低 | 1-3 sat/byte | $0.14 - $0.41 |
| 中 | 5-15 sat/byte | $0.68 - $2.03 |
| 高 | 20-50 sat/byte | $2.71 - $6.78 |
| 紧急 | 50-200 sat/byte | $6.78 - $27.1 |

> **注意**：实际费率受网络拥堵影响极大。2017年牛市高峰期，部分用户支付了超过50美元的手续费才能快速确认。

---

## 低费率交易淘汰机制

当内存池达到容量上限时，会自动淘汰费率最低的交易以腾出空间：

```rust
fn evict_low_fee_transactions(&mut self, needed_size: usize) -> Result<()> {
    let mut freed_size = 0;
    let mut to_remove = Vec::new();

    // 从【低费率到高费率】遍历（BTreeMap正向迭代）
    for (_fee_rate, txids) in self.fee_index.iter() {
        for txid in txids {
            if let Some(entry) = self.transactions.get(txid) {
                to_remove.push(txid.clone());
                freed_size += entry.size;
                if freed_size >= needed_size {
                    break;
                }
            }
        }
        if freed_size >= needed_size { break; }
    }

    // 执行淘汰
    for txid in &to_remove {
        self.remove_transaction(txid)?;
    }
    Ok(())
}
```

这个设计保证了内存池始终维护着"费率最高"的交易子集，低费率交易在竞争中被自然淘汰。

```rust
// 创建一个容量极小的内存池来演示淘汰行为
let mut mempool = Mempool::new(1000, 1.0); // 仅1KB容量

// 添加多笔交易，当超过1KB时，低费率的会被淘汰
for i in 1..=10 {
    let tx = create_tx_with_fee(i * 100); // fee: 100, 200, ..., 1000
    let _ = mempool.add_transaction(tx); // 低费率的可能被淘汰
}
```

---

## 过期交易清理

默认情况下，在内存池中等待超过72小时的交易会被清理：

```rust
// 定期调用（例如每小时一次）
let expired_count = mempool.clear_expired();
if expired_count > 0 {
    println!("清理了 {} 笔过期交易", expired_count);
}
```

---

## 内存池统计信息

```rust
let stats = mempool.get_stats();
println!("待确认交易数:   {}", stats.tx_count);
println!("内存池大小:     {} / {} bytes", stats.total_size, stats.max_size);
println!("总待收手续费:   {} sat", stats.total_fees);
println!("平均费率:       {:.2} sat/byte", stats.avg_fee_rate);
println!("最低接受费率:   {:.2} sat/byte", stats.min_fee_rate);
```

---

## Replace-By-Fee（RBF）

RBF（BIP125）允许用户用更高手续费的新交易替换内存池中的旧交易。在SimpleBTC中，`advanced_tx.rs`的`RBFManager`管理可替换交易：

```rust
use bitcoin_simulation::advanced_tx::RBFManager;

let mut rbf = RBFManager::new();

// 标记交易为可替换（发送时设置sequence < 0xFFFFFFFE）
rbf.mark_replaceable("original_tx_id");

// 稍后，以更高费用的新交易替换旧交易
let can_replace = rbf.can_replace(&old_tx, &new_tx);
match can_replace {
    Ok(()) => println!("RBF替换成功"),
    Err(reason) => println!("替换被拒绝: {}", reason),
}
```

RBF替换条件（由`can_replace`验证）：
1. 旧交易必须已标记为可替换（`replaceable = true`）
2. 新旧交易的输入数量必须相同，且引用相同的UTXO
3. 新交易手续费必须严格高于旧交易
4. 手续费增量至少为旧交易大小（约1 sat/byte）

---

## 小结

SimpleBTC的交易优先级系统由三个层次构成：

| 层次 | 组件 | 作用 |
|------|------|------|
| 内存池排序 | `Mempool` + `BTreeMap<fee_rate>` | 按费率自动维护有序队列 |
| 优先级计算 | `TxPriorityCalculator` | 费率、硬币年龄、综合评分 |
| 手续费推荐 | `FeeUrgency` + `recommend_fee` | 按紧急程度推荐合理费用 |

核心公式回顾：
```
费率（sat/byte）= 手续费 / 交易大小
综合评分        = 费率 × 0.7 + 硬币年龄优先级 × 0.001 × 0.3
推荐手续费      = 交易大小 × sat_per_byte（按紧急程度选取1/5/20/50）
```
