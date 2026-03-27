use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

/// Replace-By-Fee (RBF) 管理器
///
/// RBF（替换手续费）是BIP125提出的机制，允许用户替换未确认的交易。
///
/// 使用场景：
///
/// 1. 加速确认：
///    - 初始交易使用较低手续费（1 sat/byte）
///    - 网络拥堵导致长时间未确认
///    - 发送新交易提高手续费（10 sat/byte）
///    - 矿工优先打包高费率交易
///
/// 2. 取消交易：
///    - 发送给错误地址
///    - 通过RBF发送给自己，使原交易失效
///
/// 3. 批量支付优化：
///    - 初始交易包含部分收款人
///    - 后续通过RBF添加更多收款人
///    - 节省总手续费
///
/// 技术要点：
/// - 标记：交易的sequence字段 < 0xFFFFFFFE
/// - 规则：新交易必须支付更高的手续费
/// - 限制：新交易必须花费相同的UTXO
/// - 费率：手续费增量至少1 sat/byte
///
/// 安全考虑：
/// - 零确认交易风险：商家不应接受标记为RBF的零确认交易
/// - 双花攻击：攻击者可能用RBF将支付改为发给自己
/// - 建议：重要交易等待至少1个确认（10分钟）
///
/// 实际应用：
/// - 比特币核心钱包支持RBF
/// - Electrum钱包支持RBF
/// - 交易所通常使用RBF加速提现
pub struct RBFManager {
    // 记录可替换的交易ID列表
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
            return Err(format!("手续费增量不足，最少需要增加{}", min_increase));
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

/// 时间锁（TimeLock / nLockTime）
///
/// 时间锁是比特币的重要特性，限制交易在特定时间或区块高度之前不能被确认。
///
/// 两种类型：
///
/// 1. 基于时间（Unix时间戳）：
///    - locktime >= 500,000,000
///    - 示例：locktime = 1735689600 (2025-01-01 00:00:00)
///    - 用途：定期存款、工资发放、债券到期
///
/// 2. 基于区块高度：
///    - locktime < 500,000,000
///    - 示例：locktime = 800000（第80万个区块）
///    - 用途：更精确的时间控制（10分钟/区块）
///
/// 应用场景：
///
/// 1. 定期存款：
///    - 锁定3个月/6个月/1年
///    - 强制储蓄，防止冲动消费
///    - 到期自动解锁
///
/// 2. 遗产继承：
///    - 1年后自动转给继承人
///    - 本人活跃则可提前取消
///    - 防止意外情况导致资金丢失
///
/// 3. 智能合约：
///    - 与多签结合：时间到期前需要2-of-2，之后变为1-of-2
///    - 闪电网络：HTLC（哈希时间锁定合约）
///    - 原子交易：跨链原子互换
///
/// 4. 工资发放：
///    - 月底自动解锁
///    - 防止提前挪用资金
///
/// 5. 项目资金锁定：
///    - ICO代币锁定期
///    - 团队份额分批解锁
///    - 增强投资者信心
///
/// 技术实现：
/// - nLockTime字段（4字节）在交易中
/// - nSequence字段必须 < 0xFFFFFFFF才能启用
/// - 矿工在验证时检查时间锁
/// - 未到期的交易不会被打包进区块
///
/// 与CheckLockTimeVerify (CLTV)的区别：
/// - nLockTime：整个交易的锁定
/// - CLTV：单个UTXO的锁定（BIP65，更灵活）
///
/// 安全性：
/// - 时间锁不可撤销（一旦广播）
/// - 可以通过RBF取消（如果启用）
/// - 区块时间可能有约2小时的误差
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLock {
    pub locktime: u64,         // Unix时间戳（秒）或区块高度
    pub is_block_height: bool, // true: 基于区块高度, false: 基于时间戳
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
            FeeUrgency::Low => 1.0,     // 低优先级：1 sat/byte
            FeeUrgency::Medium => 5.0,  // 中优先级：5 sat/byte
            FeeUrgency::High => 20.0,   // 高优先级：20 sat/byte
            FeeUrgency::Urgent => 50.0, // 紧急：50 sat/byte
        };

        (tx_size as f64 * sat_per_byte) as u64
    }
}

/// 手续费紧急程度
#[derive(Debug, Clone, Copy)]
pub enum FeeUrgency {
    Low,    // 低优先级（几小时内确认）
    Medium, // 中优先级（30-60分钟）
    High,   // 高优先级（10-20分钟）
    Urgent, // 紧急（下一个区块）
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
