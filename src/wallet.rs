use bitcoin_hashes::{sha256, sha256d, Hash};
use ripemd::{Digest as RipemdDigest, Ripemd160};
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};

/// 钱包结构 - 使用真实的secp256k1椭圆曲线密码学
///
/// 密钥生成流程（与真实比特币一致）：
/// 1. 生成256位随机数作为私钥
/// 2. 使用secp256k1椭圆曲线从私钥推导公钥
/// 3. 对公钥进行SHA256 + RIPEMD160哈希
/// 4. 添加版本号和校验码，进行Base58编码得到P2PKH地址
///
/// 签名算法：ECDSA（椭圆曲线数字签名算法）
/// 曲线方程：y² = x³ + 7 (mod p)
///
/// 地址格式：P2PKH（以'1'开头），与真实比特币主网地址格式一致
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// 比特币P2PKH地址（以'1'开头）
    pub address: String,
    /// 压缩公钥的十六进制表示（33字节 = 66 hex chars）
    pub public_key: String,
    /// secp256k1私钥
    #[serde(with = "secret_key_serde")]
    private_key: SecretKey,
}

impl Wallet {
    /// 创建新钱包
    ///
    /// 使用系统随机数生成器创建secp256k1密钥对，
    /// 并从公钥推导出P2PKH地址。
    ///
    /// # 安全性
    /// - 私钥使用密码学安全的随机数生成器
    /// - 私钥必须保密，拥有私钥就拥有对应地址的所有比特币
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let address = Self::pubkey_to_address(&public_key);
        let public_key_hex = hex::encode(public_key.serialize());

