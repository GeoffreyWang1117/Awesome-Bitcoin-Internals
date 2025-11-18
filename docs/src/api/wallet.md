# Wallet API

钱包模块负责管理密钥对、地址和签名。

## 数据结构

### `Wallet`

```rust
pub struct Wallet {
    pub address: String,        // 钱包地址（公钥哈希）
    pub private_key: String,    // 私钥
    pub public_key: String,     // 公钥
}
```

## 方法

### 创建钱包

#### `new`

```rust
pub fn new() -> Self
```

创建新钱包，自动生成密钥对和地址。

**密钥生成流程**：
1. 生成随机私钥（64字符十六进制）
2. 从私钥派生公钥（SHA256）
3. 从公钥哈希得到地址（取前40字符）

**返回值**: 新的钱包实例

**安全提示**：
- ⚠️ 私钥必须保密
- ⚠️ 私钥丢失无法恢复
- ⚠️ 建议备份到安全位置

**示例**:
```rust
use bitcoin_simulation::wallet::Wallet;

// 创建新钱包
let wallet = Wallet::new();

println!("地址: {}", wallet.address);
println!("公钥: {}", wallet.public_key);
// 私钥不要打印或分享！

// 多个钱包
let alice = Wallet::new();
let bob = Wallet::new();
let charlie = Wallet::new();
```

---

#### `from_address`

```rust
pub fn from_address(address: String) -> Self
```

从已知地址创建钱包（仅用于演示）。

**注意**:
- 这会生成新的随机密钥对
- 密钥与地址不对应
- 仅用于测试和演示

**参数**:
- `address` - 指定的地址字符串

**返回值**: 钱包实例（密钥是新生成的）

**示例**:
```rust
// 用于演示创世地址
let genesis = Wallet::from_address("genesis_address".to_string());

// 实际应用中应该从私钥恢复
// let wallet = Wallet::from_private_key(private_key);
```

---

### 签名操作

#### `sign`

```rust
pub fn sign(&self, data: &str) -> String
```

使用私钥签名数据。

**签名过程**（简化版）：
```
signature = SHA256(private_key + data)
```

**实际比特币**使用ECDSA：
```
1. 对数据进行双重SHA256
2. 使用私钥和secp256k1曲线生成签名
3. 签名包含r和s两部分
```

**参数**:
- `data` - 要签名的数据（通常是交易数据）

**返回值**: 签名字符串（64字符十六进制）

**用途**:
- 证明拥有私钥
- 授权交易
- 防止交易被篡改

**示例**:
```rust
let wallet = Wallet::new();

// 签名交易数据
let tx_data = "send 100 BTC to Bob";
let signature = wallet.sign(tx_data);

println!("签名: {}", signature);

// 在交易中使用
let input = TxInput::new(
    prev_txid,
    vout,
    wallet.sign(&tx_data),  // 签名
    wallet.public_key.clone(),
);
```

---

#### `verify_signature` (静态方法)

```rust
pub fn verify_signature(
    public_key: &str,
    data: &str,
    signature: &str
) -> bool
```

验证签名是否有效（简化版）。

**验证过程**（简化版）：
- 检查公钥和签名非空

**实际比特币**使用ECDSA验证：
1. 从签名恢复公钥
2. 验证公钥匹配
3. 验证签名数学正确性

**参数**:
- `public_key` - 签名者的公钥
- `data` - 原始数据
- `signature` - 签名

**返回值**:
- `true` - 签名有效
- `false` - 签名无效

**示例**:
```rust
let wallet = Wallet::new();
let data = "transaction data";
let signature = wallet.sign(data);

// 验证签名
if Wallet::verify_signature(&wallet.public_key, data, &signature) {
    println!("✓ 签名有效");
} else {
    println!("✗ 签名无效");
}

// 在交易验证中使用
for input in transaction.inputs {
    if !Wallet::verify_signature(&input.pub_key, &tx_data, &input.signature) {
        return Err("签名验证失败");
    }
}
```

---

## 地址格式

### SimpleBTC地址

```
格式: 40字符十六进制字符串
示例: a3f2d8c9e4b7f1a89c2d5e8f3b6a1c4e7d9b2a5c
```

### 真实比特币地址

#### P2PKH（以1开头）
```
1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
```
**生成过程**:
```
公钥 → SHA256 → RIPEMD160 → 添加版本 → 校验和 → Base58编码
```

#### P2SH（以3开头）
```
3J98t1WpEZ73CNmYviecrnyiWrnqRhWNLy
```
**用途**: 多签、脚本地址

#### Bech32（以bc1开头）
```
bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
```
**优势**: SegWit地址，手续费更低

---

## 密钥管理

### 私钥安全

**最佳实践**:

```rust
// ✅ 好的做法
let wallet = Wallet::new();

// 加密存储私钥
let encrypted = encrypt_private_key(&wallet.private_key, password);
save_to_secure_storage(&encrypted);

// 使用后立即清除内存
drop(wallet);

// 备份到多个位置
backup_to_hardware_wallet(&wallet.private_key);
backup_to_paper(&wallet.private_key);
backup_to_encrypted_usb(&wallet.private_key);
```

```rust
// ❌ 不好的做法
println!("私钥: {}", wallet.private_key);  // 永不打印
save_to_file(&wallet.private_key);        // 明文存储
send_via_email(&wallet.private_key);      // 网络传输
```

### 密钥恢复

```rust
// 从私钥恢复钱包（需要实现）
fn recover_wallet(private_key: &str) -> Wallet {
    // 1. 验证私钥格式
    // 2. 从私钥派生公钥
    // 3. 从公钥生成地址
    // 4. 返回钱包实例
}

// 使用助记词（BIP39标准，需要实现）
fn from_mnemonic(words: &str) -> Wallet {
    // 助记词 → 种子 → 主私钥 → 派生密钥
}
```

