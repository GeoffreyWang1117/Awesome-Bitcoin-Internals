//! 统一的错误处理模块
//!
//! 本模块定义了SimpleBTC中所有可能的错误类型，使用thiserror提供更好的错误信息和错误链追踪。

use thiserror::Error;

/// SimpleBTC的统一错误类型
#[derive(Error, Debug)]
pub enum BitcoinError {
    // ========== 余额和UTXO相关错误 ==========
    /// 余额不足错误
    #[error("余额不足: 需要 {required} satoshi, 实际余额 {actual} satoshi")]
    InsufficientBalance {
        /// 需要的金额
        required: u64,
        /// 实际余额
        actual: u64,
    },

    /// UTXO未找到
    #[error("UTXO未找到: 交易ID {txid}, 输出索引 {vout}")]
    UtxoNotFound {
        /// 交易ID
        txid: String,
        /// 输出索引
        vout: usize,
    },

    /// UTXO已被花费（双花攻击检测）
    #[error("UTXO已被花费: {txid}:{vout}")]
    UtxoAlreadySpent {
        /// 交易ID
        txid: String,
        /// 输出索引
        vout: usize,
    },

    // ========== 交易相关错误 ==========
    /// 无效的交易
    #[error("无效的交易: {reason}")]
    InvalidTransaction {
        /// 失败原因
        reason: String,
    },

    /// 交易签名验证失败
    #[error("交易签名验证失败: 交易 {txid}, 输入索引 {input_index}")]
    InvalidSignature {
        /// 交易ID
        txid: String,
        /// 输入索引
        input_index: usize,
    },

    /// 交易输入输出不匹配
    #[error("交易输入输出不匹配: 输入总额 {inputs} < 输出总额 {outputs}")]
    InputOutputMismatch {
        /// 输入总额
        inputs: u64,
        /// 输出总额
        outputs: u64,
    },

    /// 交易费率过低
    #[error("交易费率过低: {fee_rate} sat/byte < 最低要求 {min_fee_rate} sat/byte")]
    FeeTooLow {
        /// 实际费率
        fee_rate: u64,
        /// 最低费率
        min_fee_rate: u64,
    },

    /// 交易过大
    #[error("交易大小超过限制: {size} bytes > {max_size} bytes")]
    TransactionTooLarge {
        /// 实际大小
        size: usize,
        /// 最大大小
        max_size: usize,
    },

    /// 双花攻击
    #[error("检测到双花攻击: UTXO {txid}:{vout} 已在待确认池中")]
    DoubleSpendDetected {
        /// 交易ID
        txid: String,
        /// 输出索引
        vout: usize,
    },

    /// 双花（通用）
    #[error("双花攻击: UTXO {txid}:{vout} 被重复花费")]
    DoubleSpend {
        /// 交易ID
        txid: String,
        /// 输出索引
        vout: usize,
    },

    /// 交易费用不足
    #[error("交易费用不足: 提供 {provided} satoshi, 需要 {required} satoshi")]
    InsufficientFee {
        /// 提供的费用
        provided: u64,
        /// 需要的费用
        required: u64,
    },

    /// 输入数量过多
    #[error("输入数量过多: {count} > 最大 {max}")]
    TooManyInputs {
        /// 实际数量
        count: usize,
        /// 最大数量
        max: usize,
    },

    /// 输出数量过多
    #[error("输出数量过多: {count} > 最大 {max}")]
    TooManyOutputs {
        /// 实际数量
        count: usize,
        /// 最大数量
        max: usize,
    },

    /// 粉尘输出
    #[error("粉尘输出: 输出#{output_index} 金额 {value} satoshi < 最小 {min_value} satoshi")]
    DustOutput {
        /// 输出金额
        value: u64,
        /// 最小金额
        min_value: u64,
        /// 输出索引
        output_index: usize,
    },

    /// 无效金额
    #[error("无效金额: {reason}")]
    InvalidAmount {
        /// 错误原因
        reason: String,
    },

    // ========== 区块和挖矿相关错误 ==========
    /// 挖矿失败
    #[error("挖矿失败: {reason}")]
    MiningError {
        /// 失败原因
        reason: String,
    },

    /// 无效的区块
    #[error("无效的区块: {reason}")]
    InvalidBlock {
        /// 失败原因
        reason: String,
    },

    /// 区块验证失败
    #[error("区块验证失败: 区块 #{height}, 原因: {reason}")]
    BlockValidationFailed {
        /// 区块高度
        height: usize,
        /// 失败原因
        reason: String,
    },

    /// 工作量证明不足
    #[error("工作量证明不足: 哈希 {hash} 不满足难度 {difficulty}")]
    InsufficientProofOfWork {
        /// 区块哈希
        hash: String,
        /// 难度要求
        difficulty: usize,
    },

    /// 区块链不连续
    #[error("区块链不连续: 区块 #{height} 的 previous_hash 不匹配")]
    ChainDiscontinuity {
        /// 区块高度
        height: usize,
    },

    // ========== 钱包相关错误 ==========
    /// 钱包错误
    #[error("钱包错误: {reason}")]
    WalletError {
        /// 错误原因
        reason: String,
    },

    /// 无效的地址
    #[error("无效的地址: {address}")]
    InvalidAddress {
        /// 地址
        address: String,
    },

