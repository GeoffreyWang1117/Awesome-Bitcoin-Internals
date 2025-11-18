use sha2::{Digest, Sha256};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 钱包结构 - 管理地址和密钥
///
/// 比特币钱包的核心功能是管理密钥对和地址。
///
/// 密钥生成流程（实际比特币）：
/// 1. 生成256位随机数作为私钥
/// 2. 使用椭圆曲线加密（secp256k1）从私钥生成公钥
/// 3. 对公钥进行SHA256哈希
/// 4. 对结果进行RIPEMD160哈希得到公钥哈希
/// 5. 添加版本号和校验码，进行Base58编码得到地址
///
/// 地址格式：
/// - P2PKH（传统地址）：以1开头，如 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
/// - P2SH（脚本地址）：以3开头，用于多签等高级功能
/// - Bech32（SegWit地址）：以bc1开头，如 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
///
/// 本实现为简化版本，使用SHA256模拟密钥生成。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,        // 钱包地址（公钥哈希，简化为40字符十六进制）
    pub private_key: String,    // 私钥（简化版本，64字符十六进制）
    pub public_key: String,     // 公钥（简化版本，64字符十六进制）
}

impl Wallet {
    /// 创建新钱包
    ///
    /// 生成一个全新的钱包，包含：
    /// 1. 随机生成的私钥
    /// 2. 从私钥派生的公钥
    /// 3. 从公钥哈希得到的地址
    ///
    /// 安全性提示：
    /// - 私钥必须保密，拥有私钥就拥有对应地址的所有比特币
    /// - 私钥丢失无法恢复，比特币将永久丢失
    /// - 建议使用硬件钱包或冷存储保管大额私钥
    ///
    /// # 返回值
    /// 返回新创建的钱包实例
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
    ///
    /// 注意：这个方法仅用于演示和测试。
    /// 在实际应用中，不应该为任意地址生成假的私钥。
    /// 真实的钱包应该从已知的私钥或助记词恢复。
    ///
    /// # 参数
    /// * `address` - 指定的地址
    ///
    /// # 返回值
    /// 返回使用指定地址的钱包（私钥和公钥是随机生成的，不对应该地址）
    pub fn from_address(address: String) -> Self {
        let (private_key, public_key) = Wallet::generate_keypair();

        Wallet {
            address,
            private_key,
            public_key,
        }
    }

    /// 生成密钥对（简化版本 - 实际应该使用椭圆曲线加密）
    ///
    /// 实际比特币使用secp256k1椭圆曲线：
    /// - 曲线方程: y² = x³ + 7 (mod p)
    /// - 私钥：256位随机数（32字节）
    /// - 公钥：椭圆曲线点（压缩格式33字节，非压缩65字节）
    /// - 签名算法：ECDSA（椭圆曲线数字签名算法）
    ///
    /// 本实现简化为：
    /// - 私钥：64字符随机十六进制字符串
    /// - 公钥：私钥的SHA256哈希
    ///
    /// # 返回值
    /// 返回(私钥, 公钥)元组
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
    ///
    /// 实际比特币地址生成过程：
    /// 1. SHA256(公钥) → 32字节哈希
    /// 2. RIPEMD160(SHA256结果) → 20字节公钥哈希
    /// 3. 添加版本前缀（主网0x00，测试网0x6F）
    /// 4. 计算校验和：SHA256(SHA256(版本+哈希))的前4字节
    /// 5. Base58编码(版本+哈希+校验和) → 最终地址
    ///
    /// 本实现简化为：
    /// - 只使用SHA256哈希
    /// - 取前40字符（20字节）作为地址
    ///
    /// # 参数
    /// * `public_key` - 公钥
    ///
    /// # 返回值
    /// 返回地址字符串
    fn hash_public_key(public_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();

        // 取前20字节作为地址（类似比特币的RIPEMD160结果）
        format!("{:x}", hash)[..40].to_string()
    }

    /// 签名数据（简化版本）
    ///
    /// 实际比特币使用ECDSA签名：
    /// 1. 对待签名数据进行双重SHA256哈希
    /// 2. 使用私钥和secp256k1曲线生成签名
    /// 3. 签名包含两部分：r和s（各32字节）
    /// 4. 添加签名类型标志（SIGHASH_ALL等）
    ///
    /// 签名的作用：
    /// - 证明拥有私钥（无需暴露私钥）
    /// - 防止交易被篡改
    /// - 实现不可抵赖性
    ///
    /// 本实现简化为：SHA256(私钥 + 数据)
    ///
    /// # 参数
    /// * `data` - 待签名的数据
    ///
    /// # 返回值
    /// 返回签名字符串
    pub fn sign(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", self.private_key, data).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证签名（简化版本）
    ///
    /// 实际比特币使用ECDSA验证：
    /// 1. 从签名恢复公钥
    /// 2. 验证公钥是否匹配
    /// 3. 验证签名的数学正确性
    ///
    /// 本实现极度简化，仅检查签名和公钥非空。
    /// 实际应用必须实现完整的密码学验证。
    ///
    /// # 参数
    /// * `public_key` - 公钥
    /// * `data` - 原始数据
    /// * `signature` - 签名
    ///
    /// # 返回值
    /// 如果签名有效返回true，否则返回false
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
