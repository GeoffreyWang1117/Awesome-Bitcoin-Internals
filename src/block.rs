use crate::merkle::MerkleTree;
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 区块（Block）- 区块链的基本组成单位
///
/// 区块是区块链的核心数据结构，包含一组已确认的交易。
///
/// 区块头（Block Header）包含：
/// - index: 区块高度（第几个区块）
/// - timestamp: 区块创建时间
/// - previous_hash: 前一个区块的哈希（形成链式结构）
/// - merkle_root: 交易Merkle树的根哈希
/// - nonce: 工作量证明的随机数
/// - hash: 当前区块的哈希值
///
/// 区块体（Block Body）包含：
/// - transactions: 交易列表（第一笔必须是Coinbase交易）
///
/// 区块链的链式结构：
/// Genesis Block (index=0) -> Block 1 -> Block 2 -> ... -> Latest Block
///     hash: abc...             hash: def...
///     prev: "0"                prev: abc...
///
/// 区块的不可篡改性：
/// - 任何交易的改变都会改变merkle_root
/// - merkle_root的改变会导致hash完全不同
/// - hash的改变会破坏下一个区块的previous_hash引用
/// - 因此要篡改历史区块，必须重新挖所有后续区块（计算上不可行）
///
/// 比特币区块大小限制：
/// - 原始限制：1 MB
/// - SegWit后：理论上可达4 MB（使用witness数据）
/// - 平均包含2000-3000笔交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,                     // 区块高度（索引），创世区块为0
    pub timestamp: u64,                 // Unix时间戳（秒）
    pub transactions: Vec<Transaction>, // 交易列表（第一笔是Coinbase）
    pub previous_hash: String,          // 父区块哈希（SHA256，64字符）
    pub hash: String,                   // 当前区块哈希（通过挖矿找到）
    pub nonce: u64,                     // 工作量证明的随机数（挖矿调整此值）
    pub merkle_root: String,            // 交易Merkle树根（用于高效验证交易）
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

    /// 挖矿 - 找到满足难度要求的哈希（工作量证明 Proof of Work）
    ///
    /// 工作量证明（PoW）是比特币共识机制的核心。
    ///
    /// 原理：
    /// - 找到一个nonce值，使得区块哈希满足特定条件（开头有N个0）
    /// - 由于哈希函数的不可预测性，只能通过暴力穷举
    /// - 平均需要尝试2^difficulty次才能成功
    ///
    /// 难度调整：
    /// - 比特币目标：平均10分钟产生一个区块
    /// - 每2016个区块（约2周）调整一次难度
    /// - 如果出块太快，增加难度；太慢则降低难度
    /// - 当前比特币难度约为20个前导0（需要约2^80次哈希计算）
    ///
    /// 能源消耗：
    /// - 全网算力达到数百EH/s（exahashes per second）
    /// - 这种巨大的计算量保证了网络安全
    /// - 攻击者需要掌握全网51%算力才可能发动攻击
    ///
    /// 挖矿奖励：
    /// - 区块奖励：最初50 BTC，每210,000区块减半
    /// - 2024年减半后为3.125 BTC
    /// - 加上区块内所有交易的手续费
    ///
    /// # 参数
    /// * `difficulty` - 挖矿难度（前导0的个数）
    ///
    /// # 副作用
    /// 修改区块的nonce和hash字段，直到找到有效哈希
    pub fn mine_block(&mut self, difficulty: usize) {
        // 目标：哈希以difficulty个0开头
        let target = "0".repeat(difficulty);

        // 不断增加nonce直到找到有效哈希（暴力穷举）
        // 实际比特币矿工会并行尝试大量nonce值
        while self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        println!("✓ 区块已挖出: {}", self.hash);
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
