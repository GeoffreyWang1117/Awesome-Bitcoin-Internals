//! RocksDB高性能存储层
//!
//! 使用RocksDB替代JSON文件存储，提供：
//! - 高性能键值存储（10-1000倍提升）
//! - 原子性事务支持
//! - 快速查询和索引
//! - 自动压缩和缓存

use crate::block::Block;
use crate::error::{BitcoinError, Result};
use crate::info;
use rocksdb::{BlockBasedOptions, IteratorMode, Options, WriteBatch, DB};

/// RocksDB存储管理器
pub struct RocksDBStorage {
    db: DB,
}

/// 键前缀常量
mod keys {
    pub const BLOCK_PREFIX: &[u8] = b"block:";
    pub const TX_PREFIX: &[u8] = b"tx:";
    pub const UTXO_PREFIX: &[u8] = b"utxo:";
    pub const ADDR_PREFIX: &[u8] = b"addr:";
    pub const CHAIN_HEIGHT: &[u8] = b"meta:height";
    pub const CHAIN_TIP: &[u8] = b"meta:tip";
}

impl RocksDBStorage {
    /// 创建或打开RocksDB存储
    ///
    /// # 参数
    /// * `path` - 数据库路径
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use bitcoin_simulation::storage::RocksDBStorage;
    ///
    /// let storage = RocksDBStorage::new("./data/rocksdb")?;
    /// # Ok::<(), bitcoin_simulation::error::BitcoinError>(())
    /// ```
    pub fn new(path: &str) -> Result<Self> {
        info!("初始化RocksDB存储: {}", path);

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // 性能优化配置
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024);
        opts.set_level_zero_file_num_compaction_trigger(8);
        opts.set_level_zero_slowdown_writes_trigger(17);
        opts.set_level_zero_stop_writes_trigger(24);
        opts.set_max_background_jobs(4);

        // 布隆过滤器优化查询（通过BlockBasedOptions）
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_bloom_filter(10.0, false);
        opts.set_block_based_table_factory(&block_opts);

        let db = DB::open(&opts, path).map_err(|e| BitcoinError::StorageError {
            reason: format!("打开RocksDB失败: {}", e),
        })?;

