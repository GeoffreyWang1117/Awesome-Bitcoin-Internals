# 托管服务实战

本案例演示如何使用2-of-3多签实现比特币托管服务。

## 场景描述

在电商交易中，买卖双方互不信任，需要第三方托管：
- 买家担心：付款后卖家不发货
- 卖家担心：发货后买家不付款
- 解决方案：资金托管在2-of-3多签地址

**参与方**:
- 买家（Buyer）
- 卖家（Seller）
- 仲裁员（Arbitrator）

**规则**:
- 正常交易：买家 + 卖家签名 → 资金给卖家
- 争议处理：买家/卖家 + 仲裁员 → 按仲裁结果

## 运行示例

```bash
cargo run --example escrow_service
```

## 代码详解

### 1. 初始化参与方

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::MultiSigAddress,
};

fn main() -> Result<(), String> {
    println!("=== 比特币托管服务演示 ===\n");

    // 创建区块链
    let mut blockchain = Blockchain::new();

    // 创建参与方钱包
    let buyer = Wallet::new();
    let seller = Wallet::new();
    let arbitrator = Wallet::new();

    println!("✓ 参与方已创建:");
    println!("  买家: {}", &buyer.address[..20]);
    println!("  卖家: {}", &seller.address[..20]);
    println!("  仲裁员: {}\n", &arbitrator.address[..20]);
```

### 2. 创建托管多签地址

```rust
    // 创建2-of-3托管多签地址
    let escrow_multisig = MultiSigAddress::new(
        2,  // 需要2个签名
        vec![
            buyer.public_key.clone(),
            seller.public_key.clone(),
            arbitrator.public_key.clone(),
        ]
    ).expect("创建多签地址失败");

    println!("✓ 托管地址已创建:");
    println!("  地址: {}", &escrow_multisig.address[..20]);
    println!("  类型: {}-of-{} 多签",
        escrow_multisig.required_sigs,
        escrow_multisig.total_keys);
    println!("  规则: 任意2方签名即可");
    println!("  可能组合:");
    println!("    - 买家 + 卖家（正常交易）");
    println!("    - 买家 + 仲裁员（买家退款）");
    println!("    - 卖家 + 仲裁员（卖家收款）\n");
```

**关键点**:
- 2-of-3确保没有单方控制
- 正常情况买卖双方自行解决
- 争议时仲裁员介入

### 3. 买家存入资金

```rust
    println!("--- 场景1: 买家存入托管资金 ---");

    // 买家获得初始资金
    let funding_tx = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        buyer.address.clone(),
        100000,  // 10万 satoshi
        0,
    )?;

    blockchain.add_transaction(funding_tx)?;
    blockchain.mine_pending_transactions(buyer.address.clone())?;

    let buyer_initial = blockchain.get_balance(&buyer.address);
    println!("买家余额: {} sat\n", buyer_initial);

    // 买家将货款转入托管地址
    let escrow_amount = 50000;  // 5万 sat
    println!("商品价格: {} sat", escrow_amount);
    println!("买家将货款转入托管地址...\n");

    let deposit_tx = blockchain.create_transaction(
        &buyer,
        escrow_multisig.address.clone(),
        escrow_amount,
        100,  // 手续费
    )?;

    blockchain.add_transaction(deposit_tx)?;
    blockchain.mine_pending_transactions(buyer.address.clone())?;

    let escrow_balance = blockchain.get_balance(&escrow_multisig.address);
    println!("✓ 资金已托管");
    println!("  托管金额: {} sat", escrow_balance);
    println!("  买家余额: {} sat\n", blockchain.get_balance(&buyer.address));
```

**流程**:
1. 买家先获得资金
2. 买家将货款转入托管地址
3. 资金被锁定在多签地址中
4. 卖家看到托管成功后发货

### 4. 场景A：正常交易完成

```rust
    println!("--- 场景2A: 正常交易（买家满意）---");
    println!("卖家已发货");
    println!("买家收到货物，确认满意\n");

    // 买家和卖家都签名，释放资金给卖家
    let payment_amount = escrow_balance - 50;  // 扣除手续费
    let payment_data = format!("{}{}{}",
        escrow_multisig.address,
        seller.address,
        payment_amount);

    println!("签名过程:");
    // 买家签名
    let buyer_signature = buyer.sign(&payment_data);
    println!("  ✓ 买家已签名（确认收货）");

    // 卖家签名
    let seller_signature = seller.sign(&payment_data);
    println!("  ✓ 卖家已签名（同意收款）");

    // 验证签名数量
    let signatures = vec![buyer_signature, seller_signature];

    if signatures.len() >= escrow_multisig.required_sigs {
        println!("\n✓ 签名满足要求 (2/3)");
        println!("✓ 释放资金给卖家\n");

        // 创建支付交易
        let payment_tx = blockchain.create_transaction(
            &Wallet::from_address(escrow_multisig.address.clone()),
            seller.address.clone(),
            payment_amount,
            50,
        )?;

        blockchain.add_transaction(payment_tx)?;
        blockchain.mine_pending_transactions(seller.address.clone())?;

        println!("=== 交易完成 ===");
        println!("卖家余额: {} sat", blockchain.get_balance(&seller.address));
        println!("托管余额: {} sat", blockchain.get_balance(&escrow_multisig.address));
    }
