use crate::block::Block;
use crate::transaction::Transaction;
use std::collections::HashMap;

/// 交易索引器 - 加速交易查询
#[derive(Debug, Clone)]
pub struct TransactionIndexer {
    // txid -> block_index
    tx_to_block: HashMap<String, u32>,
    // address -> Vec<txid>
    address_to_txs: HashMap<String, Vec<String>>,
}

impl TransactionIndexer {
    pub fn new() -> Self {
        TransactionIndexer {
            tx_to_block: HashMap::new(),
            address_to_txs: HashMap::new(),
        }
    }

    /// 索引区块链
    pub fn index_blockchain(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.index_block(block);
        }
        println!("✓ 已索引 {} 个区块", blocks.len());
    }

    /// 索引单个区块
    pub fn index_block(&mut self, block: &Block) {
        for tx in &block.transactions {
            // 索引交易所在区块
            self.tx_to_block.insert(tx.id.clone(), block.index);

            // 索引地址相关交易
            for output in &tx.outputs {
                self.address_to_txs
                    .entry(output.pub_key_hash.clone())
                    .or_default()
                    .push(tx.id.clone());
            }
        }
    }

    /// 快速查找交易所在区块
    pub fn find_block_index(&self, txid: &str) -> Option<u32> {
        self.tx_to_block.get(txid).copied()
    }

    /// 查找地址相关的所有交易
    pub fn find_transactions_by_address(&self, address: &str) -> Vec<String> {
        self.address_to_txs
            .get(address)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取索引统计信息
    pub fn stats(&self) -> (usize, usize) {
        (self.tx_to_block.len(), self.address_to_txs.len())
    }
}

impl Default for TransactionIndexer {
    fn default() -> Self {
        Self::new()
    }
}

/// 批量交易处理器
pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        BatchProcessor { batch_size }
    }

    /// 批量验证交易
    pub fn validate_batch(&self, transactions: &[Transaction]) -> Vec<bool> {
        // 可以在这里实现并行验证
        transactions.iter().map(|tx| tx.verify()).collect()
    }

    /// 将交易分批处理
    pub fn chunk_transactions(&self, transactions: Vec<Transaction>) -> Vec<Vec<Transaction>> {
        transactions
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new(100)
    }
}
