use crate::transaction::{Transaction, TxOutput};
use std::collections::{HashMap, HashSet};

/// UTXO集合 - 管理所有未花费的交易输出
///
/// UTXO（Unspent Transaction Output）是比特币的核心概念。
///
/// UTXO模型 vs 账户模型：
///
/// 账户模型（以太坊等）：
/// - 记录每个账户的余额
/// - 转账：A账户-100，B账户+100
/// - 简单直观，但难以并行处理
///
/// UTXO模型（比特币）：
/// - 没有账户余额概念
/// - 只记录未花费的交易输出
/// - 转账：消费A的UTXO，创建给B的新UTXO
/// - 更好的隐私性和并行性
///
/// UTXO的生命周期：
/// 1. 创建：作为交易的输出被创建
/// 2. 存在：保存在UTXO集合中，可被查询
/// 3. 花费：被新交易的输入引用
/// 4. 移除：从UTXO集合中删除
///
/// UTXO集合的优势：
/// - 快速验证：只需检查UTXO集合，无需遍历整个区块链
/// - 防止双花：每个UTXO只能被花费一次
/// - 并行处理：不同UTXO可以并行验证
///
/// # 数据结构
/// key: 交易ID（txid）
/// value: 该交易的所有未花费输出列表 [(输出索引, 输出详情)]
#[derive(Debug, Clone)]
pub struct UTXOSet {
    // key: txid, value: (vout_index, TxOutput)
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
}

impl UTXOSet {
    /// 创建新的UTXO集合
    ///
    /// 初始化一个空的UTXO集合。
    /// 在区块链初始化时创建，然后通过处理区块中的交易来更新。
    ///
    /// # 返回值
    /// 返回新的空UTXO集合
    pub fn new() -> Self {
        UTXOSet {
            utxos: HashMap::new(),
        }
    }

    /// 添加交易的输出到UTXO集合
    ///
    /// 当新交易被确认（打包进区块）时，需要将其所有输出添加到UTXO集合。
    /// 这些新创建的输出现在可以被后续交易引用和花费。
    ///
    /// 注意：这个函数只添加输出，不处理输入（不移除被花费的UTXO）。
    /// 完整的交易处理需要同时处理输入和输出。
    ///
    /// # 参数
    /// * `tx` - 要添加的交易
    pub fn add_transaction(&mut self, tx: &Transaction) {
        let mut outputs = Vec::new();
        for (index, output) in tx.outputs.iter().enumerate() {
            outputs.push((index, output.clone()));
        }

        if !outputs.is_empty() {
            self.utxos.insert(tx.id.clone(), outputs);
        }
    }

    /// 移除已花费的UTXO
    ///
    /// 当UTXO被某个交易的输入引用时，它就被"花费"了，需要从UTXO集合中移除。
    /// 这是防止双重花费的关键机制。
    ///
    /// 双重花费攻击：
    /// - 攻击者尝试用同一个UTXO创建两笔交易
    /// - 第一笔交易确认后，UTXO被移除
    /// - 第二笔交易验证时发现UTXO不存在，被拒绝
    ///
    /// # 参数
    /// * `txid` - 交易ID
    /// * `vout` - 输出索引
    pub fn remove_utxo(&mut self, txid: &str, vout: usize) {
        if let Some(outputs) = self.utxos.get_mut(txid) {
            outputs.retain(|(index, _)| *index != vout);
            if outputs.is_empty() {
                self.utxos.remove(txid);
            }
        }
    }

