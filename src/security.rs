//! 安全验证模块
//!
//! 提供区块链安全相关的验证功能：
//! - 双花检测（Double-Spend Detection）
//! - 交易大小限制（防止拒绝服务攻击）
//! - 费率验证（防止粉尘攻击）
//! - 输入输出验证
//!
//! # 比特币安全机制
//!
//! 1. **双花检测**：确保同一UTXO不能被花费两次
//! 2. **交易大小**：限制单笔交易最大100KB（区块限制1MB）
//! 3. **最小费率**：至少1 sat/byte（防止网络拥堵）
//! 4. **输出粉尘**：最小输出546 satoshi（防止UTXO膨胀）

use crate::error::{BitcoinError, Result};
use crate::info;
use crate::transaction::Transaction;
use std::collections::HashSet;

/// 安全验证器
pub struct SecurityValidator {
    /// 最大交易大小（字节）
    max_tx_size: usize,

    /// 最小费率（sat/byte）
    min_fee_rate: u64,

    /// 最小输出金额（防止粉尘）
    min_output_value: u64,

    /// 最大输入数量
    max_inputs: usize,

    /// 最大输出数量
    max_outputs: usize,
}

impl SecurityValidator {
    /// 创建默认验证器（比特币标准）
    pub fn new() -> Self {
        Self {
            max_tx_size: 100_000,  // 100 KB（比特币标准交易限制）
            min_fee_rate: 1,       // 1 sat/byte（最低费率）
            min_output_value: 546, // 546 satoshi（粉尘阈值）
            max_inputs: 10_000,    // 最大输入数
            max_outputs: 10_000,   // 最大输出数
        }
    }

    /// 创建宽松验证器（用于测试）
    pub fn permissive() -> Self {
        Self {
            max_tx_size: 1_000_000, // 1 MB
            min_fee_rate: 0,        // 允许零费率
            min_output_value: 1,    // 允许1 satoshi
            max_inputs: 100_000,
            max_outputs: 100_000,
        }
    }

    /// 验证交易安全性
    ///
    /// # 检查项
    /// - 交易大小限制
    /// - 费率验证
    /// - 输入输出数量
    /// - 粉尘输出检测
    /// - 输入输出金额验证
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // 1. 验证交易大小
        self.validate_size(tx)?;

        // 2. 验证费率
        self.validate_fee_rate(tx)?;

        // 3. 验证输入输出数量
        self.validate_io_counts(tx)?;

        // 4. 验证输出金额（粉尘检测）
        self.validate_output_values(tx)?;

        // 5. 验证输入输出总额
        self.validate_amounts(tx)?;

