# 时间锁（TimeLock / nLockTime）

时间锁是比特币的重要特性，限制交易在特定时间或区块高度之前不能被确认。

## 概述

### 什么是时间锁？

时间锁让交易在未来某个时间点才能被使用，实现"延迟支付"功能。

**示例**:
```
Alice创建交易: 1 BTC → Bob
时间锁: 2025年1月1日

在2025年1月1日之前:
  ❌ 交易无法被确认
  ❌ 矿工拒绝打包

2025年1月1日之后:
  ✓ 交易可以被确认
  ✓ 矿工可以打包
```

---

## 两种类型

### 1. 基于Unix时间戳

```rust
// locktime >= 500,000,000
let unlock_time = 1735689600;  // 2025-01-01 00:00:00
let timelock = TimeLock::new_time_based(unlock_time);
```

**特点**:
- 以秒为单位
- 适合精确时间控制
- 受系统时间影响

**使用场景**:
- 工资发放（每月1号）
- 债券到期（固定日期）
- 定期存款（3/6/12个月）

### 2. 基于区块高度

```rust
// locktime < 500,000,000
let unlock_height = 800000;  // 第800,000个区块
let timelock = TimeLock::new_block_based(unlock_height);
```

**特点**:
- 以区块为单位
- 更精确（约10分钟/块）
- 不受系统时间影响

**使用场景**:
- 更精确的时间控制
- 避免时间戳操纵
- 智能合约触发

**时间估算**:
```
1块 ≈ 10分钟
6块 ≈ 1小时
144块 ≈ 1天
1008块 ≈ 1周
4032块 ≈ 1月
```

---

## TimeLock实现

### 数据结构

```rust
pub struct TimeLock {
    pub locktime: u64,         // 锁定时间/高度
    pub is_block_height: bool, // true: 区块高度, false: 时间戳
}
```

### 方法

#### `new_time_based`

```rust
pub fn new_time_based(timestamp: u64) -> Self
```

创建基于时间的时间锁。

**参数**:
- `timestamp` - Unix时间戳（秒）

**示例**:
```rust
use bitcoin_simulation::advanced_tx::TimeLock;
use std::time::{SystemTime, UNIX_EPOCH};

let current_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

// 3个月后解锁
let three_months = 90 * 24 * 3600;
let unlock_time = current_time + three_months;
let timelock = TimeLock::new_time_based(unlock_time);

println!("锁定至: {}", format_timestamp(unlock_time));
```

#### `new_block_based`

```rust
pub fn new_block_based(block_height: u64) -> Self
```

创建基于区块高度的时间锁。

**参数**:
- `block_height` - 目标区块高度

**示例**:
```rust
let current_height = blockchain.chain.len() as u64;

// 1000个区块后解锁（约1周）
let unlock_height = current_height + 1000;
let timelock = TimeLock::new_block_based(unlock_height);

println!("锁定至区块 #{}", unlock_height);
```

#### `is_mature`

```rust
pub fn is_mature(&self, current_time: u64, current_height: u64) -> bool
```

检查时间锁是否已到期。

**参数**:
- `current_time` - 当前Unix时间戳
- `current_height` - 当前区块高度

**返回值**:
- `true` - 已到期，可以使用
- `false` - 未到期，仍被锁定

**示例**:
```rust
let timelock = TimeLock::new_time_based(unlock_time);

if timelock.is_mature(current_time, 0) {
    println!("✓ 已到期，可以花费");
} else {
    let remaining = unlock_time - current_time;
    println!("🔒 仍被锁定，剩余 {} 秒", remaining);
}
```

---

## 应用场景

### 场景1: 定期存款