        Ok(Self { db })
    }

    // ========== 区块操作 ==========

    /// 保存区块
    pub fn put_block(&self, height: u64, block: &Block) -> Result<()> {
        let key = Self::block_key(height);
        let value = serde_json::to_vec(block).map_err(BitcoinError::SerializationError)?;

        self.db
            .put(&key, value)
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("存储区块失败: {}", e),
            })?;

        info!("保存区块 #{}", height);
        Ok(())
    }

    /// 获取区块
    pub fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let key = Self::block_key(height);

        match self.db.get(&key) {
            Ok(Some(bytes)) => {
                let block =
                    serde_json::from_slice(&bytes).map_err(BitcoinError::SerializationError)?;
                Ok(Some(block))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BitcoinError::StorageError {
                reason: format!("读取区块失败: {}", e),
            }),
        }
    }

    /// 删除区块
    pub fn delete_block(&self, height: u64) -> Result<()> {
        let key = Self::block_key(height);
        self.db
            .delete(&key)
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("删除区块失败: {}", e),
            })
    }

    // ========== 交易索引 ==========

    /// 索引交易（交易ID -> 区块高度）
    pub fn index_transaction(&self, txid: &str, block_height: u64) -> Result<()> {
        let key = Self::tx_key(txid);
        let value = block_height.to_be_bytes();

        self.db
            .put(&key, value)
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("索引交易失败: {}", e),
            })
    }

    /// 查找交易所在区块
    pub fn find_transaction_block(&self, txid: &str) -> Result<Option<u64>> {
        let key = Self::tx_key(txid);

        match self.db.get(&key) {
            Ok(Some(bytes)) => {
                if bytes.len() == 8 {
                    let height = u64::from_be_bytes(bytes.try_into().unwrap());
                    Ok(Some(height))
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BitcoinError::StorageError {
                reason: format!("查询交易失败: {}", e),
            }),
        }
    }

    // ========== UTXO集合操作 ==========

    /// 添加UTXO
    pub fn put_utxo(
        &self,
        txid: &str,
        vout: usize,
        output: &crate::transaction::TxOutput,
    ) -> Result<()> {
        let key = Self::utxo_key(txid, vout);
        let value = serde_json::to_vec(output).map_err(BitcoinError::SerializationError)?;

        self.db
            .put(&key, value)
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("存储UTXO失败: {}", e),
            })
    }

    /// 获取UTXO
    pub fn get_utxo(
        &self,
        txid: &str,
        vout: usize,
    ) -> Result<Option<crate::transaction::TxOutput>> {
        let key = Self::utxo_key(txid, vout);

        match self.db.get(&key) {
            Ok(Some(bytes)) => {
                let output =
                    serde_json::from_slice(&bytes).map_err(BitcoinError::SerializationError)?;
                Ok(Some(output))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BitcoinError::StorageError {
                reason: format!("读取UTXO失败: {}", e),
            }),
        }
    }

    /// 删除UTXO（已花费）
    pub fn delete_utxo(&self, txid: &str, vout: usize) -> Result<()> {
        let key = Self::utxo_key(txid, vout);
        self.db
            .delete(&key)
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("删除UTXO失败: {}", e),
            })
    }

    /// 获取地址的所有UTXO
    pub fn get_utxos_by_address(
        &self,
        address: &str,
    ) -> Result<Vec<(String, usize, crate::transaction::TxOutput)>> {
        let mut utxos = Vec::new();

        // 遍历所有UTXO
        let iter = self.db.iterator(IteratorMode::From(
            keys::UTXO_PREFIX,
            rocksdb::Direction::Forward,
        ));

        for item in iter {
            let (key, value) = item.map_err(|e| BitcoinError::StorageError {
                reason: format!("迭代UTXO失败: {}", e),
            })?;

            // 只处理UTXO键
            if !key.starts_with(keys::UTXO_PREFIX) {
                break;
            }

            // 解析输出
            let output: crate::transaction::TxOutput =
                serde_json::from_slice(&value).map_err(BitcoinError::SerializationError)?;

            // 检查地址是否匹配
            if output.pub_key_hash == address {
                // 从键中提取txid和vout
                let key_str = String::from_utf8_lossy(&key[keys::UTXO_PREFIX.len()..]);
                let parts: Vec<&str> = key_str.split(':').collect();
                if parts.len() == 2 {
                    let txid = parts[0].to_string();
                    let vout = parts[1].parse::<usize>().unwrap_or(0);
                    utxos.push((txid, vout, output));
                }
            }
        }

        Ok(utxos)
    }

    // ========== 地址索引 ==========

    /// 索引地址交易
    pub fn index_address(&self, address: &str, txid: &str) -> Result<()> {
        let key = Self::addr_key(address, txid);
        self.db
            .put(&key, b"1")
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("索引地址失败: {}", e),
            })
    }

    /// 获取地址的所有交易
    pub fn get_address_transactions(&self, address: &str) -> Result<Vec<String>> {
        let mut txids = Vec::new();
        let prefix = format!("{}{}", String::from_utf8_lossy(keys::ADDR_PREFIX), address);

        let iter = self.db.iterator(IteratorMode::From(
            prefix.as_bytes(),
            rocksdb::Direction::Forward,
        ));

        for item in iter {
            let (key, _) = item.map_err(|e| BitcoinError::StorageError {
                reason: format!("迭代地址交易失败: {}", e),
            })?;

            let key_str = String::from_utf8_lossy(&key);
            if !key_str.starts_with(&prefix) {
                break;
            }

            // 提取txid
            if let Some(txid) = key_str.split(':').next_back() {
                txids.push(txid.to_string());
            }
        }

        Ok(txids)
    }

    // ========== 元数据操作 ==========

    /// 设置链高度
    pub fn set_chain_height(&self, height: u64) -> Result<()> {
        self.db
            .put(keys::CHAIN_HEIGHT, height.to_be_bytes())
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("设置链高度失败: {}", e),
            })
    }

    /// 获取链高度
    pub fn get_chain_height(&self) -> Result<Option<u64>> {
        match self.db.get(keys::CHAIN_HEIGHT) {
            Ok(Some(bytes)) => {
                if bytes.len() == 8 {
                    let height = u64::from_be_bytes(bytes.try_into().unwrap());
                    Ok(Some(height))
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BitcoinError::StorageError {
                reason: format!("获取链高度失败: {}", e),
            }),
        }
    }

    /// 设置链顶端区块哈希
    pub fn set_chain_tip(&self, hash: &str) -> Result<()> {
        self.db
            .put(keys::CHAIN_TIP, hash.as_bytes())
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("设置链顶端失败: {}", e),
            })
    }

    /// 获取链顶端区块哈希
    pub fn get_chain_tip(&self) -> Result<Option<String>> {
        match self.db.get(keys::CHAIN_TIP) {
            Ok(Some(bytes)) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            Ok(None) => Ok(None),
            Err(e) => Err(BitcoinError::StorageError {
                reason: format!("获取链顶端失败: {}", e),
            }),
        }
    }

    // ========== 批量操作（事务） ==========

    /// 批量写入（原子操作）
    pub fn write_batch(&self, operations: Vec<BatchOperation>) -> Result<()> {
        let mut batch = WriteBatch::default();

        for op in operations {
            match op {
                BatchOperation::PutBlock(height, block) => {
                    let key = Self::block_key(height);
                    let value =
                        serde_json::to_vec(&block).map_err(BitcoinError::SerializationError)?;
                    batch.put(&key, value);
                }
                BatchOperation::PutUtxo(txid, vout, output) => {
                    let key = Self::utxo_key(&txid, vout);
                    let value =
                        serde_json::to_vec(&output).map_err(BitcoinError::SerializationError)?;
                    batch.put(&key, value);
                }
                BatchOperation::DeleteUtxo(txid, vout) => {
                    let key = Self::utxo_key(&txid, vout);
                    batch.delete(&key);
                }
                BatchOperation::IndexTransaction(txid, height) => {
                    let key = Self::tx_key(&txid);
                    batch.put(&key, height.to_be_bytes());
                }
            }
        }

        self.db
            .write(batch)
            .map_err(|e| BitcoinError::StorageError {
                reason: format!("批量写入失败: {}", e),
            })?;

        info!("批量写入完成");
        Ok(())
    }

    // ========== 统计信息 ==========

    /// 获取UTXO集合大小
    pub fn get_utxo_count(&self) -> Result<usize> {
        let mut count = 0;
        let iter = self.db.iterator(IteratorMode::From(
            keys::UTXO_PREFIX,
            rocksdb::Direction::Forward,
        ));

        for item in iter {
            let (key, _) = item.map_err(|e| BitcoinError::StorageError {
                reason: format!("统计UTXO失败: {}", e),
            })?;

            if !key.starts_with(keys::UTXO_PREFIX) {
                break;
            }
            count += 1;
        }

        Ok(count)
    }

    /// 获取数据库统计信息
    pub fn get_stats(&self) -> String {
        self.db
            .property_value("rocksdb.stats")
            .unwrap_or(None)
            .unwrap_or_else(|| "统计信息不可用".to_string())
    }

    // ========== 辅助函数 ==========

    fn block_key(height: u64) -> Vec<u8> {
        let mut key = keys::BLOCK_PREFIX.to_vec();
        key.extend_from_slice(&height.to_be_bytes());
        key
    }

    fn tx_key(txid: &str) -> Vec<u8> {
        let mut key = keys::TX_PREFIX.to_vec();
        key.extend_from_slice(txid.as_bytes());
        key
    }

    fn utxo_key(txid: &str, vout: usize) -> Vec<u8> {
        let mut key = keys::UTXO_PREFIX.to_vec();
        key.extend_from_slice(format!("{}:{}", txid, vout).as_bytes());
        key
    }

    fn addr_key(address: &str, txid: &str) -> Vec<u8> {
        let mut key = keys::ADDR_PREFIX.to_vec();
        key.extend_from_slice(format!("{}:{}", address, txid).as_bytes());
        key
    }
}

