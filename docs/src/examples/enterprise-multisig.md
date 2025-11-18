# 企业多签钱包实战

本案例演示如何使用2-of-3多签管理企业资金。

## 场景描述

某科技公司需要管理公司的比特币资产，要求：
- 三位高管（CEO、CFO、CTO）各持一个密钥
- 任意两位高管同意即可转账
- 防止单人滥用或失控
- 某位高管不在也能正常运作

## 运行示例

```bash
cargo run --example enterprise_multisig
```

## 代码详解

### 1. 初始化

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::MultiSigAddress,
};

fn main() -> Result<(), String> {
    println!("=== 企业多签钱包演示 ===\n");

    // 创建区块链
    let mut blockchain = Blockchain::new();

    // 创建三位高管的钱包
    let ceo = Wallet::new();
    let cfo = Wallet::new();
    let cto = Wallet::new();

    println!("✓ 创建企业高管钱包:");
    println!("  CEO: {}", &ceo.address[..20]);
    println!("  CFO: {}", &cfo.address[..20]);
    println!("  CTO: {}\n", &cto.address[..20]);
```

### 2. 创建多签地址

```rust
    // 创建2-of-3多签地址
    let company_multisig = MultiSigAddress::new(
        2,  // 需要2个签名
        vec![
            ceo.public_key.clone(),
            cfo.public_key.clone(),
            cto.public_key.clone(),
        ]
    ).expect("创建多签地址失败");

    println!("✓ 公司多签地址已创建:");
    println!("  地址: {}", &company_multisig.address[..20]);
    println!("  类型: {}-of-{} 多签",
        company_multisig.required_sigs,
        company_multisig.total_keys);
    println!("  规则: 任意2位高管签名即可转账\n");
```

**关键点**：
- `required_sigs = 2`: 需要2个签名
- `total_keys = 3`: 共3个密钥
- 任意两位高管的组合都可以：CEO+CFO、CEO+CTO、CFO+CTO

### 3. 注入初始资金

```rust
    // 为公司多签地址注入资金
    println!("--- 场景1: 公司获得融资 ---");

    let investor = Wallet::new();
    println!("投资人地址: {}\n", &investor.address[..20]);

    // 创建融资交易（从创世地址）
    let funding_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        company_multisig.address.clone(),
        100000,  // 10万 satoshi
        0,
    )?;

    blockchain.add_transaction(funding_tx)?;
    blockchain.mine_pending_transactions(investor.address.clone())?;

    let company_balance = blockchain.get_balance(&company_multisig.address);
    println!("✓ 融资完成");
    println!("  公司账户余额: {} satoshi\n", company_balance);
```

### 4. 场景演示：正常支出

```rust
    println!("--- 场景2: 正常支出（CEO + CFO批准）---");

    let supplier = Wallet::new();
    println!("供应商地址: {}\n", &supplier.address[..20]);

    // 模拟多签流程
    let payment_amount = 30000;
    let payment_data = format!("{}{}", company_multisig.address, supplier.address);

    // 步骤1: CEO签名
    let ceo_signature = ceo.sign(&payment_data);
    println!("✓ CEO已审批并签名");

    // 步骤2: CFO签名
    let cfo_signature = cfo.sign(&payment_data);
    println!("✓ CFO已审批并签名");

    // 步骤3: 验证签名数量
    let signatures = vec![ceo_signature, cfo_signature];

    if signatures.len() >= company_multisig.required_sigs {
        println!("✓ 签名数量满足要求 (2/3)");
        println!("✓ 交易可以执行\n");

        // 实际转账
        let payment_tx = blockchain.create_transaction(
            &Wallet::from_address(company_multisig.address.clone()),
            supplier.address.clone(),
            payment_amount,
            100,
        )?;

        blockchain.add_transaction(payment_tx)?;
        blockchain.mine_pending_transactions(ceo.address.clone())?;

        println!("✓ 支付完成");
        println!("  支付金额: {} satoshi", payment_amount);
        println!("  公司余额: {} satoshi\n",
            blockchain.get_balance(&company_multisig.address));
    }
```

**工作流程**：
1. CEO发起支付请求
2. CEO使用私钥签名
3. CFO审核并签名
4. 系统验证签名数量（2个 ≥ 要求的2个）
5. 执行转账

### 5. 场景演示：CEO不在场

```rust
    println!("--- 场景3: CEO出差期间的紧急支出（CFO + CTO）---");

    let emergency_vendor = Wallet::new();
    println!("紧急供应商: {}\n", &emergency_vendor.address[..20]);

    let emergency_amount = 20000;
    let emergency_data = format!("{}{}",
        company_multisig.address, emergency_vendor.address);

    println!("CEO正在出差，无法联系");
    println!("CFO和CTO决定批准紧急支出\n");

    // CFO签名
    let cfo_sig = cfo.sign(&emergency_data);
    println!("✓ CFO已签名");

    // CTO签名
    let cto_sig = cto.sign(&emergency_data);
    println!("✓ CTO已签名");

    let emergency_sigs = vec![cfo_sig, cto_sig];

    if emergency_sigs.len() >= company_multisig.required_sigs {
        println!("✓ 签名满足要求 (2/3)");
        println!("✓ 即使CEO不在，业务仍可正常运作\n");

        // 执行转账
        let emergency_tx = blockchain.create_transaction(
            &Wallet::from_address(company_multisig.address.clone()),
            emergency_vendor.address,
            emergency_amount,
            100,
        )?;

        blockchain.add_transaction(emergency_tx)?;
        blockchain.mine_pending_transactions(cfo.address.clone())?;

        println!("✓ 紧急支付完成");
        println!("  最终余额: {} satoshi\n",
            blockchain.get_balance(&company_multisig.address));
    }

    Ok(())
}
```

## 输出示例

```
=== 企业多签钱包演示 ===

