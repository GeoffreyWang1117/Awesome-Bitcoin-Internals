//! 内存池(Mempool)模块
//!
//! 管理待确认交易的内存池，实现比特币节点的核心功能。
//!
//! # 核心功能
//!
//! 1. **交易存储**: 维护待确认交易集合
//! 2. **优先级排序**: 按费率(sat/byte)排序，高费率优先
//! 3. **双花检测**: 防止同一UTXO被多次花费
//! 4. **RBF支持**: Replace-By-Fee交易替换
//! 5. **容量管理**: 限制内存池大小，淘汰低费率交易
//! 6. **过期清理**: 定期清理过期交易
//!
//! # 比特币内存池特性
//!
//! - **默认大小**: 300 MB
//! - **最小费率**: 1 sat/byte (可动态调整)
//! - **最长保留**: 72小时
//! - **替换规则**: 新交易费用必须更高

use crate::error::{BitcoinError, Result};
use crate::info;
use crate::security::SecurityValidator;
use crate::transaction::Transaction;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// 内存池条目
#[derive(Debug, Clone)]
pub struct MempoolEntry {
    /// 交易
    pub transaction: Transaction,

    /// 加入时间（Unix时间戳）
    pub added_time: u64,

    /// 交易大小（字节）
    pub size: usize,

    /// 费率（sat/byte）
    pub fee_rate: f64,

    /// 是否可被替换（RBF）
    pub replaceable: bool,
}

impl MempoolEntry {
    /// 创建新的内存池条目
    pub fn new(transaction: Transaction, size: usize) -> Self {
        let added_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let fee_rate = if size > 0 {
            transaction.fee as f64 / size as f64
        } else {
            0.0
        };

        // 检查是否标记为可替换（简化版，实际需要检查sequence number）
        let replaceable = transaction.fee > 0;

        Self {
            transaction,
            added_time,
            size,
            fee_rate,
            replaceable,
        }
    }
}

/// 内存池
pub struct Mempool {
    /// 交易池: txid -> MempoolEntry
    transactions: HashMap<String, MempoolEntry>,

    /// 费率索引: fee_rate -> txid集合（用于快速选择高费率交易）
    fee_index: BTreeMap<ordered_float::NotNan<f64>, HashSet<String>>,

    /// UTXO索引: "txid:vout" -> spending_txid
    utxo_index: HashMap<String, String>,

    /// 安全验证器
    validator: SecurityValidator,

    /// 最大内存池大小（字节）
    max_size: usize,

    /// 当前内存池大小（字节）
    current_size: usize,

    /// 最小费率（sat/byte）
    min_fee_rate: f64,

    /// 最长保留时间（秒）
    max_age: u64,
}

impl Mempool {
    /// 创建新的内存池
    ///
    /// # 参数
    /// * `max_size` - 最大大小（字节），默认300MB
    /// * `min_fee_rate` - 最小费率（sat/byte），默认1.0
    pub fn new(max_size: usize, min_fee_rate: f64) -> Self {
        Self {
            transactions: HashMap::new(),
            fee_index: BTreeMap::new(),
            utxo_index: HashMap::new(),
            validator: SecurityValidator::new(),
            max_size,
            current_size: 0,
            min_fee_rate,
            max_age: 72 * 3600, // 72小时
        }
    }

    /// 创建宽松验证的内存池（用于Blockchain集成）
    ///
    /// Blockchain自身已做UTXO/签名验证，内存池仅负责排序和双花检测。
    pub fn new_permissive() -> Self {
        Self {
            transactions: HashMap::new(),
            fee_index: BTreeMap::new(),
            utxo_index: HashMap::new(),
            validator: SecurityValidator::permissive(),
            max_size: 300 * 1024 * 1024,
            current_size: 0,
            min_fee_rate: 0.0, // 允许零费率
            max_age: 72 * 3600,
        }
    }

    /// 添加交易到内存池
    ///
    /// # 验证步骤
    /// 1. 基本安全验证
    /// 2. 双花检测
    /// 3. 费率检查
    /// 4. 容量检查
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        let txid = tx.id.clone();

