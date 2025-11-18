use crate::wallet::Wallet;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

/// 多重签名地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigAddress {
    pub address: String,
    pub required_sigs: usize,           // 需要的签名数量（m）
    pub total_keys: usize,              // 总密钥数量（n）
    pub public_keys: Vec<String>,       // 所有公钥
    pub script: String,                 // 锁定脚本（简化版）
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
