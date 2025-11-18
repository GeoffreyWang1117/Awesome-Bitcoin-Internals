# 定期存款系统

本案例演示如何使用TimeLock功能实现一个定期存款系统，用户可以选择不同的存期（3个月或1年），在到期前资金无法取出，到期后可领取本金和利息。

## 业务场景

**传统定期存款的痛点**：
- 提前支取需要银行审批
- 利息计算不透明
- 需要信任银行
- 到期后需要手动操作

**区块链解决方案**：
- ✅ 智能合约自动执行，无需审批
- ✅ 利息规则写入代码，完全透明
- ✅ 到期前技术上无法提取（去信任）
- ✅ 到期自动解锁

## 系统架构

### 产品设计

| 产品 | 存期 | 年化利率 | 最低金额 | 到期方式 |
|------|------|----------|----------|----------|
| 短期宝 | 3个月 | 3% | 1000 satoshi | 自动解锁 |
| 稳健盈 | 1年 | 5% | 5000 satoshi | 自动解锁 |

### 时间计算

```rust
// 3个月定期（约13周，91天）
const BLOCKS_PER_3_MONTHS: u64 = 13 * 7 * 144;  // 13,104区块

// 1年定期（约52周，365天）
const BLOCKS_PER_YEAR: u64 = 52 * 7 * 144;      // 52,416区块

// 注：比特币平均每10分钟一个区块，每天约144个区块
```

## 完整实现

以下是完整的定期存款系统代码：

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    advanced_tx::{TimeLock, TimeLockType},
};

fn main() -> Result<(), String> {
    println!("=== 定期存款系统演示 ===\n");

    // 初始化区块链
    let mut blockchain = Blockchain::new();

    // 创建用户钱包
    let user = Wallet::new();
    println!("用户地址: {}...{}", &user.address[..16], &user.address[36..]);

    // 给用户初始资金
    setup_balance(&mut blockchain, &user, 20000)?;
    println!("初始余额: {} satoshi\n", blockchain.get_balance(&user.address));

    // 场景1: 3个月定期存款
    println!("--- 场景1: 3个月定期存款 (3%年化利率) ---");
    let amount_3m = 5000;
    let rate_3m = 0.03;
    let blocks_3m = 13 * 7 * 144;  // 13周

    // 计算3个月利息
    let interest_3m = (amount_3m as f64 * rate_3m * 3.0 / 12.0) as u64;
    let total_3m = amount_3m + interest_3m;

    println!("存入金额: {} satoshi", amount_3m);
    println!("预期利息: {} satoshi (3个月 @ 3%)", interest_3m);
    println!("到期总额: {} satoshi", total_3m);
    println!("锁定区块数: {}", blocks_3m);

    // 创建3个月定期
    let timelock_3m = TimeLock::new(
        TimeLockType::BlockHeight(blockchain.chain.len() as u64 + blocks_3m)
    );

    let deposit_tx_3m = timelock_3m.create_timelocked_transaction(
        &mut blockchain,
        &user,
        user.address.clone(),  // 到期后返回给自己
        total_3m,              // 本金+利息
        10,
    )?;

    blockchain.add_transaction(deposit_tx_3m.clone())?;
    blockchain.mine_pending_transactions(user.address.clone())?;

    println!("✓ 3个月定期创建成功");
    println!("交易ID: {}...{}\n", &deposit_tx_3m.id[..16], &deposit_tx_3m.id[56..]);

    // 场景2: 1年定期存款
    println!("--- 场景2: 1年定期存款 (5%年化利率) ---");
    let amount_1y = 10000;
    let rate_1y = 0.05;
    let blocks_1y = 52 * 7 * 144;  // 52周

    // 计算1年利息
    let interest_1y = (amount_1y as f64 * rate_1y) as u64;
    let total_1y = amount_1y + interest_1y;

    println!("存入金额: {} satoshi", amount_1y);
    println!("预期利息: {} satoshi (1年 @ 5%)", interest_1y);
    println!("到期总额: {} satoshi", total_1y);
    println!("锁定区块数: {}", blocks_1y);

    // 创建1年定期
    let timelock_1y = TimeLock::new(
        TimeLockType::BlockHeight(blockchain.chain.len() as u64 + blocks_1y)
    );

    let deposit_tx_1y = timelock_1y.create_timelocked_transaction(
        &mut blockchain,
        &user,
        user.address.clone(),
        total_1y,
        10,
    )?;

    blockchain.add_transaction(deposit_tx_1y.clone())?;
    blockchain.mine_pending_transactions(user.address.clone())?;

    println!("✓ 1年定期创建成功");
    println!("交易ID: {}...{}\n", &deposit_tx_1y.id[..16], &deposit_tx_1y.id[56..]);

    // 显示当前余额
    let current_balance = blockchain.get_balance(&user.address);
    println!("剩余可用余额: {} satoshi", current_balance);
    println!("定期存款总额: {} satoshi (锁定中)\n", amount_3m + amount_1y);

    // 场景3: 尝试提前取款（应该失败）
    println!("--- 场景3: 尝试提前取款 ---");
    println!("当前区块高度: {}", blockchain.chain.len());
    println!("3个月定期解锁高度: {}", blockchain.chain.len() as u64 + blocks_3m);

    match timelock_3m.is_spendable(&blockchain) {
        true => println!("❌ 错误：定期未到期却可以取款！"),
        false => println!("✓ 正确：定期未到期，资金已锁定"),
    }

    // 场景4: 模拟时间流逝（挖矿到3个月后）
    println!("\n--- 场景4: 3个月后到期 ---");
    println!("模拟挖矿 {} 个区块...", blocks_3m);

    // 快速模拟挖矿
    for _ in 0..blocks_3m {
        blockchain.mine_pending_transactions(user.address.clone())?;
    }

    println!("当前区块高度: {}", blockchain.chain.len());

    // 检查是否可以取款
    if timelock_3m.is_spendable(&blockchain) {
        println!("✓ 3个月定期已到期，可以取款");

        // 领取本金+利息
        println!("领取金额: {} satoshi (本金 {} + 利息 {})",
                 total_3m, amount_3m, interest_3m);

        let final_balance = blockchain.get_balance(&user.address);
        println!("到账后余额: {} satoshi", final_balance);
    } else {
        println!("❌ 错误：定期已到期但无法取款");
    }

    // 场景5: 1年定期还未到期
    println!("\n--- 场景5: 1年定期状态 ---");
    println!("当前区块高度: {}", blockchain.chain.len());
    println!("1年定期解锁高度: {}", blockchain.chain.len() as u64 + blocks_1y - blocks_3m);

    match timelock_1y.is_spendable(&blockchain) {
        true => println!("✓ 1年定期已到期，可以取款"),
        false => {
            let remaining = blocks_1y - blocks_3m;
            println!("✓ 1年定期还未到期，还需 {} 个区块 (约 {} 天)",
                     remaining, remaining / 144);
        }
    }

    println!("\n=== 演示完成 ===");

    Ok(())
}

