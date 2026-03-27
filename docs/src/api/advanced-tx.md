# Advanced TX API

高级交易模块实现在 `src/advanced_tx.rs` 中，提供三项解决比特币网络实际工程问题的关键机制：**RBF 替换手续费**（允许加速或取消未确认交易）、**TimeLock 时间锁**（限制交易在指定时间/区块前无法确认）以及 **TxPriorityCalculator 手续费计算器**（推荐合理手续费并计算交易优先级）。

---

## RBFManager — Replace-By-Fee 管理器

RBF（Replace-By-Fee）是 BIP125 定义的机制，允许用户用手续费更高的新交易替换内存池中尚未确认的旧交易，从而加速确认或取消错误交易。

### 结构体定义

```rust
pub struct RBFManager {
    // replaceable_txs: Vec<String>  // 私有字段，存储可替换的交易 ID 列表
}
```

### 方法

#### `RBFManager::new`

创建新的 RBF 管理器实例（可替换交易列表为空）。

```rust
pub fn new() -> Self
```

#### `RBFManager::mark_replaceable`

将指定交易标记为支持 RBF 替换。幂等操作，重复标记同一交易不会产生副作用。

```rust
pub fn mark_replaceable(&mut self, tx_id: &str)
```

**参数：**
- `tx_id` — 要标记为可替换的交易 ID。

在比特币协议中，通过将交易的 `nSequence` 字段设置为小于 `0xFFFFFFFE` 的值来表示支持 RBF。`AdvancedTxBuilder::with_rbf()` 会自动将 `sequence` 设为 `0xFFFFFFFD`。

#### `RBFManager::is_replaceable`

检查某笔交易是否已被标记为可替换。

```rust
pub fn is_replaceable(&self, tx_id: &str) -> bool
```

#### `RBFManager::can_replace`

验证新交易是否可以合法替换旧交易。执行完整的 RBF 规则校验：

```rust
pub fn can_replace(
    &self,
    old_tx: &Transaction,
    new_tx: &Transaction,
) -> Result<(), String>
```

**验证规则（按顺序）：**

1. **可替换性检查：** `old_tx.id` 必须在可替换列表中，否则返回 `"原交易不支持RBF"`。
2. **输入相同：** 两笔交易的输入列表长度相同，且对应输入的 `txid` 和 `vout` 完全一致（必须花费相同的 UTXO），否则返回 `"必须花费相同的UTXO"`。
3. **手续费更高：** `new_tx.fee > old_tx.fee`，否则返回 `"新交易手续费({})必须高于旧交易({})"`。
4. **增量足够：** `fee_increase >= old_tx.size()`（简化规则：手续费增量至少为旧交易字节数个 satoshi），防止低成本的垃圾替换攻击。

**返回值：**
- `Ok(())` — 替换合法，可以广播新交易。
- `Err(String)` — 具体的验证失败原因。

#### `RBFManager::remove_confirmed`

将已被区块打包确认的交易从可替换列表中移除。

```rust
pub fn remove_confirmed(&mut self, tx_id: &str)
```

### RBFManager 使用示例

```rust
use simplebtc::advanced_tx::RBFManager;

let mut rbf = RBFManager::new();

// 标记原始交易支持 RBF
rbf.mark_replaceable("original_tx_001");
assert!(rbf.is_replaceable("original_tx_001"));
assert!(!rbf.is_replaceable("other_tx_002"));

// 验证替换是否合法
match rbf.can_replace(&old_tx, &new_tx) {
    Ok(()) => println!("替换合法，广播新交易"),
    Err(e) => println!("替换被拒绝: {}", e),
}

// 交易确认后移除记录
rbf.remove_confirmed("original_tx_001");
assert!(!rbf.is_replaceable("original_tx_001"));
```

---

## TimeLock — 时间锁

时间锁限制交易在特定时间或区块高度之前无法被矿工打包，是实现定期存款、遗产继承、智能合约等高级场景的基础原语。