```

**正常流程**:
1. 卖家发货
2. 买家收货确认
3. 买家签名（确认满意）
4. 卖家签名（同意收款）
5. 2个签名满足要求
6. 资金释放给卖家

### 5. 场景B：争议处理

```rust
    println!("\n--- 场景2B: 争议处理（货物有问题）---");

    // 重新创建场景（假设）
    let escrow_multisig_dispute = MultiSigAddress::new(
        2,
        vec![
            buyer.public_key.clone(),
            seller.public_key.clone(),
            arbitrator.public_key,
        ]
    )?;

    println!("买家: 货物与描述不符，要求退款");
    println!("卖家: 货物没问题，拒绝退款");
    println!("仲裁员介入调查...\n");

    println!("仲裁结果:");
    println!("  经核实，货物确实存在问题");
    println!("  判决：退款给买家\n");

    // 买家 + 仲裁员签名
    let refund_data = format!("{}{}{}",
        escrow_multisig_dispute.address,
        buyer.address,
        payment_amount);

    println!("签名过程:");
    let buyer_sig_dispute = buyer.sign(&refund_data);
    println!("  ✓ 买家签名（同意退款）");

    let arbitrator_sig = arbitrator.sign(&refund_data);
    println!("  ✓ 仲裁员签名（执行判决）");

    let dispute_sigs = vec![buyer_sig_dispute, arbitrator_sig];

    if dispute_sigs.len() >= escrow_multisig_dispute.required_sigs {
        println!("\n✓ 签名满足要求 (2/3)");
        println!("✓ 执行退款\n");

        println!("=== 争议解决 ===");
        println!("退款给买家: {} sat", payment_amount);
        println!("仲裁费: 50 sat（从托管金扣除）");
    }

    Ok(())
}
```

**争议流程**:
1. 买家投诉货物问题
2. 卖家拒绝退款
3. 仲裁员介入调查
4. 仲裁员做出判决
5. 买家 + 仲裁员签名
6. 资金退还买家

---

## 输出示例

```
=== 比特币托管服务演示 ===

✓ 参与方已创建:
  买家: a3f2d8c9e4b7f1a8...
  卖家: b9e4c7d2a3f1e8b6...
  仲裁员: c8f1e9d3b4a7c2e5...

✓ 托管地址已创建:
  地址: 3Mf2d8c9e4b7f1a8...
  类型: 2-of-3 多签
  规则: 任意2方签名即可
  可能组合:
    - 买家 + 卖家（正常交易）
    - 买家 + 仲裁员（买家退款）
    - 卖家 + 仲裁员（卖家收款）

--- 场景1: 买家存入托管资金 ---
买家余额: 100000 sat

商品价格: 50000 sat
买家将货款转入托管地址...

✓ 资金已托管
  托管金额: 50000 sat
  买家余额: 49900 sat

--- 场景2A: 正常交易（买家满意）---
卖家已发货
买家收到货物，确认满意

签名过程:
  ✓ 买家已签名（确认收货）
  ✓ 卖家已签名（同意收款）

✓ 签名满足要求 (2/3)
✓ 释放资金给卖家

=== 交易完成 ===
卖家余额: 49950 sat
托管余额: 0 sat

--- 场景2B: 争议处理（货物有问题）---
买家: 货物与描述不符，要求退款
卖家: 货物没问题，拒绝退款
仲裁员介入调查...

仲裁结果:
  经核实，货物确实存在问题
  判决：退款给买家

签名过程:
  ✓ 买家签名（同意退款）
  ✓ 仲裁员签名（执行判决）

✓ 签名满足要求 (2/3)
✓ 执行退款

=== 争议解决 ===
退款给买家: 49950 sat
仲裁费: 50 sat（从托管金扣除）
```

---

## 业务价值

### 1. 买家保护

| 传统交易 | 托管服务 |
|---------|---------|
| ❌ 付款后卖家不发货 | ✓ 资金托管，发货后才释放 |
| ❌ 货不对版无法退款 | ✓ 仲裁员判定可退款 |
| ❌ 纠纷无处申诉 | ✓ 仲裁机制保护权益 |

### 2. 卖家保护

| 传统交易 | 托管服务 |
|---------|---------|
| ❌ 发货后买家拒付 | ✓ 货款已托管，正常发货即可 |
| ❌ 恶意退款 | ✓ 仲裁员公正判断 |
| ❌ 无担保风险 | ✓ 资金确定性 |

### 3. 公平性

```
买家单独无法取走资金（需要卖家或仲裁员）
卖家单独无法取走资金（需要买家或仲裁员）
仲裁员单独无法取走资金（需要买卖双方之一）

→ 三方制衡，公平公正
```

---

## 扩展方案

### 1. 自动仲裁

```rust
struct AutoArbitration {
    物流跟踪: bool,
    照片证据: Vec<String>,
    聊天记录: Vec<Message>,
}

