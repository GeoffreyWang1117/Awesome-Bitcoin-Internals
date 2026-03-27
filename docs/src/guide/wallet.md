# 钱包管理

比特币钱包的本质是一对密钥：私钥和公钥。SimpleBTC 使用与真实比特币完全兼容的 secp256k1 椭圆曲线密码学，实现了 `Wallet`（主钱包）和 `CryptoWallet`（扩展钱包，支持 Bech32 和 WIF）两个结构体。

---

## 密钥体系概览

比特币的密钥生成遵循严格的单向推导链：

```
随机数（256 bit）
       │
       ▼  secp256k1 椭圆曲线乘法
    私钥 (SecretKey, 32 字节)
       │
       ▼  G 点标量乘
    公钥 (PublicKey, 33 字节压缩格式)
       │
       ├─▶ SHA-256 哈希
       │          │
       │          ▼  RIPEMD-160 哈希
       │      公钥哈希 (20 字节)
       │          │
       │          ▼  版本前缀 0x00 + 双 SHA-256 校验和 + Base58
       │      P2PKH 地址（以 '1' 开头）
       │
       └─▶ SHA-256 + RIPEMD-160
                  │
                  ▼  Bech32 编码（witness v0）
              Bech32 地址（以 'bc1' 开头）
```

椭圆曲线方程（secp256k1）：

```
y² = x³ + 7  (mod p)
p = 2²⁵⁶ − 2³² − 977  （一个巨大的素数）
```

私钥到公钥的推导是单向的，在计算上不可逆（离散对数难题）。

---

## Wallet 结构体

`Wallet` 是项目中最常用的钱包类型，定义于 `src/wallet.rs`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// 比特币 P2PKH 地址（以 '1' 开头）
    pub address: String,
    /// 压缩公钥的十六进制表示（33 字节 = 66 hex 字符）
    pub public_key: String,
    /// secp256k1 私钥（序列化时以十六进制存储，访问受限）
    #[serde(with = "secret_key_serde")]
    private_key: SecretKey,
}
```

**字段说明：**
- `address`：P2PKH 格式地址，公开使用，可安全分享给他人作为收款地址
- `public_key`：压缩公钥（33 字节），用于验证签名，包含在每个交易输入中
- `private_key`：私钥，必须严格保密，拥有私钥等同于拥有对应地址的所有资金

---

## 创建钱包

### 生成随机钱包

```rust
use bitcoin_simulation::wallet::Wallet;

let wallet = Wallet::new();

println!("地址:   {}", wallet.address);       // 以 '1' 开头的 P2PKH 地址
println!("公钥:   {}", wallet.public_key);    // 66 字符十六进制
println!("私钥:   {}", wallet.private_key_hex()); // 64 字符十六进制（保密！）
```

`Wallet::new()` 内部流程：

```rust
pub fn new() -> Self {
    let secp = Secp256k1::new();
    // 使用密码学安全的随机数生成器（OsRng）
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
    let address = Self::pubkey_to_address(&public_key);
    let public_key_hex = hex::encode(public_key.serialize());

    Wallet { address, public_key: public_key_hex, private_key: secret_key }
}
```

### 创世钱包

创世钱包使用固定的私钥 `0x01`，每次启动都生成相同的地址，方便演示时花费创世区块中的初始资金：

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

// 两种等价的获取方式
let genesis = Blockchain::genesis_wallet();
let genesis2 = Wallet::genesis();

assert_eq!(genesis.address, genesis2.address); // 地址确定性一致

// 内部实现（src/wallet.rs）
pub fn genesis() -> Self {
    Self::from_private_key_hex(
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .expect("genesis private key is valid")
}
```

> **警告：** 创世钱包的私钥是公开的，切勿在生产环境中使用。

### 从私钥恢复钱包

```rust
// 从十六进制私钥恢复
let wallet = Wallet::new();
let hex = wallet.private_key_hex();  // 导出私钥

let recovered = Wallet::from_private_key_hex(&hex)?;
assert_eq!(wallet.address, recovered.address);  // 地址完全一致
```

---

## P2PKH 地址推导

`Wallet::pubkey_to_address()` 实现了与真实比特币一致的地址生成步骤：

```rust
fn pubkey_to_address(public_key: &PublicKey) -> String {
    // 步骤 1：压缩公钥序列化（33 字节：1 字节前缀 + 32 字节 x 坐标）
    let pubkey_bytes = public_key.serialize();

    // 步骤 2：SHA-256 哈希
    let sha256_hash = sha256::Hash::hash(&pubkey_bytes);

    // 步骤 3：RIPEMD-160 哈希 → 公钥哈希（20 字节）
    let mut ripemd = Ripemd160::new();
    ripemd.update(&sha256_hash[..]);
    let pubkey_hash = ripemd.finalize();

    // 步骤 4：添加版本字节（主网 = 0x00）
    let mut versioned = vec![0x00];
    versioned.extend_from_slice(&pubkey_hash);  // 总共 21 字节

    // 步骤 5：双 SHA-256 取前 4 字节作为校验和
    let checksum = sha256d::Hash::hash(&versioned);
    versioned.extend_from_slice(&checksum[0..4]);  // 总共 25 字节

    // 步骤 6：Base58 编码 → 以 '1' 开头的地址（约 34 字符）
    bs58::encode(versioned).into_string()
}
```

**为什么用 RIPEMD-160？**
- 将 33 字节公钥压缩为 20 字节，节省区块链存储空间
- 即使量子计算机破解了 ECDSA，攻击者仍需额外破解哈希函数

**为什么用 Base58（而非 Base64）？**
- 去掉了容易混淆的字符：0（零）、O（大写 O）、I（大写 i）、l（小写 L）
- 避免双击复制时包含空格等问题

---