```rust
fn savings_account_demo() -> Result<(), String> {
    println!("=== 定期存款演示 ===\n");

    let alice = Wallet::new();
    let mut blockchain = Blockchain::new();

    // 初始化余额
    setup_balance(&mut blockchain, &alice, 100000)?;

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 产品1: 3个月定存
    println!("--- 产品A: 3个月定期存款 ---");
    let three_months = 90 * 24 * 3600;
    let maturity_3m = current_time + three_months;
    let deposit_3m = TimeLock::new_time_based(maturity_3m);

    println!("存款金额: 30,000 sat");
    println!("期限: 3个月");
    println!("到期日: {}", format_date(maturity_3m));
    println!("年化利率: 3%\n");

    // 产品2: 1年定存
    println!("--- 产品B: 1年定期存款 ---");
    let one_year = 365 * 24 * 3600;
    let maturity_1y = current_time + one_year;
    let deposit_1y = TimeLock::new_time_based(maturity_1y);

    println!("存款金额: 50,000 sat");
    println!("期限: 1年");
    println!("到期日: {}", format_date(maturity_1y));
    println!("年化利率: 5%\n");

    // 检查到期状态
    println!("--- 当前状态检查 ---");
    println!("当前时间: {}", format_date(current_time));

    if deposit_3m.is_mature(current_time, 0) {
        println!("✓ 3个月定存已到期，可提取");
        let interest = 30000 * 3 / 100 / 4;  // 季度利息
        println!("  本息: {} sat", 30000 + interest);
    } else {
        let days_left = (maturity_3m - current_time) / 86400;
        println!("🔒 3个月定存锁定中");
        println!("  剩余: {} 天", days_left);
    }

    if deposit_1y.is_mature(current_time, 0) {
        println!("✓ 1年定存已到期，可提取");
        let interest = 50000 * 5 / 100;  // 年利息
        println!("  本息: {} sat", 50000 + interest);
    } else {
        let days_left = (maturity_1y - current_time) / 86400;
        println!("🔒 1年定存锁定中");
        println!("  剩余: {} 天", days_left);
    }

    Ok(())
}
```

**输出**:
```
=== 定期存款演示 ===

--- 产品A: 3个月定期存款 ---
存款金额: 30,000 sat
期限: 3个月
到期日: 2025-03-15 00:00:00
年化利率: 3%

--- 产品B: 1年定期存款 ---
存款金额: 50,000 sat
期限: 1年
到期日: 2025-12-15 00:00:00
年化利率: 5%

--- 当前状态检查 ---
当前时间: 2024-12-15 00:00:00
🔒 3个月定存锁定中
  剩余: 90 天
🔒 1年定存锁定中
  剩余: 365 天
```

---

### 场景2: 遗产继承

```rust
fn inheritance_planning() -> Result<(), String> {
    println!("=== 遗产继承方案 ===\n");

    let owner = Wallet::new();
    let heir = Wallet::new();
    let lawyer = Wallet::new();

    println!("参与方:");
    println!("  所有人: {}", &owner.address[..16]);
    println!("  继承人: {}", &heir.address[..16]);
    println!("  律师: {}\n", &lawyer.address[..16]);

    let current_time = current_timestamp();

    // 方案: 1年无活动后，资产自动转给继承人
    println!("--- 方案设计 ---");
    println!("正常情况:");
    println!("  需要: 所有人 + 继承人 (2-of-2)");
    println!("  保护隐私，防止单方面转移\n");

    println!("紧急情况（1年后）:");
    println!("  所有人失联或去世");
    println!("  时间锁到期");
    println!("  继承人可独立操作\n");

    // 创建时间锁交易
    let one_year = 365 * 24 * 3600;
    let inheritance_time = current_time + one_year;
    let timelock = TimeLock::new_time_based(inheritance_time);

    println!("--- 时间锁配置 ---");
    println!("触发时间: {}", format_date(inheritance_time));
    println!("触发条件: 1年内无所有人签名的交易\n");

    // 定期检查（由律师执行）
    println!("--- 定期检查 ---");
    let last_activity = current_time;
    let inactive_period = current_time - last_activity;

    if inactive_period > one_year {
        if timelock.is_mature(current_time, 0) {
            println!("✓ 时间锁已触发");
            println!("✓ 继承程序启动");
            println!("✓ 资产可转移给继承人");
        }
    } else {
        let days_remaining = (one_year - inactive_period) / 86400;
        println!("🔒 正常状态");
        println!("距离继承触发还有 {} 天", days_remaining);
    }

    Ok(())
}
```

