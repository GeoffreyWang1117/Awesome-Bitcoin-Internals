# 多重签名（MultiSig）

多重签名是比特币的高级功能，要求M个签名才能花费N个公钥控制的资金（M-of-N）。

## 概述

### 什么是多重签名？

多重签名地址需要多个私钥共同签名才能花费资金，而不是传统的单一私钥。

**示例**：
- **2-of-3**: 需要3个密钥中的任意2个
- **3-of-5**: 需要5个密钥中的任意3个
- **2-of-2**: 需要2个密钥都同意

### 为什么需要多签？

#### 1. 安全性提升
- 没有单点故障
- 私钥被盗不会立即丢失资金
- 分散风险

#### 2. 信任分散
- 企业治理：防止单人滥用
- 托管服务：买卖双方 + 仲裁员
- 家庭共管：夫妻共同管理

#### 3. 灵活性
- 不同的M-N组合满足不同需求
- 可以设置紧急恢复机制
- 支持复杂的业务逻辑

## 技术实现

### 多签地址结构

```rust
pub struct MultiSigAddress {
    pub address: String,            // 多签地址（以"3"开头）
    pub required_sigs: usize,       // M（需要的签名数）
    pub total_keys: usize,          // N（总密钥数）
    pub public_keys: Vec<String>,   // 所有参与者公钥
    pub script: String,             // 锁定脚本
}
```

### 创建多签地址

```rust
use bitcoin_simulation::{multisig::MultiSigAddress, wallet::Wallet};

// 创建参与者
let ceo = Wallet::new();
let cfo = Wallet::new();
let cto = Wallet::new();

// 收集公钥
let public_keys = vec![
    ceo.public_key.clone(),
    cfo.public_key.clone(),
    cto.public_key.clone(),
];

// 创建2-of-3多签地址
let multisig = MultiSigAddress::new(2, public_keys)?;

println!("多签地址: {}", multisig.address);
println!("需要签名: {}/{}", multisig.required_sigs, multisig.total_keys);
```

### 多签类型

SimpleBTC提供了预设的常用多签类型：

```rust
use bitcoin_simulation::multisig::MultiSigType;

// 2-of-2: 双方都必须同意
let two_of_two = MultiSigAddress::from_type(
    MultiSigType::TwoOfTwo,
    vec![alice.public_key, bob.public_key]
)?;

// 2-of-3: 任意两方即可（最常用）
let two_of_three = MultiSigAddress::from_type(
    MultiSigType::TwoOfThree,
    vec![party1.public_key, party2.public_key, party3.public_key]
)?;

// 3-of-5: 高安全性场景
let three_of_five = MultiSigAddress::from_type(
    MultiSigType::ThreeOfFive,
    vec![pk1, pk2, pk3, pk4, pk5]
)?;
```

## 应用场景

### 场景1: 企业财务管理

**需求**: 公司资金需要多个高管共同批准

**方案**: 2-of-3多签（CEO + CFO + CTO）

```rust
fn setup_corporate_wallet() -> Result<MultiSigAddress, String> {
    // 1. 创建高管钱包
    let ceo = Wallet::new();
    let cfo = Wallet::new();
    let cto = Wallet::new();

    println!("=== 企业多签钱包 ===");
    println!("CEO: {}", &ceo.address[..16]);
    println!("CFO: {}", &cfo.address[..16]);
    println!("CTO: {}", &cto.address[..16]);

    // 2. 创建多签地址
    let company_wallet = MultiSigAddress::new(
        2,  // 需要2个签名
        vec![
            ceo.public_key.clone(),
            cfo.public_key.clone(),
            cto.public_key.clone(),
        ]
    )?;

    println!("\n公司多签地址: {}", company_wallet.address);
    println!("规则: 任意2位高管签名即可转账\n");

    Ok(company_wallet)
}

// 转账场景
fn corporate_payment(
    multisig: &MultiSigAddress,
    ceo: &Wallet,
    cfo: &Wallet,
    recipient: &str,
    amount: u64
) -> Result<(), String> {
    println!("转账 {} satoshi 给 {}", amount, &recipient[..16]);

    // 1. CEO签名
    let ceo_sig = ceo.sign(&format!("{}{}", multisig.address, amount));
    println!("✓ CEO已签名");

    // 2. CFO签名
    let cfo_sig = cfo.sign(&format!("{}{}", multisig.address, amount));
    println!("✓ CFO已签名");

    // 3. 收集签名
    let signatures = vec![ceo_sig, cfo_sig];

    // 4. 验证签名数量
    if signatures.len() >= multisig.required_sigs {
        println!("✅ 签名数量满足要求，交易可以执行");
        // 创建并广播交易...
        Ok(())
    } else {
        Err("签名不足".to_string())
    }
}
```