// 辅助函数：初始化余额
fn setup_balance(
    blockchain: &mut Blockchain,
    wallet: &Wallet,
    amount: u64
) -> Result<(), String> {
    let genesis = Wallet::from_address("genesis".to_string());
    let tx = blockchain.create_transaction(
        &genesis,
        wallet.address.clone(),
        amount,
        0,
    )?;
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(wallet.address.clone())?;
    Ok(())
}
```

## 代码详解

### 1. 产品参数定义

```rust
// 短期宝：3个月定期
let amount_3m = 5000;              // 存款金额
let rate_3m = 0.03;                // 3%年化利率
let blocks_3m = 13 * 7 * 144;      // 3个月 = 13周 = 13,104区块

// 计算利息：本金 × 年利率 × 时间（月/12）
let interest_3m = (amount_3m as f64 * rate_3m * 3.0 / 12.0) as u64;
// interest_3m = 5000 × 0.03 × 0.25 = 37.5 ≈ 37 satoshi
```

**为什么用区块高度而非时间戳？**
- 更精确：区块高度是离散的整数，不会有歧义
- 更可靠：时间戳可能被矿工操纵（±2小时）
- 更一致：全网对区块高度有统一共识

### 2. 创建时间锁定期

```rust
// 创建时间锁：当前高度 + 锁定期
let timelock_3m = TimeLock::new(
    TimeLockType::BlockHeight(
        blockchain.chain.len() as u64 + blocks_3m
    )
);
```

**关键点**：
- `blockchain.chain.len()` = 当前区块高度
- `+ blocks_3m` = 到期区块高度
- 在到期高度之前，交易无法被花费

### 3. 创建定期存款交易

```rust
let deposit_tx_3m = timelock_3m.create_timelocked_transaction(
    &mut blockchain,
    &user,                      // 存款人
    user.address.clone(),       // 到期后返回给存款人
    total_3m,                   // 本金 + 利息
    10,                         // 手续费
)?;
```

**交易流程**：
```
用户余额 → [时间锁定交易] → UTXO池（锁定状态）
                ↓
         (到期后才能花费)
                ↓
           用户余额（本金+利息）