        // 1. 检查是否已存在
        if self.transactions.contains_key(&txid) {
            return Err(BitcoinError::InvalidTransaction {
                reason: format!("交易 {} 已在内存池中", txid),
            });
        }

        // 2. 基本安全验证
        self.validator.validate_transaction(&tx)?;

        // 3. 双花检测
        for input in &tx.inputs {
            let utxo_key = format!("{}:{}", input.txid, input.vout);
            if let Some(existing_txid) = self.utxo_index.get(&utxo_key).cloned() {
                // 检查是否是RBF替换
                if let Some(existing_entry) = self.transactions.get(&existing_txid) {
                    if existing_entry.replaceable && tx.fee > existing_entry.transaction.fee {
                        // 允许RBF替换
                        info!("RBF替换交易: {} 替换 {}", txid, existing_txid);
                        self.remove_transaction(&existing_txid)?;
                    } else {
                        return Err(BitcoinError::DoubleSpend {
                            txid: input.txid.clone(),
                            vout: input.vout,
                        });
                    }
                }
            }
        }

        // 4. 估算交易大小并计算费率
        let size = self.estimate_tx_size(&tx).max(1); // prevent division by zero
        let fee_rate = tx.fee as f64 / size as f64;

        // 5. 费率检查
        if fee_rate < self.min_fee_rate {
            return Err(BitcoinError::InsufficientFee {
                provided: tx.fee,
                required: (size as f64 * self.min_fee_rate) as u64,
            });
        }

        // 6. 容量检查（如果超过，淘汰低费率交易）
        if self.current_size + size > self.max_size {
            self.evict_low_fee_transactions(size)?;
        }

        // 7. 创建内存池条目
        let entry = MempoolEntry::new(tx.clone(), size);

        // 8. 更新索引
        self.current_size += size;

        // 添加到费率索引
        let fee_key = ordered_float::NotNan::new(fee_rate).unwrap();
        self.fee_index
            .entry(fee_key)
            .or_default()
            .insert(txid.clone());

        // 添加到UTXO索引
        for input in &tx.inputs {
            let utxo_key = format!("{}:{}", input.txid, input.vout);
            self.utxo_index.insert(utxo_key, txid.clone());
        }

        // 添加到交易池
        self.transactions.insert(txid.clone(), entry);

        info!(
            "交易 {} 加入内存池 [费率: {:.2} sat/byte, 大小: {} bytes]",
            txid, fee_rate, size
        );

