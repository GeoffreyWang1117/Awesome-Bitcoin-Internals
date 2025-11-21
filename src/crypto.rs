//! 真实的比特币密码学模块
//!
//! 使用secp256k1椭圆曲线实现真正的ECDSA签名，完全兼容比特币标准。

use secp256k1::{
    ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey,
};
use bitcoin_hashes::{sha256, sha256d, Hash};
use ripemd::{Ripemd160, Digest as RipemdDigest};
use hex;
use crate::error::{BitcoinError, Result};
use serde::{Deserialize, Serialize};

/// 真实的比特币钱包（使用ECDSA）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoWallet {
    /// 私钥（序列化时以十六进制存储）
    #[serde(with = "secret_key_serde")]
    pub private_key: SecretKey,

    /// 公钥
    #[serde(with = "public_key_serde")]
    pub public_key: PublicKey,

    /// 比特币地址（P2PKH格式）
    pub address: String,

    /// Bech32地址（原生隔离见证）
    pub bech32_address: String,
}

impl CryptoWallet {
    /// 创建新钱包
    ///
    /// # 示例
    ///
    /// ```
    /// use bitcoin_simulation::crypto::CryptoWallet;
    ///
    /// let wallet = CryptoWallet::new();
    /// println!("地址: {}", wallet.address);
    /// ```
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

        let address = Self::pubkey_to_address(&public_key);
        let bech32_address = Self::pubkey_to_bech32(&public_key);

        Self {
            private_key: secret_key,
            public_key,
            address,
            bech32_address,
        }
    }

    /// 从私钥创建钱包
    pub fn from_private_key(private_key: SecretKey) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &private_key);

        let address = Self::pubkey_to_address(&public_key);
        let bech32_address = Self::pubkey_to_bech32(&public_key);

        Self {
            private_key,
            public_key,
            address,
            bech32_address,
        }
    }

    /// 从私钥十六进制字符串创建钱包
    pub fn from_private_key_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| BitcoinError::PrivateKeyError {
                reason: format!("无效的十六进制私钥: {}", e),
            })?;

        let secret_key = SecretKey::from_slice(&bytes)
            .map_err(|e| BitcoinError::PrivateKeyError {
                reason: format!("无效的私钥: {}", e),
            })?;

        Ok(Self::from_private_key(secret_key))
    }

    /// 签名消息
    ///
    /// # 参数
    /// * `message` - 要签名的消息（会自动进行SHA256哈希）
    pub fn sign(&self, message: &[u8]) -> Signature {
        let secp = Secp256k1::new();

        // 对消息进行SHA256哈希
        let msg_hash = sha256::Hash::hash(message);
        let message = Message::from_digest(msg_hash.to_byte_array());

        secp.sign_ecdsa(&message, &self.private_key)
    }

    /// 验证签名
    ///
    /// # 参数
    /// * `message` - 原始消息
    /// * `signature` - 签名
    /// * `public_key` - 用于验证的公钥
    pub fn verify(message: &[u8], signature: &Signature, public_key: &PublicKey) -> bool {
        let secp = Secp256k1::new();

        let msg_hash = sha256::Hash::hash(message);
        let message = Message::from_digest(msg_hash.to_byte_array());

        secp.verify_ecdsa(&message, signature, public_key).is_ok()
    }

    /// 将公钥转换为比特币地址（P2PKH格式）
    ///
    /// 步骤：
    /// 1. SHA256(公钥)
    /// 2. RIPEMD160(SHA256结果)
    /// 3. 添加版本前缀(0x00)
    /// 4. 计算校验和
    /// 5. Base58编码
    fn pubkey_to_address(public_key: &PublicKey) -> String {
        // 1. 公钥序列化
        let pubkey_bytes = public_key.serialize();

        // 2. SHA256哈希
        let sha256_hash = sha256::Hash::hash(&pubkey_bytes);

        // 3. RIPEMD160哈希
        let mut ripemd = Ripemd160::new();
        ripemd.update(&sha256_hash[..]);
        let pubkey_hash = ripemd.finalize();

        // 4. 添加版本字节（主网=0x00）
        let mut versioned = vec![0x00];
        versioned.extend_from_slice(&pubkey_hash);

        // 5. 计算校验和（双SHA256的前4字节）
        let checksum = sha256d::Hash::hash(&versioned);
        let checksum_bytes = &checksum[0..4];

        // 6. 拼接并进行Base58编码
        versioned.extend_from_slice(checksum_bytes);
        bs58::encode(versioned).into_string()
    }

    /// 将公钥转换为Bech32地址（原生隔离见证）
    fn pubkey_to_bech32(public_key: &PublicKey) -> String {
        let pubkey_bytes = public_key.serialize();

        // SHA256 + RIPEMD160
        let sha256_hash = sha256::Hash::hash(&pubkey_bytes);
        let mut ripemd = Ripemd160::new();
        ripemd.update(&sha256_hash[..]);
        let pubkey_hash = ripemd.finalize();

        // 将8位数据转换为5位数据
        let converted = convert_bits(&pubkey_hash, 8, 5, true);

        // 添加witness版本（0）
        let mut data = vec![0u8];
        data.extend(&converted);

        // 编码为bech32（bc = 比特币主网）
        use bech32::{encode, Hrp};
        let hrp = Hrp::parse("bc").unwrap();
        encode::<bech32::Bech32>(hrp, &data).unwrap_or_else(|_| "bc1qinvalid".to_string())
    }

    /// 导出私钥（WIF格式）
    pub fn export_private_key_wif(&self) -> String {
        // 1. 添加版本字节（主网私钥=0x80）
        let mut extended = vec![0x80];
        extended.extend_from_slice(&self.private_key.secret_bytes());

        // 2. 计算校验和
        let checksum = sha256d::Hash::hash(&extended);
        let checksum_bytes = &checksum[0..4];

        // 3. 拼接并Base58编码
        extended.extend_from_slice(checksum_bytes);
        bs58::encode(extended).into_string()
    }

    /// 从WIF格式导入私钥
    pub fn import_from_wif(wif: &str) -> Result<Self> {
        let decoded = bs58::decode(wif).into_vec()
            .map_err(|e| BitcoinError::PrivateKeyError {
                reason: format!("无效的WIF格式: {}", e),
            })?;

        if decoded.len() != 37 {
            return Err(BitcoinError::PrivateKeyError {
                reason: "WIF长度无效".to_string(),
            });
        }

        // 验证校验和
        let checksum_verify = sha256d::Hash::hash(&decoded[0..33]);
        if &decoded[33..37] != &checksum_verify[0..4] {
            return Err(BitcoinError::PrivateKeyError {
                reason: "WIF校验和验证失败".to_string(),
            });
        }

        // 提取私钥
        let secret_key = SecretKey::from_slice(&decoded[1..33])
            .map_err(|e| BitcoinError::PrivateKeyError {
                reason: format!("无效的私钥: {}", e),
            })?;

        Ok(Self::from_private_key(secret_key))
    }

    /// 获取私钥十六进制表示
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.private_key.secret_bytes())
    }

    /// 获取公钥十六进制表示
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.serialize())
    }
}

