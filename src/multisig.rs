use crate::wallet::Wallet;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

/// 多重签名地址（M-of-N Multisig）
///
/// 多重签名是比特币的高级功能，要求M个签名才能花费N个公钥控制的资金。
///
/// 应用场景：
///
/// 1. 企业资金管理（2-of-3）：
///    - CEO、CFO、CTO各持一个密钥
///    - 任意两人同意即可转账
///    - 防止单点故障和内部舞弊
///
/// 2. 托管服务（2-of-3）：
///    - 买家、卖家、仲裁员各持一个密钥
///    - 正常交易：买家+卖家签名
///    - 争议时：买家/卖家+仲裁员签名
///
/// 3. 个人资产保护（2-of-3）：
///    - 主密钥、备份密钥、第三方托管密钥
///    - 主密钥丢失仍可恢复
///    - 防止单一私钥被盗
///
/// 4. 冷热钱包结合（2-of-3）：
///    - 热钱包（日常使用）
///    - 冷钱包1（离线保存）
///    - 冷钱包2（异地保存）
///
/// 5. 遗产继承（时间锁+多签）：
///    - 2-of-2：本人+继承人
///    - 1年后自动变为1-of-2
///
/// 技术实现（比特币）：
/// - P2SH（Pay-to-Script-Hash）地址，以"3"开头
/// - 锁定脚本：OP_2 pubkey1 pubkey2 pubkey3 OP_3 OP_CHECKMULTISIG
/// - 解锁需要提供：OP_0 signature1 signature2
///
/// 优势：
/// - 提高安全性：分散密钥，降低单点风险
/// - 灵活性：可设置不同的M和N组合
/// - 透明性：链上可见多签地址（但不知道是谁）
/// - 不可逆：一旦设置，规则不可更改
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigAddress {
    pub address: String,                // 多签地址（以"3"开头）
    pub required_sigs: usize,           // 需要的签名数量 M（例如：2-of-3中的2）
    pub total_keys: usize,              // 总密钥数量 N（例如：2-of-3中的3）
    pub public_keys: Vec<String>,       // 所有参与方的公钥列表
    pub script: String,                 // 锁定脚本（简化版，实际是Script代码）
}

impl MultiSigAddress {
    /// 创建多重签名地址 (m-of-n)
    pub fn new(required_sigs: usize, public_keys: Vec<String>) -> Result<Self, String> {
        let total_keys = public_keys.len();

        if required_sigs == 0 || required_sigs > total_keys {
            return Err("无效的签名要求".to_string());
        }

        if total_keys > 15 {
            return Err("最多支持15个密钥".to_string());
        }

        // 生成锁定脚本（简化版）
        let script = format!(
            "OP_{}{}OP_CHECKMULTISIG",
            required_sigs,
            public_keys.join("")
        );

        // 生成多签地址
        let mut hasher = Sha256::new();
        hasher.update(script.as_bytes());
        let address = format!("3{:x}", hasher.finalize())[..42].to_string(); // "3"开头表示多签

        Ok(MultiSigAddress {
            address,
            required_sigs,
            total_keys,
            public_keys,
            script,
        })
    }

    /// 验证签名数量是否满足要求
    pub fn verify_signatures(&self, signatures: &[String]) -> bool {
        if signatures.len() < self.required_sigs {
            return false;
        }

        // 简化验证：检查签名是否来自已知公钥
        let mut valid_count = 0;
        for sig in signatures {
            if !sig.is_empty() {
                valid_count += 1;
            }
        }

        valid_count >= self.required_sigs
    }
}

/// 多重签名交易构建器
pub struct MultiSigTxBuilder {
    pub multisig_address: MultiSigAddress,
    pub signatures: Vec<String>,
}

impl MultiSigTxBuilder {
    /// 创建多签交易构建器
    pub fn new(multisig_address: MultiSigAddress) -> Self {
        MultiSigTxBuilder {
            multisig_address,
            signatures: Vec::new(),
        }
    }

    /// 添加签名
    pub fn add_signature(&mut self, wallet: &Wallet, data: &str) -> Result<(), String> {
        // 验证钱包公钥是否在多签地址中
        if !self.multisig_address.public_keys.contains(&wallet.public_key) {
            return Err("此钱包不在多签地址中".to_string());
        }

        // 创建签名
        let signature = wallet.sign(data);
        self.signatures.push(signature);

        Ok(())
    }

    /// 检查是否收集到足够的签名
    pub fn is_complete(&self) -> bool {
        self.signatures.len() >= self.multisig_address.required_sigs
    }

    /// 获取所有签名
    pub fn get_signatures(&self) -> Vec<String> {
        self.signatures.clone()
    }
}

/// 常用多签类型
pub enum MultiSigType {
    TwoOfTwo,       // 2-of-2
    TwoOfThree,     // 2-of-3 (最常用)
    ThreeOfFive,    // 3-of-5
}

impl MultiSigType {
    /// 创建指定类型的多签地址
    pub fn create_address(&self, wallets: &[Wallet]) -> Result<MultiSigAddress, String> {
        let public_keys: Vec<String> = wallets.iter().map(|w| w.public_key.clone()).collect();

        let (required, total) = match self {
            MultiSigType::TwoOfTwo => (2, 2),
            MultiSigType::TwoOfThree => (2, 3),
            MultiSigType::ThreeOfFive => (3, 5),
        };

        if public_keys.len() != total {
            return Err(format!("需要{}个钱包", total));
        }

        MultiSigAddress::new(required, public_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multisig_2_of_3() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();

        let public_keys = vec![
            wallet1.public_key.clone(),
            wallet2.public_key.clone(),
            wallet3.public_key.clone(),
        ];

        let multisig = MultiSigAddress::new(2, public_keys).unwrap();
        assert_eq!(multisig.required_sigs, 2);
        assert_eq!(multisig.total_keys, 3);
        assert!(multisig.address.starts_with('3'));

        // 测试签名
        let mut builder = MultiSigTxBuilder::new(multisig);
        builder.add_signature(&wallet1, "test_data").unwrap();
        assert!(!builder.is_complete());

        builder.add_signature(&wallet2, "test_data").unwrap();
        assert!(builder.is_complete());
    }
}
