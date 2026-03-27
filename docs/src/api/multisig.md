# MultiSig API

多重签名（Multi-Signature，简称 MultiSig）实现在 `src/multisig.rs` 中，支持 Bitcoin 的 M-of-N 签名方案：从 N 个参与方中需要至少 M 个人提供有效的 ECDSA 签名，资金才能被动用。

---

## 核心概念

**M-of-N 签名：** N 个参与方各持一个密钥对，任意 M 个人签名即可授权交易。常见组合：

| 类型 | 含义 | 典型场景 |
|------|------|----------|
| 2-of-2 | 两人必须全部同意 | 联合账户、合伙企业 |
| 2-of-3 | 三人中任意两人同意 | 企业资金管理、托管服务 |
| 3-of-5 | 五人中任意三人同意 | 大型机构资金、董事会决策 |

**地址格式：** 多签地址以 `"3"` 开头，对应比特币的 P2SH（Pay-to-Script-Hash）格式。

---

## MultiSigAddress 结构体

表示一个 M-of-N 多重签名地址及其配置。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigAddress {
    pub address: String,          // 多签地址（"3" 开头，P2SH 格式）
    pub required_sigs: usize,     // 需要的签名数量 M
    pub total_keys: usize,        // 总密钥数量 N
    pub public_keys: Vec<String>, // 所有参与方的公钥列表
    pub script: String,           // 锁定脚本（简化的 Script 代码）
}
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `address` | `String` | 多签地址，以 `"3"` 开头，由锁定脚本的 SHA256 哈希截取 42 字符生成。 |
| `required_sigs` | `usize` | M 值，即花费资金所需的最少有效签名数。 |
| `total_keys` | `usize` | N 值，即参与方总数（等于 `public_keys.len()`）。 |
| `public_keys` | `Vec<String>` | 所有参与方钱包的公钥列表（十六进制编码）。 |
| `script` | `String` | 锁定脚本，格式为 `OP_{M}{pubkeys...}OP_CHECKMULTISIG`。 |

---

## MultiSigAddress 方法

### `MultiSigAddress::new`

创建一个 M-of-N 多重签名地址。验证参数合法性后生成锁定脚本和对应的 P2SH 地址。

```rust
pub fn new(
    required_sigs: usize,
    public_keys: Vec<String>,
) -> Result<Self, String>
```

**参数：**
- `required_sigs` — 所需签名数 M，必须满足 `1 <= M <= N`。
- `public_keys` — 所有参与方的公钥列表，长度即为 N，最多 15 个。

**返回值：**
- `Ok(MultiSigAddress)` — 创建成功。
- `Err(String)` — 参数非法，错误原因包括：
  - `"无效的签名要求"` — `required_sigs == 0` 或 `required_sigs > total_keys`。
  - `"最多支持15个密钥"` — `public_keys.len() > 15`（比特币协议限制）。

```rust
use simplebtc::multisig::MultiSigAddress;
use simplebtc::wallet::Wallet;

// 创建三个参与方钱包
let wallet1 = Wallet::new();
let wallet2 = Wallet::new();
let wallet3 = Wallet::new();

let public_keys = vec![
    wallet1.public_key.clone(),
    wallet2.public_key.clone(),
    wallet3.public_key.clone(),
];

// 创建 2-of-3 多签地址
let multisig = MultiSigAddress::new(2, public_keys).unwrap();
assert!(multisig.address.starts_with('3'));
assert_eq!(multisig.required_sigs, 2);
assert_eq!(multisig.total_keys, 3);
println!("多签地址: {}", multisig.address);
println!("锁定脚本: {}", multisig.script);

// 参数校验错误示例
let result = MultiSigAddress::new(0, vec!["key1".to_string()]);
assert!(result.is_err()); // required_sigs 不能为 0

let result = MultiSigAddress::new(3, vec!["key1".to_string(), "key2".to_string()]);
assert!(result.is_err()); // required_sigs(3) > total_keys(2)
```

---

### `MultiSigAddress::verify_signatures`

快速检查签名数量是否满足要求（不验证签名内容，仅校验数量）。

```rust
pub fn verify_signatures(&self, signatures: &[String]) -> bool
```

**参数：**
- `signatures` — 签名列表。

**返回值：** `signatures.len() >= self.required_sigs`。

```rust
let sigs = vec!["sig1".to_string(), "sig2".to_string()];
let count_ok = multisig.verify_signatures(&sigs);
println!("签名数量满足要求: {}", count_ok); // true（2 >= 2）
```

---

### `MultiSigAddress::verify_signatures_with_data`

完整的签名验证：不仅检查数量，还用 ECDSA 验证每个签名是否由 `public_keys` 中的某个密钥产生。

```rust
pub fn verify_signatures_with_data(
    &self,
    signatures: &[String],
    data: &str,
) -> bool
```

**参数：**
- `signatures` — 十六进制 DER 编码的 ECDSA 签名列表。
- `data` — 被签名的原始数据字符串（通常是交易 ID 或交易摘要）。