        Ok(())
    }

    /// 移除交易（已被挖矿确认或过期）
    pub fn remove_transaction(&mut self, txid: &str) -> Result<()> {
        if let Some(entry) = self.transactions.remove(txid) {
            // 更新大小
            self.current_size -= entry.size;

            // 从费率索引移除
            let fee_key = ordered_float::NotNan::new(entry.fee_rate).unwrap();
            if let Some(txids) = self.fee_index.get_mut(&fee_key) {
                txids.remove(txid);
                if txids.is_empty() {
                    self.fee_index.remove(&fee_key);
                }
            }

            // 从UTXO索引移除
            for input in &entry.transaction.inputs {
                let utxo_key = format!("{}:{}", input.txid, input.vout);
                self.utxo_index.remove(&utxo_key);
            }

            info!("交易 {} 从内存池移除", txid);
            Ok(())
        } else {
            Err(BitcoinError::InvalidTransaction {
                reason: format!("交易 {} 不在内存池中", txid),
            })
        }
    }

    /// 获取交易
    pub fn get_transaction(&self, txid: &str) -> Option<&Transaction> {
        self.transactions.get(txid).map(|entry| &entry.transaction)
    }

    /// 按费率获取前N个交易（用于挖矿）
    ///
    /// 返回费率最高的交易，用于打包进区块
    pub fn get_top_transactions(&self, max_count: usize) -> Vec<Transaction> {
        let mut result = Vec::new();

        // 从高费率到低费率遍历
        for (_fee_rate, txids) in self.fee_index.iter().rev() {
            for txid in txids {
                if let Some(entry) = self.transactions.get(txid) {
                    result.push(entry.transaction.clone());
                    if result.len() >= max_count {
                        return result;
                    }
                }
            }
        }

        result
    }

    /// 按总大小获取交易（用于打包区块）
    ///
    /// 返回费率最高且总大小不超过max_size的交易集合
    pub fn get_transactions_for_block(&self, max_size: usize) -> Vec<Transaction> {
        let mut result = Vec::new();
        let mut total_size = 0;

        // 从高费率到低费率遍历
        for (_fee_rate, txids) in self.fee_index.iter().rev() {
            for txid in txids {
                if let Some(entry) = self.transactions.get(txid) {
                    if total_size + entry.size <= max_size {
                        result.push(entry.transaction.clone());
                        total_size += entry.size;
                    }
                }
            }
        }

        info!(
            "为区块选择 {} 笔交易，总大小 {} bytes",
            result.len(),
            total_size
        );
        result
    }

    /// 清理过期交易
    pub fn clear_expired(&mut self) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut expired_txids = Vec::new();

        for (txid, entry) in &self.transactions {
            if now - entry.added_time > self.max_age {
                expired_txids.push(txid.clone());
            }
        }

        let count = expired_txids.len();
        for txid in expired_txids {
            let _ = self.remove_transaction(&txid);
        }

        if count > 0 {
            info!("清理 {} 笔过期交易", count);
        }

        count
    }

    /// 淘汰低费率交易以腾出空间
    fn evict_low_fee_transactions(&mut self, needed_size: usize) -> Result<()> {
        let mut freed_size = 0;
        let mut to_remove = Vec::new();

        // 从低费率到高费率遍历
        for (_fee_rate, txids) in self.fee_index.iter() {
            for txid in txids {
                if let Some(entry) = self.transactions.get(txid) {
                    to_remove.push(txid.clone());
                    freed_size += entry.size;

                    if freed_size >= needed_size {
                        break;
                    }
                }
            }

            if freed_size >= needed_size {
                break;
            }
        }

        if freed_size < needed_size {
            return Err(BitcoinError::Internal(format!(
                "无法腾出足够空间: 需要 {} bytes",
                needed_size
            )));
        }

        for txid in &to_remove {
            self.remove_transaction(txid)?;
        }

        info!(
            "淘汰 {} 笔低费率交易，释放 {} bytes",
            to_remove.len(),
            freed_size
        );
        Ok(())
    }

    /// 估算交易大小
    fn estimate_tx_size(&self, tx: &Transaction) -> usize {
        let base = 10;
        let inputs_size = tx.inputs.len() * 148;
        let outputs_size = tx.outputs.len() * 34;
        base + inputs_size + outputs_size
    }

    /// 获取内存池统计信息
    pub fn get_stats(&self) -> MempoolStats {
        let tx_count = self.transactions.len();
        let total_fees: u64 = self.transactions.values().map(|e| e.transaction.fee).sum();

        let avg_fee_rate = if !self.transactions.is_empty() {
            self.transactions.values().map(|e| e.fee_rate).sum::<f64>() / tx_count as f64
        } else {
            0.0
        };

        MempoolStats {
            tx_count,
            total_size: self.current_size,
            max_size: self.max_size,
            total_fees,
            avg_fee_rate,
            min_fee_rate: self.min_fee_rate,
        }
    }

    /// 获取交易数量
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

/// 内存池统计信息
#[derive(Debug, Clone)]
pub struct MempoolStats {
    /// 交易数量
    pub tx_count: usize,

    /// 当前大小（字节）
    pub total_size: usize,

    /// 最大大小（字节）
    pub max_size: usize,

    /// 总费用
    pub total_fees: u64,

    /// 平均费率
    pub avg_fee_rate: f64,

    /// 最小费率
    pub min_fee_rate: f64,
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new(300 * 1024 * 1024, 1.0) // 300 MB, 1 sat/byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxInput, TxOutput};

