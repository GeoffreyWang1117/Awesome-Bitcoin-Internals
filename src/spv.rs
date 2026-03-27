//! SPV (Simplified Payment Verification) 轻客户端
//!
//! 实现比特币白皮书第8节描述的简化支付验证。
//!
//! # SPV原理
//!
//! SPV客户端不需要下载和验证所有交易，只需：
//! 1. 下载所有区块头（每个80字节）
//! 2. 获取与自己相关交易的Merkle证明
//! 3. 信任最长工作量证明链
//!
//! # 对比
//!
//! | 特性 | 全节点 | SPV节点 |
//! |------|--------|---------|
//! | 存储 | 400+ GB | ~5 MB |
//! | 带宽 | 完整区块 | 仅区块头 |
//! | 验证 | 所有交易 | 仅相关交易 |
//! | 安全性 | 最高 | 依赖PoW |
//!
//! # 使用场景
//!
//! - 移动钱包
//! - 嵌入式设备
//! - 轻量级客户端
//! - IoT设备
//!
//! # 示例
//!
//! ```no_run
//! use bitcoin_simulation::spv::SPVClient;
//!
//! let mut client = SPVClient::new();
//!
//! // 添加区块头
//! // client.add_block_header(header)?;
//!
//! // 验证交易
//! // let valid = client.verify_transaction("txid", &proof, &merkle_root)?;
//! # Ok::<(), bitcoin_simulation::error::BitcoinError>(())
//! ```

use crate::block::Block;
use crate::error::{BitcoinError, Result};
use crate::info;
use crate::merkle::MerkleTree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 区块头 (80字节)
///
/// 比特币区块头包含：
/// - version (4 bytes)
/// - previous_hash (32 bytes)
/// - merkle_root (32 bytes)
/// - timestamp (4 bytes)
/// - bits (4 bytes)
/// - nonce (4 bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// 区块高度
    pub height: u32,

    /// 区块哈希
    pub hash: String,

    /// 前一个区块哈希
    pub previous_hash: String,

    /// Merkle根
    pub merkle_root: String,

    /// 时间戳
    pub timestamp: u64,

    /// 难度目标
    pub bits: u32,

    /// 随机数
    pub nonce: u64,
}

impl BlockHeader {
    /// 从完整区块创建区块头
    pub fn from_block(block: &Block) -> Self {
        Self {
            height: block.index,
            hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            merkle_root: block.merkle_root.clone(),
            timestamp: block.timestamp,
            bits: 0, // 简化版不包含难度
            nonce: block.nonce,
        }
    }

    /// 验证区块头的工作量证明
    pub fn verify_pow(&self, difficulty: usize) -> bool {
        let target = "0".repeat(difficulty);
        self.hash[..difficulty] == target
    }

    /// 计算区块头大小（字节）
    pub fn size() -> usize {
        80 // 固定80字节
    }
}

/// SPV客户端
pub struct SPVClient {
    /// 区块头链
    headers: Vec<BlockHeader>,

    /// 区块头索引: hash -> header
    header_index: HashMap<String, BlockHeader>,

    /// 已验证的交易: txid -> (block_hash, verified)
    verified_transactions: HashMap<String, (String, bool)>,

    /// 最长链的tip
    chain_tip: Option<String>,

    /// 总工作量
    total_work: u64,
}