**返回值：**
- `true` — 有效签名数量 >= `required_sigs`。每个签名最多匹配一个公钥（防止同一签名重复计数）。
- `false` — 有效签名数量不足，或签名不对应 `public_keys` 中的任何公钥。

```rust
// 用 wallet1 和 wallet2 对交易数据签名
let data = "交易摘要：Alice 转账 0.01 BTC 给 Bob";
let sig1 = wallet1.sign(data);
let sig2 = wallet2.sign(data);

let valid = multisig.verify_signatures_with_data(
    &[sig1, sig2],
    data,
);
println!("ECDSA 验证通过: {}", valid);
```

---

## MultiSigTxBuilder 结构体

多重签名交易构建器，负责收集签名并确认是否满足 M-of-N 要求。内置防重复签名检测。

```rust
pub struct MultiSigTxBuilder {
    pub multisig_address: MultiSigAddress,
    pub signatures: Vec<String>,
    // signed_keys: HashMap<String, bool>  // 私有字段，防止同一公钥重复签名
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `multisig_address` | `MultiSigAddress` | 关联的多签地址配置（含 M、N、公钥列表）。 |
| `signatures` | `Vec<String>` | 已收集的有效 ECDSA 签名列表。 |

---

## MultiSigTxBuilder 方法

### `MultiSigTxBuilder::new`

创建多签交易构建器，与指定的 `MultiSigAddress` 关联。

```rust
pub fn new(multisig_address: MultiSigAddress) -> Self
```

**参数：**
- `multisig_address` — 已创建的 `MultiSigAddress` 实例。

```rust
use simplebtc::multisig::MultiSigTxBuilder;

let builder = MultiSigTxBuilder::new(multisig);
assert_eq!(builder.signatures.len(), 0);
assert!(!builder.is_complete());
```

---

### `MultiSigTxBuilder::add_signature`

添加一个参与方的 ECDSA 签名。内部自动验证：
1. 该钱包的公钥必须在 `multisig_address.public_keys` 中。
2. 该钱包不能重复签名（防止同一人签名两次来伪造 M 个签名）。

```rust
pub fn add_signature(
    &mut self,
    wallet: &Wallet,
    data: &str,
) -> Result<(), String>
```

**参数：**
- `wallet` — 参与方的 `Wallet` 实例（用于调用 `wallet.sign(data)` 生成签名）。
- `data` — 要签名的数据（通常为交易摘要或交易 ID）。

**返回值：**
- `Ok(())` — 签名添加成功。
- `Err(String)` — 错误原因：
  - `"此钱包不在多签地址中"` — 该钱包的公钥不在 `public_keys` 列表中。
  - `"此钱包已签名"` — 该钱包之前已经签过名。

```rust
let mut builder = MultiSigTxBuilder::new(multisig.clone());
let data = "transfer_tx_hash_abc123";

// wallet1 签名成功
builder.add_signature(&wallet1, data).unwrap();
assert_eq!(builder.signatures.len(), 1);

// wallet1 不能重复签名
let err = builder.add_signature(&wallet1, data);
assert!(err.is_err());
println!("重复签名错误: {}", err.unwrap_err()); // "此钱包已签名"

// 不在多签地址中的钱包无法签名
let outsider = Wallet::new();
let err = builder.add_signature(&outsider, data);
assert!(err.is_err());
println!("外部钱包错误: {}", err.unwrap_err()); // "此钱包不在多签地址中"
```

---

### `MultiSigTxBuilder::is_complete`

检查是否已收集到足够的签名（`signatures.len() >= required_sigs`）。

```rust
pub fn is_complete(&self) -> bool
```

**返回值：** `true` 表示已满足 M-of-N 要求，可以广播交易。

```rust
let mut builder = MultiSigTxBuilder::new(multisig);
assert!(!builder.is_complete()); // 0 个签名

builder.add_signature(&wallet1, data).unwrap();
assert!(!builder.is_complete()); // 1 个签名，还不够（需要 2）

builder.add_signature(&wallet2, data).unwrap();
assert!(builder.is_complete());  // 2 个签名，满足 2-of-3
println!("多签已完成，可以广播交易");
```

---

### `MultiSigTxBuilder::get_signatures`

获取所有已收集签名的副本列表。

```rust
pub fn get_signatures(&self) -> Vec<String>
```

**返回值：** `Vec<String>` — 签名列表的克隆（不影响构建器内部状态）。

```rust
let sigs = builder.get_signatures();
println!("已收集 {} 个签名", sigs.len());
for (i, sig) in sigs.iter().enumerate() {
    println!("  签名 {}: {}...", i + 1, &sig[..16]);
}
```

---

## MultiSigType 枚举（便捷 API）

提供常见多签类型的快速创建方式。

```rust
pub enum MultiSigType {
    TwoOfTwo,    // 2-of-2
    TwoOfThree,  // 2-of-3（最常用）
    ThreeOfFive, // 3-of-5
}

impl MultiSigType {
    pub fn create_address(&self, wallets: &[Wallet]) -> Result<MultiSigAddress, String>
}
```

```rust
use simplebtc::multisig::MultiSigType;
use simplebtc::wallet::Wallet;