    fn create_test_tx(fee: u64, txid_suffix: &str) -> Transaction {
        let inputs = vec![TxInput {
            txid: format!("input_{}", txid_suffix),
            vout: 0,
            signature: "sig".to_string(),
            pub_key: "pubkey".to_string(),
        }];

        let outputs = vec![TxOutput {
            value: 1000,
            pub_key_hash: "addr".to_string(),
        }];

        Transaction::new(inputs, outputs, 0, fee)
    }

    #[test]
    fn test_mempool_basic() {
        let mut mempool = Mempool::default();

        let tx = create_test_tx(200, "1");
        assert!(mempool.add_transaction(tx.clone()).is_ok());

        assert_eq!(mempool.len(), 1);
        assert!(!mempool.is_empty());

        let retrieved = mempool.get_transaction(&tx.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, tx.id);
    }

    #[test]
    fn test_fee_rate_ordering() {
        let mut mempool = Mempool::default();

        // 添加不同费率的交易（费用需要>=192才能满足最小费率要求）
        let tx1 = create_test_tx(200, "1"); // 低费率
        let tx2 = create_test_tx(1000, "2"); // 高费率
        let tx3 = create_test_tx(500, "3"); // 中费率

        mempool.add_transaction(tx1).unwrap();
        mempool.add_transaction(tx2.clone()).unwrap();
        mempool.add_transaction(tx3).unwrap();

        // 获取前1个交易，应该是最高费率的
        let top = mempool.get_top_transactions(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].id, tx2.id); // tx2有最高费率
    }

    #[test]
    fn test_double_spend_detection() {
        let mut mempool = Mempool::default();

        // 创建两笔交易花费同一个UTXO
        let tx1 = create_test_tx(200, "same");
        let tx2 = create_test_tx(200, "same"); // 相同的输入

        assert!(mempool.add_transaction(tx1).is_ok());

        // 第二笔交易应该被拒绝（双花）
        let result = mempool.add_transaction(tx2);
        assert!(result.is_err());
    }

    #[test]
    fn test_rbf_replacement() {
        let mut mempool = Mempool::default();

        // 添加低费率交易
        let tx1 = create_test_tx(200, "rbf");
        mempool.add_transaction(tx1.clone()).unwrap();

        // 添加高费率替换交易（相同输入，更高费用）
        let tx2 = create_test_tx(400, "rbf"); // 更高费用

        assert!(mempool.add_transaction(tx2.clone()).is_ok());

        // 原交易应该被移除
        assert!(mempool.get_transaction(&tx1.id).is_none());

        // 新交易应该在内存池中
        assert!(mempool.get_transaction(&tx2.id).is_some());
    }

    #[test]
    fn test_remove_transaction() {
        let mut mempool = Mempool::default();

        let tx = create_test_tx(200, "remove");
        mempool.add_transaction(tx.clone()).unwrap();

        assert_eq!(mempool.len(), 1);

        assert!(mempool.remove_transaction(&tx.id).is_ok());
        assert_eq!(mempool.len(), 0);
    }

    #[test]
    fn test_get_transactions_for_block() {
        let mut mempool = Mempool::default();

        // 添加多笔交易（费用需要>=192）
        for i in 1..=5 {
            let tx = create_test_tx(200 + i * 100, &format!("block_{}", i));
            mempool.add_transaction(tx).unwrap();
        }

        // 获取用于打包区块的交易（限制大小）
        let txs = mempool.get_transactions_for_block(1000); // 1KB限制

        assert!(!txs.is_empty());
        assert!(txs.len() <= 5);

        // 验证按费率排序（最高费率的应该优先）
        if txs.len() >= 2 {
            assert!(txs[0].fee >= txs[1].fee);
        }
    }

    #[test]
    fn test_mempool_stats() {
        let mut mempool = Mempool::default();

        let tx1 = create_test_tx(200, "stat1");
        let tx2 = create_test_tx(300, "stat2");

        mempool.add_transaction(tx1).unwrap();
        mempool.add_transaction(tx2).unwrap();

        let stats = mempool.get_stats();

        assert_eq!(stats.tx_count, 2);
        assert_eq!(stats.total_fees, 500);
        assert!(stats.avg_fee_rate > 0.0);
        assert!(stats.total_size > 0);
    }
}