✓ 创建企业高管钱包:
  CEO: a3f2d8c9e4b7f1a8...
  CFO: b9e4c7d2a3f1e8b6...
  CTO: c8f1e9d3b4a7c2e5...

✓ 公司多签地址已创建:
  地址: 3Mf2d8c9e4b7f1a8...
  类型: 2-of-3 多签
  规则: 任意2位高管签名即可转账

--- 场景1: 公司获得融资 ---
投资人地址: d7c2e8f3a9b1d4c6...

区块已挖出: 0003ab4f9c2d...
✓ 融资完成
  公司账户余额: 100000 satoshi

--- 场景2: 正常支出（CEO + CFO批准）---
供应商地址: e6d1f8c2b9a3e7d4...

✓ CEO已审批并签名
✓ CFO已审批并签名
✓ 签名数量满足要求 (2/3)
✓ 交易可以执行

区块已挖出: 0007c3e8d1a9...
✓ 支付完成
  支付金额: 30000 satoshi
  公司余额: 69900 satoshi

--- 场景3: CEO出差期间的紧急支出（CFO + CTO）---
紧急供应商: f5e2d9c3a8b7f1e6...

CEO正在出差，无法联系
CFO和CTO决定批准紧急支出

✓ CFO已签名
✓ CTO已签名
✓ 签名满足要求 (2/3)
✓ 即使CEO不在，业务仍可正常运作

区块已挖出: 000ab7e4f2c8...
✓ 紧急支付完成
  最终余额: 49800 satoshi
```

## 业务价值

### 1. 安全性

| 传统单签 | 企业多签 |
|---------|---------|
| ❌ CEO私钥被盗，全部资金丢失 | ✅ 需要2个密钥，单个被盗无风险 |
| ❌ 单点故障 | ✅ 分散风险 |
| ❌ 内部舞弊风险高 | ✅ 需要两人合谋才可能 |

### 2. 业务连续性

| 场景 | 传统方案 | 多签方案 |
|------|---------|---------|
| CEO休假 | ❌ 业务暂停 | ✅ CFO+CTO继续运作 |
| 高管离职 | ❌ 需要全部转移资金 | ✅ 更换一个密钥即可 |
| 紧急支出 | ❌ 找不到唯一的密钥持有人 | ✅ 任意2人即可批准 |

### 3. 合规性

```
审计追踪：
- 每笔交易需要2个签名
- 明确记录谁批准了什么
- 符合内部控制要求
- 满足财务审计标准
```

## 扩展方案

### 分级授权

```rust
// 小额：经理级 2-of-3
if amount < 10000 {
    let managers_multisig = MultiSigAddress::new(2, manager_keys)?;
}

// 中额：高管级 2-of-3
else if amount < 100000 {
    let exec_multisig = MultiSigAddress::new(2, exec_keys)?;
}

// 大额：董事会 5-of-9
else {
    let board_multisig = MultiSigAddress::new(5, board_keys)?;
}
```

### 时间锁保护

```rust
use bitcoin_simulation::advanced_tx::TimeLock;

// 大额转账需要24小时延迟
let timelock = TimeLock::new_time_based(
    current_time() + 24 * 3600
);

// 延迟期间可以取消
// 防止胁迫转账
```

### 紧急恢复

```rust
// 正常：2-of-3
let normal_multisig = MultiSigAddress::new(
    2,
    vec![ceo_key, cfo_key, cto_key]
)?;

// 紧急（2个密钥丢失）：律师托管的恢复密钥
let recovery_multisig = MultiSigAddress::new(
    1,
    vec![lawyer_key]  // 需要法律文件证明
)?;
```

## 实施建议

### 1. 密钥管理

```
CEO密钥：
  - 主密钥：手机热钱包（日常签名）
  - 备份：硬件钱包（保险柜）

CFO密钥：
  - 主密钥：电脑热钱包（办公室）
  - 备份：纸钱包（银行保险箱）

CTO密钥：
  - 主密钥：硬件钱包（随身携带）
  - 备份：加密U盘（异地存储）
```

### 2. 操作流程

```
1. 发起人创建转账申请
2. 发起人签名
3. 通知第二审批人
4. 第二审批人审核并签名
5. 系统自动验证签名数量
6. 执行交易并通知所有人
7. 记录审计日志
```

### 3. 安全检查清单

- [ ] 密钥分散存储
- [ ] 定期测试恢复流程
- [ ] 备份所有密钥
- [ ] 设置金额阈值
- [ ] 启用交易通知
- [ ] 定期审计交易记录
- [ ] 制定密钥丢失应急预案
- [ ] 培训所有密钥持有人

## 相关资源

- [多重签名详解](../advanced/multisig.md)
- [MultiSig API文档](../api/multisig.md)
- [托管服务案例](./escrow.md)

## 总结

企业多签钱包通过2-of-3机制实现了：

✅ **安全性** - 没有单点故障
✅ **灵活性** - 任意两人可批准
✅ **连续性** - 某人不在仍可运作
✅ **合规性** - 符合内部控制
✅ **透明性** - 所有操作可追溯

是企业管理数字资产的最佳实践！

---

[查看完整源代码](../../examples/enterprise_multisig.rs)