### 结构体定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLock {
    pub locktime: u64,         // 锁定值：Unix 时间戳（秒）或区块高度
    pub is_block_height: bool, // true: 基于区块高度；false: 基于时间戳
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `locktime` | `u64` | 锁定值。`is_block_height = true` 时为区块高度，否则为 Unix 时间戳（秒）。 |
| `is_block_height` | `bool` | 锁定类型标志。对应比特币协议中 `locktime < 500_000_000` 为区块高度，`>= 500_000_000` 为时间戳。 |

### 方法

#### `TimeLock::new_time_based`

创建基于 Unix 时间戳的时间锁。交易在 `timestamp` 秒之前无法被确认。

```rust
pub fn new_time_based(timestamp: u64) -> Self
```

**参数：**
- `timestamp` — 解锁时间的 Unix 时间戳（秒）。例如 `1767225600` 表示 2026-01-01 00:00:00 UTC。

#### `TimeLock::new_height_based`

创建基于区块高度的时间锁。交易在区块链达到指定高度之前无法被确认。

```rust
pub fn new_height_based(height: u64) -> Self
```

**参数：**
- `height` — 解锁所需的区块高度。例如 `900_000` 表示约 2027 年中（基于约 10 分钟/区块估算）。

#### `TimeLock::is_mature`

检查时间锁是否已到期（可以使用）。

```rust
pub fn is_mature(&self, current_time: u64, current_height: u32) -> bool
```

**参数：**
- `current_time` — 当前 Unix 时间戳（秒）。
- `current_height` — 当前区块链高度。

**返回值：**
- 基于时间：`current_time >= self.locktime`
- 基于区块高度：`current_height as u64 >= self.locktime`

#### `TimeLock::remaining`

获取距离解锁还剩多少时间（秒）或区块数。

```rust
pub fn remaining(&self, current_time: u64, current_height: u32) -> i64
```

**返回值：** 剩余秒数或区块数。负值表示已超过锁定时间（已到期）。

### TimeLock 使用示例

```rust
use simplebtc::advanced_tx::TimeLock;

// 基于时间戳：锁定至 2026-01-01 00:00:00 UTC
let time_lock = TimeLock::new_time_based(1_767_225_600);
let now = 1_740_000_000u64; // 当前时间（2025年）
println!("时间锁已到期: {}", time_lock.is_mature(now, 0)); // false
println!("距解锁剩余: {} 秒", time_lock.remaining(now, 0));

// 基于区块高度：锁定至第 90 万个区块
let block_lock = TimeLock::new_height_based(900_000);
let current_height = 850_000u32; // 当前区块高度
println!("区块锁已到期: {}", block_lock.is_mature(0, current_height)); // false
println!("距解锁剩余: {} 个区块", block_lock.remaining(0, current_height)); // 50000

// 已到期的时间锁
let expired_lock = TimeLock::new_height_based(800_000);
println!("已到期: {}", expired_lock.is_mature(0, 850_000)); // true
println!("剩余（负数表示已过期）: {}", expired_lock.remaining(0, 850_000)); // -50000
```

---

## AdvancedTxBuilder — 高级交易构建器

`AdvancedTxBuilder` 是一个构建器（Builder Pattern），用于配置交易的高级选项（RBF 支持和时间锁），并生成对应的 `sequence` 字段值。

### 结构体定义

```rust
pub struct AdvancedTxBuilder {
    pub enable_rbf: bool,
    pub timelock: Option<TimeLock>,
    pub sequence: u32,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable_rbf` | `bool` | 是否启用 RBF 支持。`with_rbf()` 后为 `true`。 |
| `timelock` | `Option<TimeLock>` | 关联的时间锁配置。`with_timelock()` 后为 `Some(TimeLock)`。 |
| `sequence` | `u32` | 交易输入的序列号，编码了 RBF 和时间锁状态：`0xFFFFFFFF`（默认/无功能）、`0xFFFFFFFD`（RBF）、`0x00000000`（时间锁）。 |

### 方法

#### `AdvancedTxBuilder::new`

创建默认构建器。默认不启用 RBF 和时间锁，`sequence = 0xFFFFFFFF`。

