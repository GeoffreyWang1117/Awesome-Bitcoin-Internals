use std::time::{SystemTime, UNIX_EPOCH};
use crate::transaction::Transaction;
use crate::merkle::MerkleTree;
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
    pub merkle_root: String,             // Merkle树根哈希
}

impl Block {
    /// 创建新区块
    pub fn new(index: u32, transactions: Vec<Transaction>, previous_hash: String) -> Block {
        // 获取当前Unix时间戳
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 计算Merkle根
        let tx_ids: Vec<String> = transactions.iter().map(|tx| tx.id.clone()).collect();
        let merkle_tree = MerkleTree::new(&tx_ids);
        let merkle_root = merkle_tree.get_root_hash();

        // 创建初始区块，哈希为空，nonce为0
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            merkle_root,
        };

        // 计算区块哈希
        block.hash = block.calculate_hash();
        block
    }

    /// 计算区块哈希（包含Merkle根）
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        // 计算哈希（使用Merkle根而非完整交易数据，提升效率）
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.merkle_root, self.previous_hash, self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证交易是否在区块中（使用Merkle证明）
    pub fn verify_transaction_inclusion(&self, tx_id: &str, index: usize) -> bool {
        let tx_ids: Vec<String> = self.transactions.iter().map(|tx| tx.id.clone()).collect();
        let merkle_tree = MerkleTree::new(&tx_ids);

        if let Some(proof) = merkle_tree.get_proof(tx_id) {
            MerkleTree::verify_proof(tx_id, &proof, &self.merkle_root, index)
        } else {
            false
        }
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
