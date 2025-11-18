use crate::transaction::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Replace-By-Fee (RBF) 管理器
pub struct RBFManager {
    // 记录可替换的交易
    replaceable_txs: Vec<String>,
}

impl RBFManager {
    pub fn new() -> Self {
        RBFManager {
            replaceable_txs: Vec::new(),
        }
    }

    /// 标记交易为可替换
    pub fn mark_replaceable(&mut self, tx_id: &str) {
        if !self.replaceable_txs.contains(&tx_id.to_string()) {
            self.replaceable_txs.push(tx_id.to_string());
        }
    }

    /// 检查交易是否可替换
    pub fn is_replaceable(&self, tx_id: &str) -> bool {
        self.replaceable_txs.contains(&tx_id.to_string())
    }

    /// 验证替换交易（新交易必须支付更高的手续费）
    pub fn can_replace(&self, old_tx: &Transaction, new_tx: &Transaction) -> Result<(), String> {
        // 1. 检查旧交易是否可替换
        if !self.is_replaceable(&old_tx.id) {
            return Err("原交易不支持RBF".to_string());
        }

        // 2. 检查输入是否相同（必须花费同样的UTXO）
        if old_tx.inputs.len() != new_tx.inputs.len() {
            return Err("输入数量必须相同".to_string());
        }

        for (old_input, new_input) in old_tx.inputs.iter().zip(new_tx.inputs.iter()) {
            if old_input.txid != new_input.txid || old_input.vout != new_input.vout {
                return Err("必须花费相同的UTXO".to_string());
            }
        }

        // 3. 新交易必须支付更高的手续费
        if new_tx.fee <= old_tx.fee {
            return Err(format!(
                "新交易手续费({})必须高于旧交易({})",
                new_tx.fee, old_tx.fee
            ));
        }

        // 4. 手续费增量必须足够（至少增加1 satoshi/byte）
        let fee_increase = new_tx.fee - old_tx.fee;
        let old_size = old_tx.size() as u64;
        let min_increase = old_size; // 简化：至少增加size数量的satoshi

        if fee_increase < min_increase {
            return Err(format!(
                "手续费增量不足，最少需要增加{}",
                min_increase
            ));
        }

        Ok(())
    }

    /// 移除已确认的交易
    pub fn remove_confirmed(&mut self, tx_id: &str) {
        self.replaceable_txs.retain(|id| id != tx_id);
    }
}

impl Default for RBFManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 时间锁交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLock {
    pub locktime: u64,      // Unix时间戳或区块高度
    pub is_block_height: bool, // true: 区块高度, false: 时间戳
}

impl TimeLock {
    /// 创建基于时间的锁
    pub fn new_time_based(timestamp: u64) -> Self {
        TimeLock {
            locktime: timestamp,
            is_block_height: false,
        }
    }

    /// 创建基于区块高度的锁
    pub fn new_height_based(height: u64) -> Self {
        TimeLock {
            locktime: height,
            is_block_height: true,
        }
    }

    /// 检查时间锁是否已过期（可以使用）
    pub fn is_mature(&self, current_time: u64, current_height: u32) -> bool {
        if self.is_block_height {
            current_height as u64 >= self.locktime
        } else {
            current_time >= self.locktime
        }
    }

    /// 获取剩余时间/区块数
    pub fn remaining(&self, current_time: u64, current_height: u32) -> i64 {
        if self.is_block_height {
            self.locktime as i64 - current_height as i64
        } else {
            self.locktime as i64 - current_time as i64
        }
    }
}

/// 高级交易构建器
pub struct AdvancedTxBuilder {
    pub enable_rbf: bool,
    pub timelock: Option<TimeLock>,
    pub sequence: u32,
}

impl AdvancedTxBuilder {
    /// 创建默认构建器
    pub fn new() -> Self {
        AdvancedTxBuilder {
            enable_rbf: false,
            timelock: None,
            sequence: 0xFFFFFFFF, // 默认：不支持RBF和时间锁
        }
    }

    /// 启用 RBF
    pub fn with_rbf(mut self) -> Self {
        self.enable_rbf = true;
        self.sequence = 0xFFFFFFFD; // 序列号 < 0xFFFFFFFE 表示支持RBF
        self
    }

    /// 设置时间锁
    pub fn with_timelock(mut self, timelock: TimeLock) -> Self {
        self.timelock = Some(timelock);
        self.sequence = 0; // 启用时间锁
        self
    }

    /// 获取序列号
    pub fn get_sequence(&self) -> u32 {
        self.sequence
    }

    /// 检查是否支持RBF
    pub fn supports_rbf(&self) -> bool {
        self.sequence < 0xFFFFFFFE
    }
}

impl Default for AdvancedTxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 交易优先级计算器
pub struct TxPriorityCalculator;

impl TxPriorityCalculator {
    /// 计算交易优先级分数
    /// 优先级 = (输入价值 * 输入年龄) / 交易大小
    pub fn calculate_priority(
        input_value: u64,
        input_age: u32, // 确认区块数
        tx_size: usize,
    ) -> f64 {
        if tx_size == 0 {
            return 0.0;
        }

        (input_value as f64 * input_age as f64) / tx_size as f64
    }

    /// 计算费率优先级（sat/byte）
    pub fn calculate_fee_rate(fee: u64, size: usize) -> f64 {
        if size == 0 {
            return 0.0;
        }
        fee as f64 / size as f64
    }

    /// 综合评分（70%费率 + 30%优先级）
    pub fn calculate_score(fee_rate: f64, priority: f64) -> f64 {
        fee_rate * 0.7 + priority * 0.001 * 0.3
    }

    /// 推荐手续费（基于当前内存池状态）
    pub fn recommend_fee(tx_size: usize, urgency: FeeUrgency) -> u64 {
        let sat_per_byte = match urgency {
            FeeUrgency::Low => 1.0,      // 低优先级：1 sat/byte
            FeeUrgency::Medium => 5.0,   // 中优先级：5 sat/byte
            FeeUrgency::High => 20.0,    // 高优先级：20 sat/byte
            FeeUrgency::Urgent => 50.0,  // 紧急：50 sat/byte
        };

        (tx_size as f64 * sat_per_byte) as u64
    }
}

/// 手续费紧急程度
#[derive(Debug, Clone, Copy)]
pub enum FeeUrgency {
    Low,      // 低优先级（几小时内确认）
    Medium,   // 中优先级（30-60分钟）
    High,     // 高优先级（10-20分钟）
    Urgent,   // 紧急（下一个区块）
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbf() {
        let mut rbf = RBFManager::new();
        rbf.mark_replaceable("tx1");
        assert!(rbf.is_replaceable("tx1"));
        assert!(!rbf.is_replaceable("tx2"));
    }

    #[test]
    fn test_timelock() {
        let timelock = TimeLock::new_time_based(1000);
        assert!(timelock.is_mature(1100, 0));
        assert!(!timelock.is_mature(900, 0));

        let block_lock = TimeLock::new_height_based(100);
        assert!(block_lock.is_mature(0, 101));
        assert!(!block_lock.is_mature(0, 99));
    }

    #[test]
    fn test_priority() {
        let fee_rate = TxPriorityCalculator::calculate_fee_rate(500, 100);
        assert_eq!(fee_rate, 5.0);

        let priority = TxPriorityCalculator::calculate_priority(10000, 10, 100);
        assert_eq!(priority, 1000.0);
    }
}