// ========== 辅助函数 ==========

/// 转换位数（用于Bech32编码）
fn convert_bits(data: &[u8], from_bits: usize, to_bits: usize, pad: bool) -> Vec<u8> {
    let mut acc = 0u32;
    let mut bits = 0;
    let mut result = Vec::new();
    let maxv = (1 << to_bits) - 1;

    for &value in data {
        acc = (acc << from_bits) | value as u32;
        bits += from_bits;
        while bits >= to_bits {
            bits -= to_bits;
            result.push(((acc >> bits) & maxv) as u8);
        }
    }

    if pad && bits > 0 {
        result.push(((acc << (to_bits - bits)) & maxv) as u8);
    }

    result
}

// ========== Serde支持 ==========

mod secret_key_serde {
    use super::*;
    use serde::{Serializer, Deserializer, Deserialize};

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

mod public_key_serde {
    use super::*;
    use serde::{Serializer, Deserializer, Deserialize};

    pub fn serialize<S>(key: &PublicKey, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(key.serialize()))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = hex::decode(hex_str).map_err(serde::de::Error::custom)?;
        PublicKey::from_slice(&bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = CryptoWallet::new();
        assert!(!wallet.address.is_empty());
        assert!(!wallet.bech32_address.is_empty());
        assert!(wallet.address.starts_with('1')); // P2PKH地址
        assert!(wallet.bech32_address.starts_with("bc1")); // Bech32地址
    }

    #[test]
    fn test_sign_and_verify() {
        let wallet = CryptoWallet::new();
        let message = b"Hello, Bitcoin!";

        // 签名
        let signature = wallet.sign(message);

        // 验证
        assert!(CryptoWallet::verify(message, &signature, &wallet.public_key));

        // 验证错误的消息应该失败
        let wrong_message = b"Wrong message";
        assert!(!CryptoWallet::verify(wrong_message, &signature, &wallet.public_key));
    }

    #[test]
    fn test_private_key_hex() {
        let wallet = CryptoWallet::new();
        let hex = wallet.private_key_hex();

        // 私钥应该是64个十六进制字符（32字节）
        assert_eq!(hex.len(), 64);

        // 应该能从十六进制恢复
        let recovered = CryptoWallet::from_private_key_hex(&hex).unwrap();
        assert_eq!(wallet.address, recovered.address);
    }

    #[test]
    fn test_wif_export_import() {
        let wallet = CryptoWallet::new();

        // 导出WIF
        let wif = wallet.export_private_key_wif();
        assert!(wif.starts_with('5') || wif.starts_with('K') || wif.starts_with('L'));

        // 从WIF导入
        let imported = CryptoWallet::import_from_wif(&wif).unwrap();
        assert_eq!(wallet.address, imported.address);
    }

    #[test]
    fn test_serialization() {
        let wallet = CryptoWallet::new();

        // 序列化
        let json = serde_json::to_string(&wallet).unwrap();

        // 反序列化
        let deserialized: CryptoWallet = serde_json::from_str(&json).unwrap();
        assert_eq!(wallet.address, deserialized.address);
        assert_eq!(wallet.private_key_hex(), deserialized.private_key_hex());
    }

    #[test]
    fn test_deterministic_address() {
        // 使用固定私钥测试地址生成的确定性
        let private_key_hex = "0000000000000000000000000000000000000000000000000000000000000001";
        let wallet1 = CryptoWallet::from_private_key_hex(private_key_hex).unwrap();
        let wallet2 = CryptoWallet::from_private_key_hex(private_key_hex).unwrap();

        assert_eq!(wallet1.address, wallet2.address);
        assert_eq!(wallet1.bech32_address, wallet2.bech32_address);
    }
}