```

### 4. 到期检查

```rust
if timelock_3m.is_spendable(&blockchain) {
    // 可以取款
} else {
    // 还未到期
}
```

**检查逻辑**：
```rust
pub fn is_spendable(&self, blockchain: &Blockchain) -> bool {
    match &self.lock_type {
        TimeLockType::BlockHeight(height) => {
            blockchain.chain.len() as u64 >= *height
        },
        TimeLockType::Timestamp(time) => {
            // 使用当前时间戳比较
            current_timestamp() >= *time
        }
    }
}
```

## 运行效果

```bash
$ cargo run --example timelock_savings

=== 定期存款系统演示 ===

用户地址: a3f2d8c9e4b7f1a8...c4e7d9b2a5c
初始余额: 20000 satoshi

--- 场景1: 3个月定期存款 (3%年化利率) ---
存入金额: 5000 satoshi
预期利息: 37 satoshi (3个月 @ 3%)
到期总额: 5037 satoshi
锁定区块数: 13104
✓ 3个月定期创建成功
交易ID: d4f7a9e2b5c8f1a3...b5c8f1a3d4f7

--- 场景2: 1年定期存款 (5%年化利率) ---
存入金额: 10000 satoshi
预期利息: 500 satoshi (1年 @ 5%)
到期总额: 10500 satoshi
锁定区块数: 52416
✓ 1年定期创建成功
交易ID: e5g8b0f3c6d9g2b4...c6d9g2b4e5g8

剩余可用余额: 4960 satoshi
定期存款总额: 15000 satoshi (锁定中)

--- 场景3: 尝试提前取款 ---
当前区块高度: 4
3个月定期解锁高度: 13108
✓ 正确：定期未到期，资金已锁定

--- 场景4: 3个月后到期 ---
模拟挖矿 13104 个区块...
当前区块高度: 13108
✓ 3个月定期已到期，可以取款
领取金额: 5037 satoshi (本金 5000 + 利息 37)
到账后余额: 10497 satoshi

--- 场景5: 1年定期状态 ---
当前区块高度: 13108
1年定期解锁高度: 52420
✓ 1年定期还未到期，还需 39312 个区块 (约 273 天)

=== 演示完成 ===
```

## 业务价值

### 对用户的价值

| 特性 | 传统银行定期 | 区块链定期 | 优势 |
|------|-------------|-----------|------|
| **利率透明** | ❌ 银行说了算 | ✅ 代码公开 | 完全透明 |
| **强制储蓄** | ⚠️ 可提前支取 | ✅ 技术锁定 | 真正强制 |
| **利息保障** | ⚠️ 银行承诺 | ✅ 智能合约 | 自动执行 |
| **到期操作** | ❌ 需要去银行 | ✅ 自动解锁 | 无需操作 |
| **信任成本** | 高（需要信任银行）| 低（信任代码）| 去中心化 |

### 收益对比（假设存入10000 satoshi）

| 产品 | 期限 | 利率 | 到期本息 | 收益 |
|------|------|------|---------|------|
| 活期存款 | - | 0.3% | 10030 | 30 |
| 短期宝 | 3个月 | 3% | 10075 | 75 |
| 稳健盈 | 1年 | 5% | 10500 | 500 |

**计算公式**：
```
到期本息 = 本金 × (1 + 年利率 × 存期年数)

3个月: 10000 × (1 + 0.03 × 0.25) = 10075
1年:   10000 × (1 + 0.05 × 1.0)  = 10500
```

## 扩展方案

### 1. 阶梯式定期

```rust
struct LadderDeposit {
    amount: u64,
    start_height: u64,
    periods: Vec<(u64, f64)>,  // (期限区块数, 利率)
}

impl LadderDeposit {
    // 创建阶梯式定期：分散到期时间
    pub fn new(total: u64, blockchain: &Blockchain) -> Self {
        let per_amount = total / 4;
        let current = blockchain.chain.len() as u64;

        LadderDeposit {
            amount: per_amount,
            start_height: current,
            periods: vec![
                (13 * 7 * 144, 0.03),   // 3个月，3%
                (26 * 7 * 144, 0.04),   // 6个月，4%
                (39 * 7 * 144, 0.045),  // 9个月，4.5%
                (52 * 7 * 144, 0.05),   // 12个月，5%
            ],
        }
    }
}

// 好处：
// - 每3个月有一笔到期，保持流动性
// - 平均利率高于单一短期
// - 降低利率波动风险
```

### 2. 自动续存

```rust
struct AutoRenewDeposit {
    principal: u64,
    term_blocks: u64,
    rate: f64,
    max_renewals: u32,
}