**优势**:
- ✅ 防止单人滥用资金
- ✅ CEO出差时，CFO+CTO仍可运作
- ✅ 任何一人被攻击，资金仍安全

### 场景2: 托管交易

**需求**: 买卖双方不信任对方，需要第三方仲裁

**方案**: 2-of-3多签（买家 + 卖家 + 仲裁员）

```rust
fn escrow_service() -> Result<(), String> {
    // 参与方
    let buyer = Wallet::new();
    let seller = Wallet::new();
    let arbitrator = Wallet::new();

    println!("=== 托管服务 ===");
    println!("买家: {}", &buyer.address[..16]);
    println!("卖家: {}", &seller.address[..16]);
    println!("仲裁员: {}", &arbitrator.address[..16]);

    // 创建托管多签地址
    let escrow = MultiSigAddress::new(
        2,
        vec![
            buyer.public_key.clone(),
            seller.public_key.clone(),
            arbitrator.public_key.clone(),
        ]
    )?;

    println!("\n托管地址: {}", escrow.address);

    // 情况1: 正常交易（买家 + 卖家）
    println!("\n--- 场景1: 交易顺利完成 ---");
    println!("买家收到货物，满意");
    println!("买家签名: ✓");
    println!("卖家签名: ✓");
    println!("✅ 2/3签名，资金释放给卖家");

    // 情况2: 争议（买家 + 仲裁员 或 卖家 + 仲裁员）
    println!("\n--- 场景2: 发生争议 ---");
    println!("买家: 货物有问题");
    println!("卖家: 货物没问题");
    println!("仲裁员介入调查...");
    println!("仲裁员: 买家有理");
    println!("买家签名: ✓");
    println!("仲裁员签名: ✓");
    println!("✅ 2/3签名，资金退还给买家");

    Ok(())
}
```

**优势**:
- ✅ 买家保护：货不对版可退款
- ✅ 卖家保护：正常交易自动放款
- ✅ 公平：仲裁员无法单独控制资金

### 场景3: 个人资产保护

**需求**: 防止单一私钥丢失或被盗

**方案**: 2-of-3多签（主密钥 + 备份密钥 + 托管密钥）

```rust
fn personal_security_setup() -> Result<(), String> {
    // 密钥分配
    let main_key = Wallet::new();      // 日常使用
    let backup_key = Wallet::new();    // 保险柜
    let custodian_key = Wallet::new(); // 律师/信托公司

    println!("=== 个人资产保护 ===");
    println!("主密钥（日常）: {}", &main_key.address[..16]);
    println!("备份密钥（保险柜）: {}", &backup_key.address[..16]);
    println!("托管密钥（律师）: {}", &custodian_key.address[..16]);

    let secure_wallet = MultiSigAddress::new(
        2,
        vec![
            main_key.public_key,
            backup_key.public_key,
            custodian_key.public_key,
        ]
    )?;

    println!("\n安全钱包: {}", secure_wallet.address);

    // 使用场景
    println!("\n--- 使用场景 ---");
    println!("日常转账: 主密钥 + 备份密钥");
    println!("主密钥丢失: 备份密钥 + 托管密钥");
    println!("被盗风险: 需要2个密钥，单个被盗无风险");

    Ok(())
}
```

### 场景4: 冷热钱包组合

**需求**: 大额存储安全 + 小额使用便利

**方案**: 2-of-3（热钱包 + 冷钱包1 + 冷钱包2）

```rust
fn cold_hot_wallet_setup() -> Result<(), String> {
    let hot_wallet = Wallet::new();    // 联网设备
    let cold_wallet_1 = Wallet::new(); // 硬件钱包1
    let cold_wallet_2 = Wallet::new(); // 纸钱包

    println!("=== 冷热钱包组合 ===");
    println!("热钱包（手机）: {}", &hot_wallet.address[..16]);
    println!("冷钱包1（Ledger）: {}", &cold_wallet_1.address[..16]);
    println!("冷钱包2（纸钱包）: {}", &cold_wallet_2.address[..16]);

    let vault = MultiSigAddress::new(
        2,
        vec![
            hot_wallet.public_key,
            cold_wallet_1.public_key,
            cold_wallet_2.public_key,
        ]
    )?;

    println!("\n金库地址: {}", vault.address);

    println!("\n--- 使用策略 ---");
    println!("日常小额: 热钱包 + 冷钱包1（方便）");
    println!("大额转账: 冷钱包1 + 冷钱包2（最安全）");
    println!("热钱包被黑: 仍需冷钱包配合，资金安全");

    Ok(())
}
```

## 高级用法

### 时间锁 + 多签

结合时间锁实现遗产继承：

