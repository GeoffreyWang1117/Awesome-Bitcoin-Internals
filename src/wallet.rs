use sha2::{Digest, Sha256};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 钱包结构 - 管理地址和密钥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,        // 钱包地址（公钥哈希）
    pub private_key: String,    // 私钥（简化版本）
    pub public_key: String,     // 公钥（简化版本）
}

impl Wallet {
    /// 创建新钱包
    pub fn new() -> Self {
        let (private_key, public_key) = Wallet::generate_keypair();
        let address = Wallet::hash_public_key(&public_key);

        Wallet {
            address,
            private_key,
            public_key,
        }
    }

    /// 从已知地址创建钱包（用于演示）
    pub fn from_address(address: String) -> Self {
        let (private_key, public_key) = Wallet::generate_keypair();

        Wallet {
            address,
            private_key,
            public_key,
        }
    }

    /// 生成密钥对（简化版本 - 实际应该使用椭圆曲线加密）
    fn generate_keypair() -> (String, String) {
        let mut rng = rand::thread_rng();
        let private_key: String = (0..64)
            .map(|_| format!("{:x}", rng.gen::<u8>()))
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(private_key.as_bytes());
        let public_key = format!("{:x}", hasher.finalize());

        (private_key, public_key)
    }

    /// 对公钥进行哈希得到地址
    fn hash_public_key(public_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();

        // 取前20字节作为地址（类似比特币）
        format!("{:x}", hash)[..40].to_string()
    }

    /// 签名数据（简化版本）
    pub fn sign(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", self.private_key, data).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证签名（简化版本）
    pub fn verify_signature(public_key: &str, data: &str, signature: &str) -> bool {
        // 简化验证 - 实际应该使用椭圆曲线验证
        !signature.is_empty() && !public_key.is_empty()
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}