        info!("交易 {} 通过安全验证", tx.id);
        Ok(())
    }

    /// 验证交易大小
    fn validate_size(&self, tx: &Transaction) -> Result<()> {
        let size = self.estimate_tx_size(tx);

        if size > self.max_tx_size {
            return Err(BitcoinError::TransactionTooLarge {
                size,
                max_size: self.max_tx_size,
            });
        }

        Ok(())
    }

    /// 估算交易大小（字节）
    ///
    /// 粗略估算：
    /// - 基础：10 bytes
    /// - 每个输入：~148 bytes（P2PKH）
    /// - 每个输出：~34 bytes（P2PKH）
    fn estimate_tx_size(&self, tx: &Transaction) -> usize {
        let base = 10;
        let inputs_size = tx.inputs.len() * 148;
        let outputs_size = tx.outputs.len() * 34;
        base + inputs_size + outputs_size
    }

    /// 验证费率
    fn validate_fee_rate(&self, tx: &Transaction) -> Result<()> {
        if tx.is_coinbase() {
            return Ok(()); // Coinbase交易不需要费率验证
        }

        let size = self.estimate_tx_size(tx);
        let fee_rate = tx.fee as f64 / size as f64;

        if fee_rate < self.min_fee_rate as f64 {
            return Err(BitcoinError::InsufficientFee {
                provided: tx.fee,
                required: (size as u64 * self.min_fee_rate),
            });
        }

        Ok(())
    }

    /// 验证输入输出数量
    fn validate_io_counts(&self, tx: &Transaction) -> Result<()> {
        if tx.inputs.len() > self.max_inputs {
            return Err(BitcoinError::TooManyInputs {
                count: tx.inputs.len(),
                max: self.max_inputs,
            });
        }

        if tx.outputs.len() > self.max_outputs {
            return Err(BitcoinError::TooManyOutputs {
                count: tx.outputs.len(),
                max: self.max_outputs,
            });
        }

        Ok(())
    }

    /// 验证输出金额（粉尘检测）
    fn validate_output_values(&self, tx: &Transaction) -> Result<()> {
        for (idx, output) in tx.outputs.iter().enumerate() {
            if output.value < self.min_output_value {
                return Err(BitcoinError::DustOutput {
                    value: output.value,
                    min_value: self.min_output_value,
                    output_index: idx,
                });
            }
        }

        Ok(())
    }

    /// 验证输入输出总额
    fn validate_amounts(&self, tx: &Transaction) -> Result<()> {
        if tx.is_coinbase() {
            return Ok(()); // Coinbase交易特殊处理
        }

        // 输入总额必须 >= 输出总额 + 费用
        let total_output: u64 = tx.outputs.iter().map(|o| o.value).sum();

        // 注意：这里假设输入已经验证过
        // 实际实现中，需要从UTXO集合中查询输入金额
        if total_output == 0 {
            return Err(BitcoinError::InvalidAmount {
                reason: "输出总额为0".to_string(),
            });
        }

        Ok(())
    }

    /// 检测双花攻击
    ///
    /// 验证交易列表中没有重复花费同一个UTXO
    ///
    /// # 参数
    /// * `transactions` - 待验证的交易列表
    ///
    /// # 返回
    /// 如果检测到双花返回Err，否则返回Ok
    pub fn detect_double_spend(&self, transactions: &[Transaction]) -> Result<()> {
        let mut spent_utxos = HashSet::new();

        for tx in transactions {
            if tx.is_coinbase() {
                continue; // Coinbase交易没有输入
            }

            for input in &tx.inputs {
                let utxo_key = format!("{}:{}", input.txid, input.vout);

                if spent_utxos.contains(&utxo_key) {
                    return Err(BitcoinError::DoubleSpend {
                        txid: input.txid.clone(),
                        vout: input.vout,
                    });
                }

                spent_utxos.insert(utxo_key);
            }
        }

        info!("双花检测通过，验证了 {} 笔交易", transactions.len());
        Ok(())
    }

    /// 批量验证交易
    ///
    /// 验证一组交易的安全性，并检测双花
    pub fn validate_transactions(&self, transactions: &[Transaction]) -> Result<()> {
        // 1. 逐个验证交易
        for tx in transactions {
            self.validate_transaction(tx)?;
        }

        // 2. 检测双花
        self.detect_double_spend(transactions)?;

        Ok(())
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxInput, TxOutput};

    fn create_test_tx() -> Transaction {
        let inputs = vec![TxInput {
            txid: "prev_tx".to_string(),
            vout: 0,
            signature: "sig".to_string(),
            pub_key: "pubkey".to_string(),
        }];

        let outputs = vec![TxOutput {
            value: 1000,
            pub_key_hash: "addr".to_string(),
        }];

        // 估算交易大小约192字节，最小费率1 sat/byte，所以费用应该>=192
        Transaction::new(inputs, outputs, 0, 200)
    }

    #[test]
    fn test_validate_transaction_success() {
        let validator = SecurityValidator::new();
        let tx = create_test_tx();
        assert!(validator.validate_transaction(&tx).is_ok());
    }

    #[test]
    fn test_dust_output_rejection() {
        let validator = SecurityValidator::new();

        let inputs = vec![TxInput {
            txid: "prev_tx".to_string(),
            vout: 0,
            signature: "sig".to_string(),
            pub_key: "pubkey".to_string(),
        }];

        // 粉尘输出（低于546 satoshi）
        let outputs = vec![TxOutput {
            value: 100, // 低于粉尘阈值
            pub_key_hash: "addr".to_string(),
        }];

        let tx = Transaction::new(inputs, outputs, 0, 200);
        assert!(validator.validate_transaction(&tx).is_err());
    }

    #[test]
    fn test_double_spend_detection() {
        let validator = SecurityValidator::new();

        // 创建两笔交易花费同一个UTXO
        let tx1 = create_test_tx();
        let tx2 = create_test_tx(); // 相同的输入！

        let result = validator.detect_double_spend(&[tx1, tx2]);
        assert!(result.is_err());

        if let Err(BitcoinError::DoubleSpend { txid, vout }) = result {
            assert_eq!(txid, "prev_tx");
            assert_eq!(vout, 0);
        } else {
            panic!("应该检测到双花");
        }
    }

    #[test]
    fn test_no_double_spend() {
        let validator = SecurityValidator::new();

        // 创建两笔交易花费不同UTXO
        let inputs1 = vec![TxInput {
            txid: "tx1".to_string(),
            vout: 0,
            signature: "sig".to_string(),
            pub_key: "pubkey".to_string(),
        }];

        let inputs2 = vec![TxInput {
            txid: "tx2".to_string(), // 不同的txid
            vout: 0,
            signature: "sig".to_string(),
            pub_key: "pubkey".to_string(),
        }];

        let outputs = vec![TxOutput {
            value: 1000,
            pub_key_hash: "addr".to_string(),
        }];

        let tx1 = Transaction::new(inputs1, outputs.clone(), 0, 200);
        let tx2 = Transaction::new(inputs2, outputs, 0, 200);

        assert!(validator.detect_double_spend(&[tx1, tx2]).is_ok());
    }

    #[test]
    fn test_transaction_size_limit() {
        let validator = SecurityValidator::new();

        // 创建超大交易（超过100KB）
        let mut inputs = Vec::new();
        for i in 0..1000 {
            inputs.push(TxInput {
                txid: format!("tx{}", i),
                vout: 0,
                signature: "a".repeat(100), // 大签名
                pub_key: "pubkey".to_string(),
            });
        }

        let outputs = vec![TxOutput {
            value: 1000,
            pub_key_hash: "addr".to_string(),
        }];

        let tx = Transaction::new(inputs, outputs, 0, 200_000); // 大费用
        let result = validator.validate_transaction(&tx);

        assert!(result.is_err());
        if let Err(BitcoinError::TransactionTooLarge { size, max_size }) = result {
            assert!(size > max_size);
        } else {
            panic!("应该拒绝超大交易");
        }
    }

    #[test]
    fn test_insufficient_fee_rate() {
        let mut validator = SecurityValidator::new();
        validator.min_fee_rate = 10; // 提高最小费率

        // 创建低费率交易
        let tx = create_test_tx(); // fee=10
        let result = validator.validate_transaction(&tx);

        assert!(result.is_err());
    }

    #[test]
    fn test_permissive_validator() {
        let validator = SecurityValidator::permissive();

        // 粉尘输出（在宽松模式下允许）
        let inputs = vec![TxInput {
            txid: "prev_tx".to_string(),
            vout: 0,
            signature: "sig".to_string(),
            pub_key: "pubkey".to_string(),
        }];

        let outputs = vec![TxOutput {
            value: 1, // 仅1 satoshi
            pub_key_hash: "addr".to_string(),
        }];

        let tx = Transaction::new(inputs, outputs, 0, 0); // 零费用
        assert!(validator.validate_transaction(&tx).is_ok());
    }
}