---

### 场景3: 工资发放

```rust
fn salary_payment_system() -> Result<(), String> {
    println!("=== 工资发放系统 ===\n");

    let company = Wallet::new();
    let employees: Vec<_> = (0..5)
        .map(|i| (format!("员工{}", i+1), Wallet::new()))
        .collect();

    let current_time = current_timestamp();

    println!("公司地址: {}", &company.address[..16]);
    println!("员工数量: {}\n", employees.len());

    // 每月1号发放工资
    println!("--- 工资发放计划 ---");

    for month in 1..=3 {
        // 计算下个月1号的时间戳
        let payment_date = calculate_first_day_of_month(current_time, month);
        let timelock = TimeLock::new_time_based(payment_date);

        println!("第{}月工资:", month);
        println!("  发放日期: {}", format_date(payment_date));

        if timelock.is_mature(current_time, 0) {
            println!("  状态: ✓ 可发放");

            for (name, wallet) in &employees {
                println!("    {} → {} sat", name, 10000);
            }
        } else {
            let days_until = (payment_date - current_time) / 86400;
            println!("  状态: 🔒 锁定中");
            println!("  倒计时: {} 天", days_until);
        }
        println!();
    }

    println!("--- 优势 ---");
    println!("✓ 自动化发放");
    println!("✓ 无法提前挪用");
    println!("✓ 员工可预期收入");
    println!("✓ 降低管理成本");

    Ok(())
}
```

---

### 场景4: 众筹退款

```rust
fn crowdfunding_refund() -> Result<(), String> {
    println!("=== 众筹退款机制 ===\n");

    let project_owner = Wallet::new();
    let backers: Vec<_> = (0..10).map(|_| Wallet::new()).collect();

    let current_time = current_timestamp();

    println!("--- 众筹项目 ---");
    println!("目标金额: 1,000,000 sat");
    println!("当前筹集: 500,000 sat");
    println!("截止日期: 30天后\n");

    // 30天后如果未达标，自动退款
    let deadline = current_time + 30 * 86400;
    let refund_timelock = TimeLock::new_time_based(deadline);

    println!("--- 退款时间锁 ---");
    println!("触发条件: 30天后未达标");
    println!("触发时间: {}", format_date(deadline));
    println!("退款方式: 自动返还支持者\n");

    // 检查状态
    if refund_timelock.is_mature(current_time, 0) {
        println!("--- 项目失败，执行退款 ---");
        for (i, backer) in backers.iter().enumerate() {
            println!("✓ 退款给支持者#{}: {} sat", i+1, 50000);
        }
    } else {
        let days_left = (deadline - current_time) / 86400;
        println!("--- 众筹进行中 ---");
        println!("剩余时间: {} 天", days_left);
        println!("仍需筹集: 500,000 sat");
    }

    Ok(())
}
```

---

## 高级用法

### 时间锁 + 多签

结合多签实现更复杂的逻辑：

```rust
fn timelock_multisig_combination() -> Result<(), String> {
    let owner = Wallet::new();
    let heir = Wallet::new();
    let lawyer = Wallet::new();

    // 正常：2-of-2（所有人 + 继承人）
    let normal_multisig = MultiSigAddress::new(
        2,
        vec![owner.public_key.clone(), heir.public_key.clone()]
    )?;

    // 1年后：2-of-3（任意两人）
    let emergency_multisig = MultiSigAddress::new(
        2,
        vec![owner.public_key, heir.public_key, lawyer.public_key]
    )?;

    let one_year = 365 * 24 * 3600;
    let timelock = TimeLock::new_time_based(current_timestamp() + one_year);

    println!("=== 时间锁 + 多签组合 ===");
    println!("\n正常时期（第一年）:");
    println!("  多签地址: {}", &normal_multisig.address[..16]);
    println!("  要求: 所有人 + 继承人 (2-of-2)");

    println!("\n紧急时期（一年后）:");
    println!("  多签地址: {}", &emergency_multisig.address[..16]);
    println!("  要求: 任意两人 (2-of-3)");
    println!("  可能组合:");
    println!("    - 所有人 + 继承人");
    println!("    - 所有人 + 律师");
    println!("    - 继承人 + 律师");

    Ok(())
}
```