    /// 私钥错误
    #[error("私钥错误: {reason}")]
    PrivateKeyError {
        /// 错误原因
        reason: String,
    },

    // ========== 多签相关错误 ==========
    /// 多签配置错误
    #[error("多签配置错误: 需要 {required} 个签名，总共 {total} 个公钥")]
    InvalidMultisigConfig {
        /// 需要的签名数
        required: usize,
        /// 总公钥数
        total: usize,
    },

    /// 签名数量不足
    #[error("签名数量不足: 需要 {required} 个签名，实际 {actual} 个")]
    InsufficientSignatures {
        /// 需要的签名数
        required: usize,
        /// 实际签名数
        actual: usize,
    },

    // ========== RBF相关错误 ==========
    /// RBF错误
    #[error("RBF错误: {reason}")]
    RbfError {
        /// 错误原因
        reason: String,
    },

    /// 交易不可替换
    #[error("交易不可替换: 交易 {txid} 未标记为RBF或已被确认")]
    TransactionNotReplaceable {
        /// 交易ID
        txid: String,
    },

    /// 替换交易费用不足
    #[error("替换交易费用不足: 新费用 {new_fee} <= 原费用 {old_fee}")]
    InsufficientReplacementFee {
        /// 新费用
        new_fee: u64,
        /// 原费用
        old_fee: u64,
    },

    // ========== TimeLock相关错误 ==========
    /// TimeLock错误
    #[error("TimeLock错误: {reason}")]
    TimeLockError {
        /// 错误原因
        reason: String,
    },

    /// 交易尚未到期
    #[error("交易尚未到期: 当前 {current}, 要求 {required}")]
    TransactionLocked {
        /// 当前值（区块高度或时间戳）
        current: u64,
        /// 要求值
        required: u64,
    },

    // ========== 持久化相关错误 ==========
    /// 存储错误
    #[error("存储错误: {reason}")]
    StorageError {
        /// 错误原因
        reason: String,
    },

    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    // ========== 网络相关错误 ==========
    /// 网络错误
    #[error("网络错误: {reason}")]
    NetworkError {
        /// 错误原因
        reason: String,
    },

    // ========== 配置相关错误 ==========
    /// 配置错误
    #[error("配置错误: {reason}")]
    ConfigError {
        /// 错误原因
        reason: String,
    },

    /// TOML解析错误
    #[error("TOML解析错误: {0}")]
    TomlError(#[from] toml::de::Error),

    // ========== 通用错误 ==========
    /// 未实现的功能
    #[error("功能未实现: {feature}")]
    NotImplemented {
        /// 功能名称
        feature: String,
    },

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 其他错误（用于包装外部错误）
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// 简化的Result类型，使用BitcoinError作为错误类型
pub type Result<T> = std::result::Result<T, BitcoinError>;

// ========== 辅助函数 ==========

impl BitcoinError {
    /// 创建一个内部错误
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// 创建一个无效交易错误
    pub fn invalid_transaction(reason: impl Into<String>) -> Self {
        Self::InvalidTransaction {
            reason: reason.into(),
        }
    }

    /// 创建一个余额不足错误
    pub fn insufficient_balance(required: u64, actual: u64) -> Self {
        Self::InsufficientBalance { required, actual }
    }

    /// 创建一个UTXO未找到错误
    pub fn utxo_not_found(txid: impl Into<String>, vout: usize) -> Self {
        Self::UtxoNotFound {
            txid: txid.into(),
            vout,
        }
    }

    /// 创建一个UTXO已被花费错误
    pub fn utxo_already_spent(txid: impl Into<String>, vout: usize) -> Self {
        Self::UtxoAlreadySpent {
            txid: txid.into(),
            vout,
        }
    }

    /// 判断是否是余额不足错误
    pub fn is_insufficient_balance(&self) -> bool {
        matches!(self, Self::InsufficientBalance { .. })
    }

    /// 判断是否是双花攻击
    pub fn is_double_spend(&self) -> bool {
        matches!(
            self,
            Self::DoubleSpendDetected { .. } | Self::UtxoAlreadySpent { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_balance_error() {
        let err = BitcoinError::insufficient_balance(1000, 500);
        assert!(err.is_insufficient_balance());
        assert_eq!(
            err.to_string(),
            "余额不足: 需要 1000 satoshi, 实际余额 500 satoshi"
        );
    }

    #[test]
    fn test_utxo_not_found_error() {
        let err = BitcoinError::utxo_not_found("abc123", 0);
        assert_eq!(err.to_string(), "UTXO未找到: 交易ID abc123, 输出索引 0");
    }

    #[test]
    fn test_double_spend_detection() {
        let err1 = BitcoinError::utxo_already_spent("tx1", 0);
        assert!(err1.is_double_spend());

        let err2 = BitcoinError::DoubleSpendDetected {
            txid: "tx2".to_string(),
            vout: 1,
        };
        assert!(err2.is_double_spend());
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let btc_err: BitcoinError = io_err.into();
        assert!(matches!(btc_err, BitcoinError::IoError(_)));
    }

    #[test]
    fn test_error_from_serde() {
        let json = "{ invalid json }";
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(json);
        let btc_err: BitcoinError = result.unwrap_err().into();
        assert!(matches!(btc_err, BitcoinError::SerializationError(_)));
    }
}