impl SPVClient {
    /// 创建新的SPV客户端
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            header_index: HashMap::new(),
            verified_transactions: HashMap::new(),
            chain_tip: None,
            total_work: 0,
        }
    }

    /// 添加区块头
    ///
    /// SPV客户端只下载区块头，不下载完整区块
    pub fn add_block_header(&mut self, header: BlockHeader) -> Result<()> {
        // 1. 验证区块头连接
        if !self.headers.is_empty() {
            let last_header = self.headers.last().unwrap();
            if header.previous_hash != last_header.hash {
                return Err(BitcoinError::InvalidBlock {
                    reason: format!(
                        "区块头不连续: 期望 previous_hash={}, 实际={}",
                        last_header.hash, header.previous_hash
                    ),
                });
            }

            if header.height != last_header.height + 1 {
                return Err(BitcoinError::InvalidBlock {
                    reason: format!(
                        "区块高度不连续: 期望 {}, 实际 {}",
                        last_header.height + 1,
                        header.height
                    ),
                });
            }
        }

        // 2. 添加到链
        let hash = header.hash.clone();
        self.headers.push(header.clone());
        self.header_index.insert(hash.clone(), header);

        // 3. 更新tip
        self.chain_tip = Some(hash);
        self.total_work += 1;

        info!(
            "SPV客户端添加区块头 #{}, 总数: {}",
            self.headers.len(),
            self.headers.len()
        );

        Ok(())
    }

    /// 批量添加区块头
    pub fn add_block_headers(&mut self, headers: Vec<BlockHeader>) -> Result<()> {
        for header in headers {
            self.add_block_header(header)?;
        }
        Ok(())
    }

    /// 验证交易包含性（使用Merkle证明）
    ///
    /// # 参数
    /// * `tx_id` - 交易ID
    /// * `proof` - Merkle证明
    /// * `block_hash` - 区块哈希
    /// * `tx_index` - 交易在区块中的索引
    pub fn verify_transaction(
        &mut self,
        tx_id: &str,
        proof: &[String],
        block_hash: &str,
        tx_index: usize,
    ) -> Result<bool> {
        // 1. 获取区块头
        let header =
            self.header_index
                .get(block_hash)
                .ok_or_else(|| BitcoinError::InvalidBlock {
                    reason: format!("区块头未找到: {}", block_hash),
                })?;

        // 2. 验证Merkle证明
        let valid = MerkleTree::verify_proof(tx_id, proof, &header.merkle_root, tx_index);

        // 3. 记录验证结果
        self.verified_transactions
            .insert(tx_id.to_string(), (block_hash.to_string(), valid));

        if valid {
            info!("✓ SPV验证成功: 交易 {} 在区块 {} 中", tx_id, block_hash);
        } else {
            info!("✗ SPV验证失败: 交易 {} 不在区块 {} 中", tx_id, block_hash);
        }

        Ok(valid)
    }

    /// 获取区块头
    pub fn get_header(&self, hash: &str) -> Option<&BlockHeader> {
        self.header_index.get(hash)
    }

    /// 获取链高度
    pub fn get_height(&self) -> u32 {
        self.headers.len() as u32
    }

    /// 获取最新区块头
    pub fn get_tip(&self) -> Option<&BlockHeader> {
        self.headers.last()
    }

    /// 检查交易是否已验证
    pub fn is_transaction_verified(&self, tx_id: &str) -> Option<bool> {
        self.verified_transactions
            .get(tx_id)
            .map(|(_, verified)| *verified)
    }

    /// 获取存储大小估算（字节）
    pub fn estimate_storage_size(&self) -> usize {
        // 每个区块头80字节
        self.headers.len() * BlockHeader::size()
    }

    /// 获取SPV客户端统计信息
    pub fn get_stats(&self) -> SPVStats {
        SPVStats {
            header_count: self.headers.len(),
            storage_size: self.estimate_storage_size(),
            verified_tx_count: self.verified_transactions.len(),
            chain_height: self.get_height(),
            total_work: self.total_work,
        }
    }

    /// 同步区块头（从完整节点）
    ///
    /// 在实际应用中，这会通过P2P网络从全节点获取区块头
    pub fn sync_from_blocks(&mut self, blocks: &[Block]) -> Result<()> {
        for block in blocks {
            let header = BlockHeader::from_block(block);
            self.add_block_header(header)?;
        }

        info!("SPV客户端同步完成: {} 个区块头", blocks.len());
        Ok(())
    }

    /// 计算链的累积难度
    ///
    /// 在实际实现中，这会根据bits字段计算真实的工作量
    pub fn calculate_chain_work(&self) -> u64 {
        self.total_work
    }

    /// 验证区块头链的连续性
    pub fn verify_chain_continuity(&self) -> Result<bool> {
        for i in 1..self.headers.len() {
            if self.headers[i].previous_hash != self.headers[i - 1].hash {
                return Ok(false);
            }

            if self.headers[i].height != self.headers[i - 1].height + 1 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Default for SPVClient {
    fn default() -> Self {
        Self::new()
    }
}

/// SPV客户端统计信息
#[derive(Debug, Clone)]
pub struct SPVStats {
    /// 区块头数量
    pub header_count: usize,

    /// 存储大小（字节）
    pub storage_size: usize,

    /// 已验证交易数
    pub verified_tx_count: usize,

    /// 链高度
    pub chain_height: u32,

    /// 总工作量
    pub total_work: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    fn create_test_header(height: u32, prev_hash: &str) -> BlockHeader {
        BlockHeader {
            height,
            hash: format!("block_{}", height),
            previous_hash: prev_hash.to_string(),
            merkle_root: format!("merkle_{}", height),
            timestamp: 1000000 + height as u64,
            bits: 0,
            nonce: height as u64,
        }
    }

    #[test]
    fn test_spv_client_creation() {
        let client = SPVClient::new();
        assert_eq!(client.get_height(), 0);
        assert!(client.get_tip().is_none());
    }

    #[test]
    fn test_add_block_header() {
        let mut client = SPVClient::new();

        let header = create_test_header(0, "0");
        assert!(client.add_block_header(header).is_ok());

        assert_eq!(client.get_height(), 1);
        assert!(client.get_tip().is_some());
    }

    #[test]
    fn test_block_header_chain() {
        let mut client = SPVClient::new();

        // 添加连续的区块头
        let header1 = create_test_header(0, "0");
        client.add_block_header(header1).unwrap();

        let header2 = create_test_header(1, "block_0");
        client.add_block_header(header2).unwrap();

        let header3 = create_test_header(2, "block_1");
        client.add_block_header(header3).unwrap();

        assert_eq!(client.get_height(), 3);

        // 验证链连续性
        assert!(client.verify_chain_continuity().unwrap());
    }

    #[test]
    fn test_discontinuous_headers_rejected() {
        let mut client = SPVClient::new();

        let header1 = create_test_header(0, "0");
        client.add_block_header(header1).unwrap();

        // 尝试添加不连续的区块头
        let header2 = create_test_header(1, "wrong_hash");
        let result = client.add_block_header(header2);

        assert!(result.is_err());
    }

    #[test]
    fn test_merkle_proof_verification() {
        let mut client = SPVClient::new();

        // 添加区块头
        let header = BlockHeader {
            height: 0,
            hash: "block_0".to_string(),
            previous_hash: "0".to_string(),
            merkle_root: "root_hash".to_string(),
            timestamp: 1000000,
            bits: 0,
            nonce: 0,
        };
        client.add_block_header(header).unwrap();

        // 注意：实际的Merkle证明验证需要真实的proof数据
        // 这里只是测试接口是否正常工作
        let proof = vec!["hash1".to_string(), "hash2".to_string()];
        let result = client.verify_transaction("tx1", &proof, "block_0", 0);

        // 由于merkle_root不匹配，验证会失败，但函数应该正常返回
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage_size_estimation() {
        let mut client = SPVClient::new();

        let header1 = create_test_header(0, "0");
        let header2 = create_test_header(1, "block_0");
        let header3 = create_test_header(2, "block_1");

        client.add_block_header(header1).unwrap();
        client.add_block_header(header2).unwrap();
        client.add_block_header(header3).unwrap();

        // 3个区块头 * 80字节 = 240字节
        assert_eq!(client.estimate_storage_size(), 240);
    }

    #[test]
    fn test_spv_stats() {
        let mut client = SPVClient::new();

        let header1 = create_test_header(0, "0");
        let header2 = create_test_header(1, "block_0");

        client.add_block_header(header1).unwrap();
        client.add_block_header(header2).unwrap();

        let stats = client.get_stats();

        assert_eq!(stats.header_count, 2);
        assert_eq!(stats.chain_height, 2);
        assert_eq!(stats.storage_size, 160); // 2 * 80
    }

    #[test]
    fn test_sync_from_blocks() {
        let mut client = SPVClient::new();

        // 创建测试区块
        let tx = Transaction::new_coinbase("miner".to_string(), 50, 0, 0);
        let block1 = Block::new(0, vec![tx.clone()], "0".to_string());
        let block2 = Block::new(1, vec![tx], block1.hash.clone());

        let blocks = vec![block1, block2];

        assert!(client.sync_from_blocks(&blocks).is_ok());
        assert_eq!(client.get_height(), 2);
    }

    #[test]
    fn test_get_header() {
        let mut client = SPVClient::new();

        let header = create_test_header(0, "0");
        let hash = header.hash.clone();

        client.add_block_header(header).unwrap();

        let retrieved = client.get_header(&hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().height, 0);
    }
}