/// 批量操作枚举
#[derive(Debug)]
pub enum BatchOperation {
    PutBlock(u64, Block),
    PutUtxo(String, usize, crate::transaction::TxOutput),
    DeleteUtxo(String, usize),
    IndexTransaction(String, u64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (RocksDBStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_creation() {
        let (storage, _temp) = create_test_storage();
        assert!(storage.get_chain_height().unwrap().is_none());
    }

    #[test]
    fn test_chain_height() {
        let (storage, _temp) = create_test_storage();

        storage.set_chain_height(100).unwrap();
        assert_eq!(storage.get_chain_height().unwrap(), Some(100));
    }

    #[test]
    fn test_chain_tip() {
        let (storage, _temp) = create_test_storage();

        let hash = "0000abc123";
        storage.set_chain_tip(hash).unwrap();
        assert_eq!(storage.get_chain_tip().unwrap(), Some(hash.to_string()));
    }

    #[test]
    fn test_transaction_index() {
        let (storage, _temp) = create_test_storage();

        let txid = "tx123";
        storage.index_transaction(txid, 42).unwrap();
        assert_eq!(storage.find_transaction_block(txid).unwrap(), Some(42));
    }

    #[test]
    fn test_utxo_operations() {
        use crate::transaction::TxOutput;
        let (storage, _temp) = create_test_storage();

        let output = TxOutput {
            value: 1000,
            pub_key_hash: "address123".to_string(),
        };

        // 添加UTXO
        storage.put_utxo("tx1", 0, &output).unwrap();

        // 读取UTXO
        let retrieved = storage.get_utxo("tx1", 0).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 1000);

        // 删除UTXO
        storage.delete_utxo("tx1", 0).unwrap();
        assert!(storage.get_utxo("tx1", 0).unwrap().is_none());
    }

    #[test]
    fn test_utxo_count() {
        use crate::transaction::TxOutput;
        let (storage, _temp) = create_test_storage();

        let output = TxOutput {
            value: 1000,
            pub_key_hash: "addr1".to_string(),
        };

        // 添加多个UTXO
        for i in 0..5 {
            storage.put_utxo(&format!("tx{}", i), 0, &output).unwrap();
        }

        assert_eq!(storage.get_utxo_count().unwrap(), 5);
    }
}