        Wallet {
            address,
            public_key: public_key_hex,
            private_key: secret_key,
        }
    }

    /// 创建创世钱包（确定性，仅用于演示和测试）
    ///
    /// 使用固定私钥 `0x01` 创建钱包，保证每次启动时创世区块的地址一致。
    /// **警告：切勿在生产环境中使用已知私钥。**
    pub fn genesis() -> Self {
        Self::from_private_key_hex(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .expect("genesis private key is valid")
    }

    /// 从十六进制私钥恢复钱包
    ///
    /// # 参数
    /// * `hex_str` - 32字节（64个十六进制字符）的私钥
    pub fn from_private_key_hex(hex_str: &str) -> Result<Self, String> {
        let bytes = hex::decode(hex_str).map_err(|e| format!("无效的十六进制私钥: {}", e))?;
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&bytes).map_err(|e| format!("无效的私钥: {}", e))?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let address = Self::pubkey_to_address(&public_key);
        let public_key_hex = hex::encode(public_key.serialize());

        Ok(Wallet {
            address,
            public_key: public_key_hex,
            private_key: secret_key,
        })
    }

    /// 使用ECDSA对数据签名
    ///
    /// 签名流程：
    /// 1. 对数据进行SHA256哈希
    /// 2. 使用私钥和secp256k1曲线生成ECDSA签名
    /// 3. 返回DER编码签名的十六进制表示
    ///
    /// # 参数
    /// * `data` - 待签名的数据字符串
    ///
    /// # 返回值
    /// DER编码的ECDSA签名（十六进制字符串）
    pub fn sign(&self, data: &str) -> String {
        let secp = Secp256k1::new();
        let msg_hash = sha256::Hash::hash(data.as_bytes());
        let message = Message::from_digest(msg_hash.to_byte_array());
        let signature = secp.sign_ecdsa(&message, &self.private_key);
        hex::encode(signature.serialize_der())
    }

    /// 验证ECDSA签名
    ///
    /// 验证流程：
    /// 1. 从十六进制解码公钥和签名
    /// 2. 对数据进行SHA256哈希（与签名时相同）
    /// 3. 使用secp256k1验证签名的数学正确性
    ///
    /// # 参数
    /// * `public_key_hex` - 压缩公钥的十六进制表示
    /// * `data` - 原始数据
    /// * `signature_hex` - DER编码签名的十六进制表示
    ///
    /// # 返回值
    /// 签名有效返回true，否则返回false
    pub fn verify_signature(public_key_hex: &str, data: &str, signature_hex: &str) -> bool {
        let Ok(pubkey_bytes) = hex::decode(public_key_hex) else {
            return false;
        };
        let Ok(public_key) = PublicKey::from_slice(&pubkey_bytes) else {
            return false;
        };

        let Ok(sig_bytes) = hex::decode(signature_hex) else {
            return false;
        };
        let Ok(signature) = Signature::from_der(&sig_bytes) else {
            return false;
        };

        let secp = Secp256k1::new();
        let msg_hash = sha256::Hash::hash(data.as_bytes());
        let message = Message::from_digest(msg_hash.to_byte_array());

        secp.verify_ecdsa(&message, &signature, &public_key).is_ok()
    }

    /// 获取私钥的十六进制表示
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.private_key.secret_bytes())
    }

    /// 将公钥转换为P2PKH地址
    ///
    /// 步骤（与真实比特币一致）：
    /// 1. 压缩公钥序列化（33字节）
    /// 2. SHA256哈希
    /// 3. RIPEMD160哈希 → 20字节公钥哈希
    /// 4. 添加版本前缀 0x00（主网）
    /// 5. 双SHA256计算校验和（取前4字节）
    /// 6. Base58编码 → 最终地址（以'1'开头）
    fn pubkey_to_address(public_key: &PublicKey) -> String {
        let pubkey_bytes = public_key.serialize();

        // SHA256
        let sha256_hash = sha256::Hash::hash(&pubkey_bytes);

        // RIPEMD160
        let mut ripemd = Ripemd160::new();
        ripemd.update(&sha256_hash[..]);
        let pubkey_hash = ripemd.finalize();

        // 版本前缀 + 公钥哈希
        let mut versioned = vec![0x00];
        versioned.extend_from_slice(&pubkey_hash);

        // 校验和（双SHA256前4字节）
        let checksum = sha256d::Hash::hash(&versioned);
        versioned.extend_from_slice(&checksum[0..4]);

        // Base58编码
        bs58::encode(versioned).into_string()
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

// ========== SecretKey的Serde支持 ==========
mod secret_key_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(key: &SecretKey, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(key.secret_bytes()))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<SecretKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = hex::decode(hex_str).map_err(serde::de::Error::custom)?;
        SecretKey::from_slice(&bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(wallet.address.starts_with('1'), "P2PKH地址应以'1'开头");
        assert_eq!(wallet.public_key.len(), 66, "压缩公钥应为66 hex字符");
    }

    #[test]
    fn test_sign_and_verify() {
        let wallet = Wallet::new();
        let data = "Hello, Bitcoin!";
        let signature = wallet.sign(data);

        assert!(
            Wallet::verify_signature(&wallet.public_key, data, &signature),
            "签名验证应通过"
        );

        // 验证篡改的数据应该失败
        assert!(
            !Wallet::verify_signature(&wallet.public_key, "Tampered data", &signature),
            "篡改数据的签名验证应失败"
        );
    }

    #[test]
    fn test_wrong_key_fails() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let data = "test message";
        let signature = wallet1.sign(data);

        // 用wallet2的公钥验证wallet1的签名应失败
        assert!(
            !Wallet::verify_signature(&wallet2.public_key, data, &signature),
            "错误公钥的签名验证应失败"
        );
    }

    #[test]
    fn test_genesis_wallet_deterministic() {
        let w1 = Wallet::genesis();
        let w2 = Wallet::genesis();
        assert_eq!(w1.address, w2.address, "创世钱包地址应确定性一致");
        assert_eq!(w1.public_key, w2.public_key);
    }

    #[test]
    fn test_from_private_key_hex() {
        let wallet = Wallet::new();
        let hex = wallet.private_key_hex();
        let recovered = Wallet::from_private_key_hex(&hex).unwrap();
        assert_eq!(wallet.address, recovered.address);
        assert_eq!(wallet.public_key, recovered.public_key);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let wallet = Wallet::new();
        let json = serde_json::to_string(&wallet).unwrap();
        let deserialized: Wallet = serde_json::from_str(&json).unwrap();
        assert_eq!(wallet.address, deserialized.address);
        assert_eq!(wallet.public_key, deserialized.public_key);

        // 确认反序列化后签名仍然有效
        let data = "roundtrip test";
        let sig = deserialized.sign(data);
        assert!(Wallet::verify_signature(
            &deserialized.public_key,
            data,
            &sig
        ));
    }
}