```rust
use bitcoin_simulation::advanced_tx::TimeLock;

fn inheritance_setup() -> Result<(), String> {
    let owner = Wallet::new();
    let heir = Wallet::new();
    let lawyer = Wallet::new();

    // 正常情况：2-of-2（本人 + 继承人，保护隐私）
    let normal_multisig = MultiSigAddress::new(
        2,
        vec![owner.public_key.clone(), heir.public_key.clone()]
    )?;

    // 时间锁设置：1年后
    let one_year = 365 * 24 * 60 * 60;
    let unlock_time = current_timestamp() + one_year;
    let timelock = TimeLock::new_time_based(unlock_time);

    println!("=== 遗产继承方案 ===");
    println!("正常时期: 需要本人 + 继承人（2-of-2）");
    println!("1年后: 自动变为继承人可独立操作");

    // 或使用3-of-3，1年后降级为2-of-3
    let emergency_multisig = MultiSigAddress::new(
        2,  // 1年后只需2个
        vec![owner.public_key, heir.public_key, lawyer.public_key]
    )?;

    Ok(())
}
```

### 分层多签

大型组织的多层多签结构：

```rust
// 董事会多签: 5-of-9
let board = MultiSigAddress::new(5, board_members)?;

// 执行委员会多签: 3-of-5
let exec_committee = MultiSigAddress::new(3, executives)?;

// 小额快速多签: 2-of-3
let petty_cash = MultiSigAddress::new(2, managers)?;

println!("权限分级:");
println!("< 10 BTC: 经理级 2-of-3");
println!("10-100 BTC: 高管级 3-of-5");
println!("> 100 BTC: 董事会 5-of-9");
```

## 安全考虑

### ⚠️ 注意事项

1. **密钥管理**
   - 分散存储，不要放在一起
   - 使用硬件钱包存储冷密钥
   - 定期测试备份恢复

2. **M值选择**
   - M太小：安全性降低
   - M太大：可用性降低
   - 推荐：M = (N+1)/2 或 N-1

3. **N值选择**
   - N=2: 简单但单点故障
   - N=3: 平衡安全与便利（最常用）
   - N=5+: 高安全但复杂

4. **参与者选择**
   - 地理分散
   - 信任但相互独立
   - 有紧急联系方式

### 最佳实践

```rust
// ✅ 好的实践
let multisig = MultiSigAddress::new(
    2,  // 合理的M值
    vec![key1, key2, key3]  // 3个独立密钥
)?;

// 分散存储
// key1 -> 手机热钱包
// key2 -> 硬件钱包（保险柜）
// key3 -> 纸钱包（银行保险箱）

// ❌ 不好的实践
// 所有密钥存在同一台电脑
// M=N（失去容错能力）
// 使用同一个助记词派生多个密钥
```

## 完整示例

```rust
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::MultiSigAddress,
};

fn complete_multisig_demo() -> Result<(), String> {
    let mut blockchain = Blockchain::new();

    // 创建参与者
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    // 创建2-of-3多签
    let multisig = MultiSigAddress::new(
        2,
        vec![
            alice.public_key.clone(),
            bob.public_key.clone(),
            charlie.public_key.clone(),
        ]
    )?;

    println!("多签地址: {}", multisig.address);

    // 1. 存入资金
    let funding_tx = blockchain.create_transaction(
        &Wallet::from_address("funder".to_string()),
        multisig.address.clone(),
        10000,
        0,
    )?;
    blockchain.add_transaction(funding_tx)?;
    blockchain.mine_pending_transactions(alice.address.clone())?;

    println!("多签余额: {}", blockchain.get_balance(&multisig.address));

    // 2. 多签转账（需要2个签名）
    let recipient = Wallet::new();

    // Alice签名
    let alice_sig = alice.sign(&format!("{}{}",
        multisig.address, recipient.address));

    // Bob签名
    let bob_sig = bob.sign(&format!("{}{}",
        multisig.address, recipient.address));

    // 验证签名
    println!("\n收集签名:");
    println!("Alice: ✓");
    println!("Bob: ✓");

    if vec![alice_sig, bob_sig].len() >= multisig.required_sigs {
        println!("✅ 签名满足要求，可以转账");

        // 创建转账交易
        // 注意：实际实现需要多签交易构建逻辑
        println!("交易已创建并广播");
    }

    Ok(())
}
```

## 参考资料

- [BIP11 - M-of-N Standard Transactions](https://github.com/bitcoin/bips/blob/master/bip-0011.mediawiki)
- [BIP16 - P2SH](https://github.com/bitcoin/bips/blob/master/bip-0016.mediawiki)
- [企业多签示例](../examples/enterprise-multisig.md)
- [托管服务示例](../examples/escrow.md)

---

**下一步**: [时间锁教程](./timelock.md) | [RBF机制](./rbf.md)

[返回高级特性](./README.md)