## 交易签名

`Wallet::sign()` 使用私钥对数据生成 ECDSA 签名：

```rust
pub fn sign(&self, data: &str) -> String {
    let secp = Secp256k1::new();
    // 1. 对原始数据进行 SHA-256 哈希
    let msg_hash = sha256::Hash::hash(data.as_bytes());
    let message = Message::from_digest(msg_hash.to_byte_array());
    // 2. 使用私钥生成 ECDSA 签名
    let signature = secp.sign_ecdsa(&message, &self.private_key);
    // 3. DER 编码后返回十六进制字符串
    hex::encode(signature.serialize_der())
}
```

在 `Blockchain::create_transaction()` 中，签名的数据是 `"{txid}{vout}"`，即被引用 UTXO 的位置标识：

```rust
// src/blockchain.rs 节选
for (txid, vout) in utxos {
    let signature = from_wallet.sign(&format!("{}{}", txid, vout));
    let input = TxInput::new(txid, vout, signature, from_wallet.public_key.clone());
    inputs.push(input);
}
```

这样每个输入的签名都绑定到具体的 UTXO，防止签名被重放到其他 UTXO 上。

---

## 签名验证

`Wallet::verify_signature()` 是静态方法，不需要持有私钥：

```rust
pub fn verify_signature(public_key_hex: &str, data: &str, signature_hex: &str) -> bool {
    // 1. 解码公钥
    let Ok(pubkey_bytes) = hex::decode(public_key_hex) else { return false; };
    let Ok(public_key) = PublicKey::from_slice(&pubkey_bytes) else { return false; };

    // 2. 解码 DER 签名
    let Ok(sig_bytes) = hex::decode(signature_hex) else { return false; };
    let Ok(signature) = Signature::from_der(&sig_bytes) else { return false; };

    // 3. 重新哈希原始数据（与签名时完全一致）
    let secp = Secp256k1::new();
    let msg_hash = sha256::Hash::hash(data.as_bytes());
    let message = Message::from_digest(msg_hash.to_byte_array());

    // 4. 数学验证：检查签名是否由对应私钥生成
    secp.verify_ecdsa(&message, &signature, &public_key).is_ok()
}
```

完整的签名 + 验证示例：

```rust
use bitcoin_simulation::wallet::Wallet;

let wallet = Wallet::new();
let data = "Hello, Bitcoin!";

// 签名
let signature = wallet.sign(data);
println!("签名: {}", &signature[..32]); // DER 编码的十六进制

// 验证正确数据：应通过
assert!(Wallet::verify_signature(&wallet.public_key, data, &signature));

// 验证篡改数据：应失败
assert!(!Wallet::verify_signature(&wallet.public_key, "Tampered!", &signature));

// 用错误公钥验证：应失败
let other = Wallet::new();
assert!(!Wallet::verify_signature(&other.public_key, data, &signature));
```

---

## 扩展钱包：CryptoWallet

`src/crypto.rs` 中的 `CryptoWallet` 在 `Wallet` 基础上增加了更多比特币协议功能：

```rust
use bitcoin_simulation::crypto::CryptoWallet;

let wallet = CryptoWallet::new();

println!("P2PKH 地址:   {}", wallet.address);           // 以 '1' 开头
println!("Bech32 地址:  {}", wallet.bech32_address);    // 以 'bc1' 开头（SegWit）
println!("私钥十六进制: {}", wallet.private_key_hex()); // 64 字符
println!("公钥十六进制: {}", wallet.public_key_hex());  // 66 字符
```

### WIF 私钥格式

WIF（Wallet Import Format）是比特币钱包之间导入导出私钥的标准格式：

```rust
let wallet = CryptoWallet::new();

// 导出为 WIF（以 '5'、'K' 或 'L' 开头）
let wif = wallet.export_private_key_wif();
println!("WIF: {}", wif);

// 从 WIF 恢复钱包
let imported = CryptoWallet::import_from_wif(&wif)?;
assert_eq!(wallet.address, imported.address);
```

WIF 格式的编码步骤：
1. 添加版本字节 `0x80`（主网私钥前缀）
2. 计算双 SHA-256 校验和（取前 4 字节）
3. 拼接后进行 Base58 编码

### CryptoWallet 签名接口

`CryptoWallet` 的签名接口接受字节切片，更灵活：

```rust
let wallet = CryptoWallet::new();
let message = b"Hello, Bitcoin!";

// 签名（返回 secp256k1::ecdsa::Signature 类型）
let signature = wallet.sign(message);

// 验证（静态方法）
assert!(CryptoWallet::verify(message, &signature, &wallet.public_key));
```

---

## 钱包序列化

`Wallet` 和 `CryptoWallet` 均实现了 `Serialize` / `Deserialize`，私钥以十六进制字符串安全存储：

```rust
use bitcoin_simulation::wallet::Wallet;

let wallet = Wallet::new();

// 序列化为 JSON
let json = serde_json::to_string(&wallet)?;
// {"address":"1...","public_key":"02...","private_key":"a1b2c3..."}

// 从 JSON 恢复，签名能力完整保留
let restored: Wallet = serde_json::from_str(&json)?;
let sig = restored.sign("test");
assert!(Wallet::verify_signature(&restored.public_key, "test", &sig));
```

---

## 安全建议

| 注意事项 | 说明 |
|---------|------|
| 私钥保密 | 任何人获得私钥即可花费对应地址的全部资金 |
| 不要重用地址 | 每次收款使用新地址，保护隐私 |
| 创世钱包仅用于演示 | `Wallet::genesis()` 使用公开私钥，绝不能用于真实资金 |
| 备份私钥 | 丢失私钥意味着永久失去对应资金 |
| 使用 WIF 格式备份 | WIF 格式含校验和，可检测录入错误 |
