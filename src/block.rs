use std::time::{SystemTime, UNIX_EPOCH};
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

/// 表示区块链中的一个区块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,                      // 区块在链中的索引
    pub timestamp: u64,                  // 区块创建时间戳
    pub transactions: Vec<Transaction>,  // 区块中的交易列表
    pub previous_hash: String,           // 前一个区块的哈希
    pub hash: String,                    // 当前区块的哈希
    pub nonce: u64,                      // 挖矿使用的随机数（工作量证明）
}

impl Block {
    /// 创建新区块
    pub fn new(index: u32, transactions: Vec<Transaction>, previous_hash: String) -> Block {
        // 获取当前Unix时间戳
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 创建初始区块，哈希为空，nonce为0
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };

        // 计算区块哈希
        block.hash = block.calculate_hash();
        block
    }

    /// 计算区块哈希
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        // 序列化交易数据
        let tx_data = serde_json::to_string(&self.transactions).unwrap_or_default();

        // 计算哈希
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, tx_data, self.previous_hash, self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 挖矿 - 找到满足难度要求的哈希
    pub fn mine_block(&mut self, difficulty: usize) {
        // 目标：哈希以difficulty个0开头
        let target = "0".repeat(difficulty);

        // 不断增加nonce直到找到有效哈希
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        println!("区块已挖出: {}", self.hash);
    }

    /// 验证区块中的所有交易
    pub fn validate_transactions(&self) -> bool {
        for tx in &self.transactions {
            if !tx.verify() {
                return false;
            }
        }
        true
    }
}