```rust
pub fn new() -> Self
```

#### `AdvancedTxBuilder::with_rbf`

启用 RBF 支持。将 `enable_rbf` 设为 `true`，`sequence` 设为 `0xFFFFFFFD`（小于 `0xFFFFFFFE`，符合 BIP125 规范）。

```rust
pub fn with_rbf(mut self) -> Self
```

**返回值：** `Self`（支持链式调用）。

#### `AdvancedTxBuilder::with_timelock`

设置时间锁。将 `timelock` 设为 `Some(timelock)`，`sequence` 设为 `0`（启用 `nLockTime` 机制）。

```rust
pub fn with_timelock(mut self, timelock: TimeLock) -> Self
```

**参数：**
- `timelock` — 要关联的 `TimeLock` 实例。

**返回值：** `Self`（支持链式调用）。

#### `AdvancedTxBuilder::get_sequence`

获取最终的 `sequence` 字段值，应写入交易输入的 `nSequence` 字段。

```rust
pub fn get_sequence(&self) -> u32
```

#### `AdvancedTxBuilder::supports_rbf`

检查当前配置是否支持 RBF（`sequence < 0xFFFFFFFE`）。

```rust
pub fn supports_rbf(&self) -> bool
```

### AdvancedTxBuilder 使用示例

```rust
use simplebtc::advanced_tx::{AdvancedTxBuilder, TimeLock};

// 仅启用 RBF
let rbf_builder = AdvancedTxBuilder::new()
    .with_rbf();
println!("sequence: 0x{:08X}", rbf_builder.get_sequence()); // 0xFFFFFFFD
println!("支持 RBF: {}", rbf_builder.supports_rbf()); // true

// 仅启用时间锁（锁定至第 900,000 个区块）
let timelock = TimeLock::new_height_based(900_000);
let timelock_builder = AdvancedTxBuilder::new()
    .with_timelock(timelock);
println!("sequence: 0x{:08X}", timelock_builder.get_sequence()); // 0x00000000

// RBF + 时间锁组合（with_timelock 会覆盖 sequence 为 0）
let combined = AdvancedTxBuilder::new()
    .with_rbf()
    .with_timelock(TimeLock::new_time_based(1_800_000_000));
println!("时间锁: {:?}", combined.timelock);
println!("sequence: 0x{:08X}", combined.get_sequence()); // 0x00000000

// 默认构建器（无高级功能）
let default_builder = AdvancedTxBuilder::new();
println!("sequence: 0x{:08X}", default_builder.get_sequence()); // 0xFFFFFFFF
println!("支持 RBF: {}", default_builder.supports_rbf()); // false
```

---

## TxPriorityCalculator — 交易优先级计算器

`TxPriorityCalculator` 是一个无状态工具类（所有方法均为关联函数），用于计算交易优先级分数和推荐合理手续费。矿工使用优先级分数决定先打包哪些内存池中的交易。

### 结构体定义

```rust
pub struct TxPriorityCalculator;
```

### FeeUrgency — 手续费紧急程度

```rust
#[derive(Debug, Clone, Copy)]
pub enum FeeUrgency {
    Low,    // 低优先级：1 sat/byte，几小时内确认
    Medium, // 中优先级：5 sat/byte，30-60 分钟确认
    High,   // 高优先级：20 sat/byte，10-20 分钟（约 1-2 个区块）
    Urgent, // 紧急：50 sat/byte，下一个区块（约 10 分钟）
}
```

| 枚举值 | 费率 | 预期确认时间 | 典型场景 |
|--------|------|-------------|----------|
| `Low` | 1 sat/byte | 数小时至数天 | 非紧急转账、低网络费用时段 |
| `Medium` | 5 sat/byte | 30-60 分钟 | 日常交易、普通确认速度 |
| `High` | 20 sat/byte | 10-20 分钟 | 时间敏感交易（闪电网络开通） |
| `Urgent` | 50 sat/byte | ~10 分钟（下一区块）| 紧急支付、交易所提现 |