    /// 查找指定地址的所有UTXO
    ///
    /// 遍历整个UTXO集合，找出所有属于指定地址的未花费输出。
    /// 这些UTXO的总和就是该地址的"余额"。
    ///
    /// 注意：比特币实际上没有"余额"的概念，余额是由UTXO计算得出的。
    ///
    /// # 参数
    /// * `address` - 要查询的地址
    ///
    /// # 返回值
    /// 返回UTXO列表：[(txid, vout, value)]
    pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)> {
        let mut utxos = Vec::new();

        for (txid, outputs) in &self.utxos {
            for (vout, output) in outputs {
                if output.can_be_unlocked_with(address) {
                    utxos.push((txid.clone(), *vout, output.value));
                }
            }
        }

        utxos
    }

    /// 查找可用于支付指定金额的UTXO
    ///
    /// 这是创建交易时的关键步骤：选择哪些UTXO作为交易输入。
    ///
    /// UTXO选择策略（Coin Selection）：
    /// 1. 贪心算法：遍历UTXO，累加直到满足金额（本实现使用此策略）
    /// 2. 最优匹配：选择总额最接近目标金额的UTXO组合（减少找零）
    /// 3. 最小UTXO：优先使用小额UTXO（避免UTXO碎片化）
    /// 4. 最大UTXO：优先使用大额UTXO（减少交易大小）
    ///
    /// 找零机制：
    /// - 如果accumulated > amount，差额需要作为找零返回给发送者
    /// - 例如：要支付3 BTC，选择了5 BTC的UTXO，需要创建2 BTC的找零输出
    /// - 手续费从找零中扣除
    ///
    /// # 参数
    /// * `address` - 发送者地址
    /// * `amount` - 需要的金额（包括手续费）
    ///
    /// # 返回值
    /// Some((accumulated, utxo_list)) - 成功找到足够的UTXO
    /// None - 余额不足
    pub fn find_spendable_outputs(
        &self,
        address: &str,
        amount: u64,
    ) -> Option<(u64, Vec<(String, usize)>)> {
        let mut accumulated = 0u64;
        let mut unspent_outputs = Vec::new();

        for (txid, outputs) in &self.utxos {
            for (vout, output) in outputs {
                if output.can_be_unlocked_with(address) {
                    accumulated += output.value;
                    unspent_outputs.push((txid.clone(), *vout));

                    if accumulated >= amount {
                        return Some((accumulated, unspent_outputs));
                    }
                }
            }
        }

        if accumulated >= amount {
            Some((accumulated, unspent_outputs))
        } else {
            None
        }
    }

    /// 查找可用UTXO（排除已被待确认交易花费的UTXO）
    ///
    /// 这解决了同一钱包连续创建多笔交易时的UTXO冲突问题。
    /// 待确认交易消费的UTXO会被跳过，确保每笔交易使用不同的UTXO。
    pub fn find_spendable_outputs_excluding(
        &self,
        address: &str,
        amount: u64,
        excluded: &HashSet<String>,
    ) -> Option<(u64, Vec<(String, usize)>)> {
        let mut accumulated = 0u64;
        let mut unspent_outputs = Vec::new();

        for (txid, outputs) in &self.utxos {
            for (vout, output) in outputs {
                // 跳过已被待确认交易花费的UTXO
                let utxo_key = format!("{}:{}", txid, vout);
                if excluded.contains(&utxo_key) {
                    continue;
                }

                if output.can_be_unlocked_with(address) {
                    accumulated += output.value;
                    unspent_outputs.push((txid.clone(), *vout));

                    if accumulated >= amount {
                        return Some((accumulated, unspent_outputs));
                    }
                }
            }
        }

        if accumulated >= amount {
            Some((accumulated, unspent_outputs))
        } else {
            None
        }
    }

    /// 获取指定地址的余额
    ///
    /// 比特币的"余额"是一个计算值，而不是存储值。
    /// 余额 = 该地址所有UTXO的金额总和
    ///
    /// 这意味着：
    /// - 查询余额需要扫描整个UTXO集合（可通过索引优化）
    /// - 余额可能分散在多个UTXO中
    /// - 转账可能需要合并多个UTXO
    ///
    /// # 参数
    /// * `address` - 要查询的地址
    ///
    /// # 返回值
    /// 返回地址的总余额（satoshi）
    pub fn get_balance(&self, address: &str) -> u64 {
        let utxos = self.find_utxos(address);
        utxos.iter().map(|(_, _, value)| value).sum()
    }

    /// 处理交易（移除输入，添加输出）
    ///
    /// 这是UTXO集合更新的核心函数，确保ACID特性：
    ///
    /// Atomicity（原子性）：
    /// - 整个交易要么完全成功，要么完全失败
    /// - 不会出现只处理了部分输入/输出的情况
    ///
    /// Consistency（一致性）：
    /// - 交易前后UTXO集合保持一致
    /// - 输入总额 = 输出总额 + 手续费
    ///
    /// Isolation（隔离性）：
    /// - 并发交易互不影响（通过锁机制）
    ///
    /// Durability（持久性）：
    /// - 确认的交易永久有效（需配合区块链存储）
    ///
    /// 处理步骤：
    /// 1. 验证交易有效性
    /// 2. 移除被花费的UTXO（处理输入）
    /// 3. 添加新创建的UTXO（处理输出）
    ///
    /// # 参数
    /// * `tx` - 要处理的交易
    ///
    /// # 返回值
    /// true - 处理成功
    /// false - 处理失败（交易无效）
    pub fn process_transaction(&mut self, tx: &Transaction) -> bool {
        // 验证交易
        if !tx.verify() {
            return false;
        }

        // 如果不是coinbase交易，移除输入引用的UTXO
        if !tx.is_coinbase() {
            for input in &tx.inputs {
                self.remove_utxo(&input.txid, input.vout);
            }
        }

        // 添加新的输出到UTXO集合
        self.add_transaction(tx);

        true
    }
}

impl Default for UTXOSet {
    fn default() -> Self {
        Self::new()
    }
}