---

## 使用场景

### 场景1: 基本转账

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn basic_transfer() -> Result<(), String> {
    let mut blockchain = Blockchain::new();

    // 创建参与者
    let alice = Wallet::new();
    let bob = Wallet::new();

    // Alice获得初始资金
    setup_balance(&mut blockchain, &alice, 10000)?;

    // Alice向Bob转账
    let tx = blockchain.create_transaction(
        &alice,              // from_wallet
        bob.address.clone(),
        5000,               // amount
        10,                 // fee
    )?;

    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(alice.address.clone())?;

    // 查看余额
    println!("Alice: {}", blockchain.get_balance(&alice.address));
    println!("Bob: {}", blockchain.get_balance(&bob.address));

    Ok(())
}
```

### 场景2: 批量创建钱包

```rust
fn create_wallet_pool(count: usize) -> Vec<Wallet> {
    let mut wallets = Vec::new();

    for i in 0..count {
        let wallet = Wallet::new();
        println!("钱包 #{}: {}", i, &wallet.address[..16]);
        wallets.push(wallet);
    }

    wallets
}

// 使用
let users = create_wallet_pool(100);  // 创建100个钱包
```

### 场景3: 钱包导入导出

```rust
use serde_json;

// 导出钱包（加密）
fn export_wallet(wallet: &Wallet, password: &str) -> Result<String, String> {
    let wallet_json = serde_json::to_string(wallet)?;
    let encrypted = encrypt(&wallet_json, password);
    Ok(encrypted)
}

// 导入钱包
fn import_wallet(encrypted_data: &str, password: &str) -> Result<Wallet, String> {
    let decrypted = decrypt(encrypted_data, password)?;
    let wallet: Wallet = serde_json::from_str(&decrypted)?;
    Ok(wallet)
}

// 使用
let wallet = Wallet::new();
let backup = export_wallet(&wallet, "strong_password")?;
save_to_file("wallet_backup.enc", &backup)?;

// 恢复
let backup_data = read_from_file("wallet_backup.enc")?;
let recovered = import_wallet(&backup_data, "strong_password")?;
```

### 场景4: 多签钱包集成

```rust
use bitcoin_simulation::multisig::MultiSigAddress;

fn create_multisig_wallet() -> Result<MultiSigAddress, String> {
    // 创建参与者钱包
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    // 收集公钥
    let public_keys = vec![
        alice.public_key,
        bob.public_key,
        charlie.public_key,
    ];

    // 创建2-of-3多签地址
    let multisig = MultiSigAddress::new(2, public_keys)?;

    println!("多签地址: {}", multisig.address);

    Ok(multisig)
}
```

---

## 与真实比特币的差异

| 特性 | SimpleBTC | 真实比特币 |
|------|-----------|------------|
| 私钥生成 | 随机字符串 | 256位随机数 |
| 公钥推导 | SHA256 | secp256k1椭圆曲线 |
| 地址格式 | 40字符十六进制 | Base58/Bech32编码 |
| 签名算法 | SHA256 | ECDSA |
| 签名验证 | 简化检查 | 完整数学验证 |

**真实比特币流程**:
```
私钥(256 bits)
  ↓ secp256k1
公钥(33/65 bytes)
  ↓ SHA256 + RIPEMD160
公钥哈希(20 bytes)
  ↓ 版本 + 校验和 + Base58
地址(25-34 chars)
```

---

## 安全建议

### 1. 私钥保护

```rust
// 使用操作系统密钥环
use keyring::Entry;

fn store_private_key(address: &str, private_key: &str) -> Result<(), String> {
    let entry = Entry::new("SimpleBTC", address)?;
    entry.set_password(private_key)?;
    Ok(())
}

fn retrieve_private_key(address: &str) -> Result<String, String> {
    let entry = Entry::new("SimpleBTC", address)?;
    let private_key = entry.get_password()?;
    Ok(private_key)
}
```

### 2. 多重备份

- ✅ 纸钱包（防火防水）
- ✅ 硬件钱包（Ledger, Trezor）
- ✅ 加密U盘（异地存储）
- ✅ 分片存储（Shamir秘密共享）

### 3. 定期审计

```rust
fn audit_wallets(wallets: &[Wallet]) {
    for (i, wallet) in wallets.iter().enumerate() {
        println!("钱包 #{}", i);
        println!("  地址: {}", wallet.address);
        println!("  公钥存在: {}", !wallet.public_key.is_empty());
        println!("  私钥存在: {}", !wallet.private_key.is_empty());

        // 测试签名
        let test_sig = wallet.sign("test");
        assert!(Wallet::verify_signature(
            &wallet.public_key,
            "test",
            &test_sig
        ));
    }
}
```

---

## 常见问题

### Q: 如何恢复丢失的钱包？

**A:** 只能从备份的私钥恢复。如果私钥丢失，比特币永久丢失。

### Q: 可以从地址推导私钥吗？

**A:** 不可以。地址是单向哈希，计算上不可逆。

### Q: 一个私钥可以生成多个地址吗？

**A:** 分层确定性钱包（HD Wallet, BIP32）可以从一个种子派生多个密钥对。

### Q: 如何知道钱包是否被盗用？

**A:** 监控区块链上的交易记录，如果出现未授权的交易，说明私钥泄露。

---

## 参考

- [Transaction API](./transaction.md) - 使用钱包创建交易
- [MultiSig API](./multisig.md) - 多签钱包
- [快速入门](../guide/quickstart.md) - 钱包使用教程
- [BIP32 - HD Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP39 - 助记词](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)

---

[返回API目录](./core.md)