### 方法

#### `TxPriorityCalculator::calculate_priority`

计算基于 UTXO 价值和年龄的传统优先级分数。

**公式：** `priority = (input_value × input_age) / tx_size`

```rust
pub fn calculate_priority(
    input_value: u64, // 输入 UTXO 总价值（satoshi）
    input_age: u32,   // 输入 UTXO 的年龄（确认区块数）
    tx_size: usize,   // 交易大小（字节）
) -> f64
```

较老（age 大）且价值较高的 UTXO 的优先级更高。`tx_size = 0` 时返回 `0.0`（防止除零）。

#### `TxPriorityCalculator::calculate_fee_rate`

计算交易的手续费率（sat/byte）。

**公式：** `fee_rate = fee / size`

```rust
pub fn calculate_fee_rate(
    fee: u64,    // 手续费（satoshi）
    size: usize, // 交易大小（字节）
) -> f64
```

`size = 0` 时返回 `0.0`。

#### `TxPriorityCalculator::calculate_score`

计算综合评分（矿工排序依据）。

**公式：** `score = fee_rate × 0.7 + priority × 0.001 × 0.3`

```rust
pub fn calculate_score(fee_rate: f64, priority: f64) -> f64
```

权重分配：70% 基于费率，30% 基于 UTXO 优先级。高费率的交易综合得分更高，更容易被矿工选中。

#### `TxPriorityCalculator::recommend_fee`

根据交易大小和紧急程度推荐手续费（satoshi）。

```rust
pub fn recommend_fee(
    tx_size: usize,    // 交易大小（字节）
    urgency: FeeUrgency, // 手续费紧急程度
) -> u64
```

**返回值：** `(tx_size × sat_per_byte) as u64` 向下取整。

### TxPriorityCalculator 使用示例

```rust
use simplebtc::advanced_tx::{TxPriorityCalculator, FeeUrgency};

// 标准比特币交易约 250 字节（1 输入 + 2 输出）
let tx_size = 250usize;

// 推荐各紧急程度的手续费
println!("低优先级:  {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Low));
// 250 sat
println!("中优先级:  {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Medium));
// 1250 sat
println!("高优先级:  {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::High));
// 5000 sat
println!("紧急:      {} sat", TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Urgent));
// 12500 sat

// 计算现有交易的费率
let actual_fee = 2000u64; // 实际手续费
let fee_rate = TxPriorityCalculator::calculate_fee_rate(actual_fee, tx_size);
println!("实际费率: {:.1} sat/byte", fee_rate); // 8.0 sat/byte

// 计算 UTXO 优先级（持有 1 BTC、已确认 100 个区块、250 字节交易）
let priority = TxPriorityCalculator::calculate_priority(
    100_000_000, // 1 BTC = 100,000,000 satoshi
    100,         // 100 个区块年龄
    tx_size,
);
println!("优先级分数: {:.0}", priority); // 40,000,000

// 综合评分（矿工排序依据）
let score = TxPriorityCalculator::calculate_score(fee_rate, priority);
println!("综合评分: {:.2}", score);
```

---

## 完整使用示例

### 场景一：使用 RBF 加速未确认交易

```rust
use simplebtc::advanced_tx::{RBFManager, AdvancedTxBuilder, TxPriorityCalculator, FeeUrgency};

fn accelerate_tx_example() {
    let mut rbf = RBFManager::new();

    // 1. 发送原始交易（低手续费，支持 RBF）
    let builder = AdvancedTxBuilder::new().with_rbf();
    println!("RBF sequence: 0x{:08X}", builder.get_sequence()); // 0xFFFFFFFD

    // 模拟交易被发送，但 30 分钟后仍未确认
    // ... 创建并广播原始交易 original_tx ...
    rbf.mark_replaceable("original_tx_id_001");

    // 2. 网络拥堵，需要提高手续费
    let tx_size = 250usize;
    let old_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::Low);
    let new_fee = TxPriorityCalculator::recommend_fee(tx_size, FeeUrgency::High);
    println!("原始手续费: {} sat -> 新手续费: {} sat", old_fee, new_fee);

    // 3. 验证替换规则
    // can_replace 会检查：输入相同、新费更高、增量足够
    // match rbf.can_replace(&old_tx, &new_tx) {
    //     Ok(()) => { /* 广播新交易 */ }
    //     Err(e) => println!("替换被拒绝: {}", e),
    // }

    // 4. 旧交易确认（或被替换后）清理
    rbf.remove_confirmed("original_tx_id_001");
}
```