---

## 技术细节

### nLockTime字段

在实际比特币交易中：

```rust
struct Transaction {
    version: u32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,  // 时间锁字段
}
```

**规则**:
```
if locktime < 500,000,000:
    # 区块高度模式
    if current_block_height >= locktime:
        可以确认
    else:
        拒绝

else:
    # 时间戳模式
    if current_timestamp >= locktime:
        可以确认
    else:
        拒绝
```

### nSequence与时间锁

要启用时间锁，nSequence必须 < 0xFFFFFFFF：

```rust
// 启用时间锁
input.sequence = 0xFFFFFFFD;

// 禁用时间锁（最终交易）
input.sequence = 0xFFFFFFFF;
```

---

## 安全考虑

### 1. 时间戳操纵

**问题**: 矿工可能操纵区块时间戳

**限制**:
- 时间戳不能早于前11个区块的中位数
- 不能晚于当前时间2小时以上

**建议**: 使用区块高度更可靠

### 2. 紧急情况

**问题**: 时间锁无法取消

**解决方案**:
```rust
// 方案1: 使用RBF在到期前替换
if !timelock.is_mature(...) && need_cancel {
    replace_with_non_locked_tx();
}

// 方案2: 双重支出（到期前）
create_alternative_tx_without_timelock();
```

### 3. 密钥丢失

**问题**: 到期前密钥丢失

**建议**:
- 使用多签降低风险
- 备份密钥
- 设置恢复机制

---

## 与CLTV/CSV的关系

### CheckLockTimeVerify (CLTV)

**BIP65引入**:
```
OP_CLTV操作码
锁定单个UTXO
更灵活
```

**nLockTime vs CLTV**:
```
nLockTime: 锁定整个交易
CLTV: 锁定单个输出（更灵活）
```

### CheckSequenceVerify (CSV)

**BIP112引入**:
```
OP_CSV操作码
相对时间锁
从UTXO创建时间开始计算
```

---

## 最佳实践

### 1. 选择正确的类型

```rust
// 精确日期：使用时间戳
let birthday = to_timestamp("2025-01-01");
let timelock = TimeLock::new_time_based(birthday);

// 相对延迟：使用区块高度
let blocks_1week = 1008;  // 约1周
let timelock = TimeLock::new_block_based(current_height + blocks_1week);
```

### 2. 用户友好的时间显示

```rust
fn display_timelock_status(timelock: &TimeLock, current_time: u64, current_height: u64) {
    if timelock.is_mature(current_time, current_height) {
        println!("✓ 已解锁");
    } else {
        if timelock.is_block_height {
            let blocks_left = timelock.locktime - current_height;
            let hours = blocks_left * 10 / 60;  // 约10分钟/块
            println!("🔒 锁定中，还需 {} 个区块 (约{}小时)", blocks_left, hours);
        } else {
            let seconds_left = timelock.locktime - current_time;
            let days = seconds_left / 86400;
            println!("🔒 锁定中，还需 {} 天", days);
        }
    }
}
```

### 3. 测试时间锁

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_timelock() {
        let current = 1000000;
        let future = 2000000;

        let timelock = TimeLock::new_time_based(future);

        // 未到期
        assert!(!timelock.is_mature(current, 0));

        // 已到期
        assert!(timelock.is_mature(future + 1, 0));
    }
}
```

---

## 参考资料

- [BIP65 - CLTV](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
- [BIP112 - CSV](https://github.com/bitcoin/bips/blob/master/bip-0112.mediawiki)
- [定期存款示例](../examples/timelock-savings.md)
- [多签教程](./multisig.md)

---

**总结**: 时间锁是实现延迟支付、智能合约的关键技术。合理使用可以实现定期存款、遗产继承、工资发放等多种应用。

[返回高级特性](./README.md)