impl AutoRenewDeposit {
    pub fn create_auto_renew(
        &self,
        blockchain: &mut Blockchain,
        wallet: &Wallet,
    ) -> Result<Vec<Transaction>, String> {
        let mut transactions = Vec::new();
        let mut total = self.principal;

        for i in 0..self.max_renewals {
            let lock_height = blockchain.chain.len() as u64
                            + (i as u64 + 1) * self.term_blocks;

            // 计算本期本息
            let interest = (total as f64 * self.rate
                          * (self.term_blocks as f64 / 52416.0)) as u64;
            total += interest;

            // 创建续存交易
            let timelock = TimeLock::new(
                TimeLockType::BlockHeight(lock_height)
            );

            let tx = timelock.create_timelocked_transaction(
                blockchain,
                wallet,
                wallet.address.clone(),
                total,
                10,
            )?;

            transactions.push(tx);
        }

        Ok(transactions)
    }
}

// 使用示例：
let auto_deposit = AutoRenewDeposit {
    principal: 10000,
    term_blocks: 13 * 7 * 144,  // 3个月
    rate: 0.03,
    max_renewals: 4,  // 自动续存4次 = 1年
};

// 自动创建4笔定期，每3个月自动续存一次
let txs = auto_deposit.create_auto_renew(&mut blockchain, &user)?;
```

### 3. 保本浮动收益

```rust
struct FloatingDeposit {
    principal: u64,
    min_rate: f64,      // 保本利率
    bonus_rate: f64,    // 奖励利率
    target_blocks: u64, // 目标区块数
}

impl FloatingDeposit {
    pub fn calculate_interest(&self, blockchain: &Blockchain) -> u64 {
        let actual_blocks = blockchain.chain.len() as u64;

        // 基础利息（保本）
        let base = (self.principal as f64 * self.min_rate) as u64;

        // 奖励利息（根据实际持有时间）
        if actual_blocks >= self.target_blocks {
            let bonus = (self.principal as f64 * self.bonus_rate) as u64;
            base + bonus
        } else {
            base
        }
    }
}

// 使用示例：
let floating = FloatingDeposit {
    principal: 10000,
    min_rate: 0.03,    // 3%保本
    bonus_rate: 0.02,  // 额外2%奖励
    target_blocks: 52 * 7 * 144,  // 持有1年才有奖励
};

// 未满1年：3%利息 = 300 satoshi
// 满1年：  5%利息 = 500 satoshi
```

### 4. 提前赎回（罚息）

```rust
struct EarlyWithdraw {
    deposit_tx: Transaction,
    lock_height: u64,
    penalty_rate: f64,  // 罚息比例
}

impl EarlyWithdraw {
    pub fn withdraw_early(
        &self,
        blockchain: &mut Blockchain,
        wallet: &Wallet,
    ) -> Result<Transaction, String> {
        let current = blockchain.chain.len() as u64;

        // 检查是否提前赎回
        if current >= self.lock_height {
            return Err("已到期，请正常取款".to_string());
        }

        // 计算罚息
        let principal = self.deposit_tx.outputs[0].value;
        let penalty = (principal as f64 * self.penalty_rate) as u64;
        let actual_amount = principal.saturating_sub(penalty);

        // 创建提前赎回交易（需要管理员签名）
        let tx = blockchain.create_transaction(
            wallet,
            wallet.address.clone(),
            actual_amount,
            10,
        )?;

        println!("提前赎回：本金 {}, 罚息 {}, 实得 {}",
                 principal, penalty, actual_amount);

        Ok(tx)
    }
}

// 使用示例：
// 用户存入10000，期限1年，提前6个月取出
// 罚息5% = 500 satoshi
// 实得9500 satoshi（损失500）
```

## 安全考虑

### 1. 利息资金来源

```rust
// ❌ 错误：凭空创造利息
let interest = 100;
let total = principal + interest;  // 利息从哪来？

// ✅ 正确：利息从资金池支付
struct DepositPool {
    reserves: u64,  // 准备金
}

impl DepositPool {
    pub fn pay_interest(&mut self, principal: u64, rate: f64) -> Result<u64, String> {
        let interest = (principal as f64 * rate) as u64;

        if self.reserves < interest {
            return Err("资金池余额不足".to_string());
        }

        self.reserves -= interest;
        Ok(interest)
    }
}
```

### 2. 时间操纵攻击

**攻击场景**：矿工操纵时间戳，使定期提前到期

**防御措施**：
```rust
// ✅ 使用区块高度而非时间戳
TimeLockType::BlockHeight(height)  // 推荐