### 场景二：定期存款时间锁

```rust
use simplebtc::advanced_tx::{AdvancedTxBuilder, TimeLock};

fn savings_timelock() {
    // 锁定至区块高度 950,000（约 2028 年）
    let unlock_height = 950_000u64;
    let timelock = TimeLock::new_height_based(unlock_height);

    let builder = AdvancedTxBuilder::new()
        .with_timelock(timelock.clone());

    println!("交易 sequence: 0x{:08X}", builder.get_sequence()); // 0x00000000
    println!("启用时间锁: {}", builder.timelock.is_some());

    // 检查当前是否可以动用资金
    let current_height = 870_000u32;
    if timelock.is_mature(0, current_height) {
        println!("资金已解锁，可以使用");
    } else {
        let remaining = timelock.remaining(0, current_height);
        println!("还需等待 {} 个区块（约 {} 天）",
            remaining,
            remaining * 10 / 60 / 24); // 约算天数
    }
}
```

### 场景三：手续费策略分析

```rust
use simplebtc::advanced_tx::{TxPriorityCalculator, FeeUrgency};

fn fee_strategy_analysis() {
    let tx_sizes = vec![
        (125,  "简单支付（1 输入 1 输出）"),
        (250,  "标准交易（1 输入 2 输出）"),
        (500,  "批量支付（多输入多输出）"),
        (1000, "大型交易（SegWit 之前常见）"),
    ];

    println!("{:<40} {:>10} {:>10} {:>10} {:>10}",
        "交易类型", "Low", "Medium", "High", "Urgent");
    println!("{}", "-".repeat(80));

    for (size, desc) in &tx_sizes {
        println!("{:<40} {:>10} {:>10} {:>10} {:>10}",
            desc,
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::Low),
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::Medium),
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::High),
            TxPriorityCalculator::recommend_fee(*size, FeeUrgency::Urgent),
        );
    }

    // 综合评分比较
    let fee_rate_a = TxPriorityCalculator::calculate_fee_rate(500, 250); // 2 sat/byte
    let fee_rate_b = TxPriorityCalculator::calculate_fee_rate(5000, 250); // 20 sat/byte
    let priority_a = TxPriorityCalculator::calculate_priority(10_000_000, 50, 250);
    let priority_b = TxPriorityCalculator::calculate_priority(100_000, 1, 250);

    println!("\n交易A（老 UTXO，低费率）综合分: {:.2}", TxPriorityCalculator::calculate_score(fee_rate_a, priority_a));
    println!("交易B（新 UTXO，高费率）综合分: {:.2}", TxPriorityCalculator::calculate_score(fee_rate_b, priority_b));
}
```

---

## sequence 字段值对照

| sequence 值 | 含义 |
|-------------|------|
| `0xFFFFFFFF` | 默认值，不启用 RBF 和时间锁 |
| `0xFFFFFFFE` | 不支持 RBF，但允许 nLockTime |
| `0xFFFFFFFD` | 支持 RBF（BIP125 标准值） |
| `0x00000000` | 启用时间锁（nLockTime 生效） |

---

## 相关模块

- [`Mempool`](advanced.md#内存池mempool) — 内存池使用 `RBFManager` 处理交易替换，使用 `TxPriorityCalculator` 排序待打包交易。
- [`MultiSig`](multisig.md) — 多签与时间锁可组合，实现"时间到期前需要 M-of-N，之后降为 1-of-N"等场景。
- [高级模块概览](advanced.md) — 查看完整的高级模块依赖图。