fn auto_judge(evidence: &AutoArbitration) -> Decision {
    if evidence.物流跟踪 && evidence.照片证据.len() > 3 {
        Decision::RefundBuyer  // 自动退款
    } else {
        Decision::ManualReview  // 人工审核
    }
}
```

### 2. 分阶段释放

```rust
// 阶段1: 发货确认 - 释放50%
// 阶段2: 收货确认 - 释放剩余50%

let stage1 = escrow_amount / 2;
let stage2 = escrow_amount - stage1;

// 卖家提供物流单号 → 释放stage1
// 买家确认收货 → 释放stage2
```

### 3. 时间锁保护

```rust
use bitcoin_simulation::advanced_tx::TimeLock;

// 7天内无争议，自动释放给卖家
let seven_days = 7 * 24 * 3600;
let auto_release = TimeLock::new_time_based(current_time + seven_days);

if auto_release.is_mature(...) && no_dispute {
    release_to_seller();
}
```

### 4. 多层仲裁

```rust
// 一级仲裁: 普通仲裁员
// 二级仲裁: 资深仲裁员
// 三级仲裁: 仲裁委员会 (3-of-5)

let appeals_committee = MultiSigAddress::new(
    3,
    vec![arbitrator1, arbitrator2, arbitrator3, arbitrator4, arbitrator5]
)?;
```

---

## 仲裁员机制

### 选择标准

```
✓ 信誉良好（历史记录）
✓ 专业知识（商品类别）
✓ 中立公正（无利益冲突）
✓ 响应及时（24小时内）
```

### 仲裁费用

```rust
let arbitration_fee = match dispute_complexity {
    Simple => 50,      // 0.1%
    Medium => 100,     // 0.2%
    Complex => 500,    // 1%
};

// 从托管金扣除
let net_amount = escrow_amount - arbitration_fee;
```

### 仲裁流程

```
1. 买家/卖家发起争议
2. 提交证据（照片、聊天记录）
3. 仲裁员审核（3个工作日内）
4. 做出判决
5. 执行判决（签名）
6. 收取仲裁费
```

---

## 安全考虑

### 1. 仲裁员串通

**风险**: 仲裁员与买家/卖家串通

**防护**:
```rust
// 仲裁员需要质押
let arbitrator_deposit = 100000;

// 串通被发现，扣除质押
if collusion_detected {
    slash_deposit(&arbitrator);
    ban_arbitrator(&arbitrator);
}

// 多个仲裁员投票
let arbitrators = vec![arb1, arb2, arb3];
let decision = majority_vote(&arbitrators);
```

### 2. 证据造假

**防护**:
```rust
// 物流信息上链
blockchain.add_tracking_info(tracking_number);

// 照片哈希上链（防篡改）
let photo_hash = hash_photo(photo);
blockchain.add_evidence_hash(photo_hash);

// 时间戳证明
let timestamp = blockchain.get_block_time();
```

### 3. 恶意拖延

**防护**:
```rust
// 设置仲裁时限
let deadline = current_time + 7 * 86400;  // 7天

if current_time > deadline && no_decision {
    // 超时自动退款
    refund_to_buyer();
}
```

---

## 实施建议

### 1. 技术栈

```
前端: Web界面展示托管流程
后端: SimpleBTC + 数据库
存储: 证据存储（IPFS）
通知: 邮件/短信提醒
```

### 2. 用户流程

```
买家:
  1. 浏览商品
  2. 下单并将货款转入托管
  3. 等待卖家发货
  4. 收货并确认
  5. 签名释放资金

卖家:
  1. 等待买家托管资金
  2. 看到托管成功后发货
  3. 提供物流单号
  4. 等待买家确认
  5. 签名收款
```

### 3. 费用结构

```
平台手续费: 1%
仲裁费: 0.1-1%（争议时）
区块链手续费: 动态（50-200 sat）
```

---

## 对比传统方案

### vs 支付宝担保交易

| 特性 | SimpleBTC托管 | 支付宝担保 |
|------|--------------|-----------|
| 去中心化 | ✓ | ❌ 中心化 |
| 审查抗性 | ✓ | ❌ 可被审查 |
| 跨境支付 | ✓ | ❌ 受限 |
| 手续费 | 低（0.1-1%） | 较高（1-3%） |
| 隐私性 | 较好 | 较差 |

### vs PayPal争议

| 特性 | SimpleBTC托管 | PayPal |
|------|--------------|--------|
| 争议解决 | 仲裁员 | 平台客服 |
| 透明度 | 链上可查 | 黑箱操作 |
| 不可逆 | ✓ | ❌ 可冻结账户 |

---

## 相关资源

- [多重签名详解](../advanced/multisig.md)
- [MultiSig API](../api/multisig.md)
- [企业钱包案例](./enterprise-multisig.md)

## 总结

托管服务通过2-of-3多签实现了：

✅ **买家保护** - 货不对版可退款
✅ **卖家保护** - 货款确定到账
✅ **公平仲裁** - 第三方公正判决
✅ **去中心化** - 无需信任中心平台
✅ **透明可查** - 链上记录公开

是电商、自由职业、跨境贸易的理想解决方案！

---

[查看完整源代码](../../examples/escrow_service.rs)