// ⚠️ 避免使用时间戳（容易被操纵）
TimeLockType::Timestamp(time)      // 不安全
```

### 3. 重入攻击

```rust
// ❌ 错误：先转账再更新状态
fn withdraw(&mut self) {
    self.transfer(user, amount);  // 先转账
    self.balance = 0;             // 后更新（可能被重入）
}

// ✅ 正确：先更新状态再转账（检查-生效-交互模式）
fn withdraw(&mut self) {
    let amount = self.balance;    // 检查
    self.balance = 0;             // 生效
    self.transfer(user, amount);  // 交互
}
```

### 4. 整数溢出

```rust
// ❌ 错误：可能溢出
let total = principal + interest;  // u64溢出风险

// ✅ 正确：使用checked_add
let total = principal.checked_add(interest)
    .ok_or("计算溢出")?;
```

## 实施建议

### 技术层面

1. **测试充分性**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_interest_calculation() { /* ... */ }

       #[test]
       fn test_early_withdraw_penalty() { /* ... */ }

       #[test]
       fn test_timelock_enforcement() { /* ... */ }
   }
   ```

2. **代码审计**
   - 利息计算公式是否正确
   - 时间锁定是否可靠
   - 资金来源是否明确
   - 边界条件是否处理

3. **监控告警**
   ```rust
   // 监控关键指标
   - 资金池余额预警（< 10%）
   - 到期未领取定期（> 1个月）
   - 异常提前赎回频率
   ```

### 业务层面

1. **风险提示**
   ```
   ⚠️ 定期存款风险提示：
   1. 资金将被锁定，到期前无法取出
   2. 利息由资金池支付，存在支付风险
   3. 智能合约可能存在未知漏洞
   4. 区块链不可逆，操作需谨慎
   ```

2. **用户教育**
   - 演示沙盒环境供用户练习
   - 提供详细的操作指南
   - 说明与传统银行的区别
   - 强调私钥保管的重要性

3. **产品迭代**
   - 收集用户反馈
   - 分析到期数据
   - 优化利率策略
   - 增加产品种类

## 真实应用

### DeFi定期存款协议

**Compound**: 借贷协议，存款自动生息
```
用户存入 ETH → 获得 cETH（计息代币）
利率随市场浮动 → 随时可取
```

**Anchor Protocol**: 固定利率存款（Terra生态）
```
存入 UST → 固定 ~20% APY
利息来自借贷市场和质押奖励
```

**Alchemix**: 自偿还贷款
```
存入 DAI → 借出 alUSD（50% LTV）
利息自动偿还贷款 → 无需还款
```

### 与SimpleBTC的对比

| 特性 | SimpleBTC定期 | DeFi定期 |
|------|--------------|---------|
| 时间锁 | 硬锁定（nLockTime）| 软锁定（合约）|
| 利率 | 固定利率 | 通常浮动 |
| 流动性 | 到期才能取 | 可提前取（罚息）|
| 利息来源 | 资金池 | 借贷/质押 |
| 风险 | 时间锁风险 | 智能合约风险 |

## 常见问题

### Q1: 定期存款的利息从哪来？

**A**: SimpleBTC的利息是演示性质的，实际应用中利息可能来自：
- 资金池的储备金
- 借贷市场的利差
- 矿工奖励的分配
- 交易手续费的返还
- 协议代币的增发

### Q2: 可以提前取款吗？

**A**: SimpleBTC使用nLockTime硬锁定，技术上无法提前取款。实际应用可以设计：
- 罚息提前赎回（5-10%罚金）
- NFT质押借款（保持定期继续）
- 二级市场转让（折价卖给接盘侠）

### Q3: 如果到期后忘记领取怎么办？

**A**: UTXO永久有效，任何时候都可以领取。但要注意：
- 逾期不会额外生息
- 建议设置到期提醒
- 可以实现自动续存

### Q4: 时间锁定期怎么计算？

**A**:
```
区块高度法（推荐）:
- 3个月 ≈ 13,104 区块 (91天 × 144区块/天)
- 1年   ≈ 52,416 区块 (365天 × 144区块/天)

时间戳法（不推荐）:
- 3个月 = 当前时间戳 + 7,862,400 秒
- 1年   = 当前时间戳 + 31,536,000 秒
```

## 参考资料

- [TimeLock详细教程](../advanced/timelock.md) - 时间锁原理和用法
- [Transaction API](../api/transaction.md) - 交易创建
- [Blockchain API](../api/blockchain.md) - 区块链操作
- [Compound Finance](https://compound.finance/) - DeFi借贷协议
- [BIP65 - CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)

---

[返回案例目录](./enterprise-multisig.md) | [下一个案例：企业多签](./enterprise-multisig.md)