let wallets: Vec<Wallet> = (0..3).map(|_| Wallet::new()).collect();
let ms_addr = MultiSigType::TwoOfThree.create_address(&wallets).unwrap();
println!("2-of-3 地址: {}", ms_addr.address);

// 如果钱包数量不匹配会返回错误
let err = MultiSigType::ThreeOfFive.create_address(&wallets); // 需要 5 个钱包
assert!(err.is_err());
```

---

## 完整使用示例

### 场景一：企业资金管理（2-of-3）

```rust
use simplebtc::multisig::{MultiSigAddress, MultiSigTxBuilder};
use simplebtc::wallet::Wallet;

fn corporate_treasury() {
    // 公司三个高管各持一个密钥
    let ceo = Wallet::new();
    let cfo = Wallet::new();
    let cto = Wallet::new();

    // 创建 2-of-3 企业多签地址
    let pub_keys = vec![
        ceo.public_key.clone(),
        cfo.public_key.clone(),
        cto.public_key.clone(),
    ];
    let treasury = MultiSigAddress::new(2, pub_keys).unwrap();
    println!("企业金库地址: {}", treasury.address);

    // 发起一笔支付（需要 CEO + CFO 共同签名）
    let tx_data = "支付 10 BTC 给供应商 ABC";
    let mut builder = MultiSigTxBuilder::new(treasury);

    builder.add_signature(&ceo, tx_data).unwrap();
    println!("CEO 已签名，等待第二个授权...");

    builder.add_signature(&cfo, tx_data).unwrap();
    println!("CFO 已签名");

    if builder.is_complete() {
        let signatures = builder.get_signatures();
        println!("交易已完成授权，签名数: {}", signatures.len());
        // 此处将 signatures 附加到交易并广播到网络
    }
}
```

### 场景二：第三方托管（买家-卖家-仲裁员）

```rust
fn escrow_service() {
    let buyer  = Wallet::new();
    let seller = Wallet::new();
    let arbiter = Wallet::new();

    let pub_keys = vec![
        buyer.public_key.clone(),
        seller.public_key.clone(),
        arbiter.public_key.clone(),
    ];

    // 2-of-3：正常情况买家+卖家，争议时任一方+仲裁员
    let escrow = MultiSigAddress::new(2, pub_keys).unwrap();
    println!("托管地址: {}", escrow.address);

    let release_tx = "释放托管资金至卖家地址";

    // 正常流程：买家确认收货，买家+卖家共同签名释放资金
    let mut builder = MultiSigTxBuilder::new(escrow.clone());
    builder.add_signature(&buyer, release_tx).unwrap();
    builder.add_signature(&seller, release_tx).unwrap();
    assert!(builder.is_complete());
    println!("资金正常释放给卖家");

    // 争议流程：仲裁员介入，卖家+仲裁员签名释放资金
    let dispute_tx = "争议裁定：退款给买家";
    let mut dispute_builder = MultiSigTxBuilder::new(escrow);
    dispute_builder.add_signature(&buyer, dispute_tx).unwrap();
    dispute_builder.add_signature(&arbiter, dispute_tx).unwrap();
    assert!(dispute_builder.is_complete());
    println!("仲裁完成，资金退还买家");
}
```

### 场景三：完整签名验证流程

```rust
fn full_verification() {
    let w1 = Wallet::new();
    let w2 = Wallet::new();

    let pub_keys = vec![w1.public_key.clone(), w2.public_key.clone()];
    let ms = MultiSigAddress::new(2, pub_keys).unwrap(); // 2-of-2

    let tx_data = "转账 1 BTC";

    // 收集签名
    let sig1 = w1.sign(tx_data);
    let sig2 = w2.sign(tx_data);
    let sigs = vec![sig1, sig2];

    // 完整 ECDSA 验证（验证签名是否由 public_keys 中的密钥产生）
    let valid = ms.verify_signatures_with_data(&sigs, tx_data);
    println!("ECDSA 多签验证: {}", valid);

    // 快速数量检查（不验证内容）
    let count_ok = ms.verify_signatures(&sigs);
    println!("签名数量满足: {}", count_ok);
}
```

---

## 限制与注意事项

- **最大密钥数：** 由于比特币脚本限制，N 最大为 **15**（超出返回错误）。
- **防重复签名：** `MultiSigTxBuilder` 内部维护已签名公钥集合，同一钱包调用 `add_signature` 两次会返回错误。
- **签名顺序：** 验证时不要求签名顺序与公钥顺序一致，任意 M 个有效签名即可。
- **零确认风险：** 多签交易在被矿工打包前仍属于未确认状态，重要交易应等待至少 1 个区块确认。

---

## 相关模块

- [`Wallet`](wallet.md) — 提供 `sign()` 和 `verify_signature()` 方法，是多签 ECDSA 操作的基础。
- [`AdvancedTxBuilder`](advanced-tx.md) — 可与时间锁结合，实现"时间到期后多签要求降低"等高级场景。
- [高级模块概览](advanced.md) — 了解多签在 SimpleBTC 整体架构中的位置。
