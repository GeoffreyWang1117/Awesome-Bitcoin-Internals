use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 交易输入 - 引用之前的交易输出（UTXO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: String,           // 被引用的交易ID
    pub vout: usize,            // 输出索引
    pub signature: String,      // 签名（简化版本，实际应该是数字签名）
    pub pub_key: String,        // 公钥（用于验证签名）
}

impl TxInput {
    pub fn new(txid: String, vout: usize, signature: String, pub_key: String) -> Self {
        TxInput {
            txid,
            vout,
            signature,
            pub_key,
        }
    }
}

/// 交易输出 - 未花费的交易输出（UTXO）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u64,             // 金额
    pub pub_key_hash: String,   // 接收者的公钥哈希（地址）
}

impl TxOutput {
    pub fn new(value: u64, address: String) -> Self {
        TxOutput {
            value,
            pub_key_hash: address,
        }
    }

    /// 检查是否可以被指定的地址解锁
    pub fn can_be_unlocked_with(&self, address: &str) -> bool {
        self.pub_key_hash == address
    }
}

/// 交易结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,             // 交易ID
    pub inputs: Vec<TxInput>,   // 输入列表
    pub outputs: Vec<TxOutput>, // 输出列表
    pub timestamp: u64,         // 时间戳
    pub fee: u64,               // 交易费用
}

impl Transaction {
    /// 创建新交易
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>, timestamp: u64, fee: u64) -> Self {
        let mut tx = Transaction {
            id: String::new(),
            inputs,
            outputs,
            timestamp,
            fee,
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 创建Coinbase交易（挖矿奖励 + 交易费用）
    pub fn new_coinbase(to: String, reward: u64, timestamp: u64, total_fees: u64) -> Self {
        let tx_out = TxOutput::new(reward + total_fees, to);
        let tx_in = TxInput {
            txid: String::new(),
            vout: 0,
            signature: String::from("coinbase"),
            pub_key: String::from("coinbase"),
        };

        let mut tx = Transaction {
            id: String::new(),
            inputs: vec![tx_in],
            outputs: vec![tx_out],
            timestamp,
            fee: 0, // Coinbase交易没有费用
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 计算交易哈希
    pub fn calculate_hash(&self) -> String {
        let tx_data = serde_json::to_string(&self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 检查是否是Coinbase交易
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].txid.is_empty()
    }

    /// 验证交易（简化版本）
    pub fn verify(&self) -> bool {
        // Coinbase交易总是有效的
        if self.is_coinbase() {
            return true;
        }

        // 检查是否有输入和输出
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        // 简化验证：检查签名是否存在
        for input in &self.inputs {
            if input.signature.is_empty() || input.pub_key.is_empty() {
                return false;
            }
        }

        true
    }

    /// 计算交易大小（字节）
    pub fn size(&self) -> usize {
        serde_json::to_string(self).unwrap_or_default().len()
    }

    /// 计算交易费率（satoshi/byte）
    pub fn fee_rate(&self) -> f64 {
        let size = self.size();
        if size == 0 {
            return 0.0;
        }
        self.fee as f64 / size as f64
    }

    /// 获取交易的输入总额
    pub fn input_sum(&self) -> u64 {
        // 注意：这需要从UTXO集合中查询，这里只是占位
        0
    }

    /// 获取交易的输出总额
    pub fn output_sum(&self) -> u64 {
        self.outputs.iter().map(|o| o.value).sum()
    }
}
