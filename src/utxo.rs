use crate::transaction::{Transaction, TxOutput};
use std::collections::HashMap;

/// UTXO集合 - 管理所有未花费的交易输出
#[derive(Debug, Clone)]
pub struct UTXOSet {
    // key: txid, value: (vout_index, TxOutput)
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
}

impl UTXOSet {
    /// 创建新的UTXO集合
    pub fn new() -> Self {
        UTXOSet {
            utxos: HashMap::new(),
        }
    }

    /// 添加交易的输出到UTXO集合
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
    pub fn remove_utxo(&mut self, txid: &str, vout: usize) {
        if let Some(outputs) = self.utxos.get_mut(txid) {
            outputs.retain(|(index, _)| *index != vout);
            if outputs.is_empty() {
                self.utxos.remove(txid);
            }
        }
    }

    /// 查找指定地址的所有UTXO
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
    pub fn find_spendable_outputs(&self, address: &str, amount: u64) -> Option<(u64, Vec<(String, usize)>)> {
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

    /// 获取指定地址的余额
    pub fn get_balance(&self, address: &str) -> u64 {
        let utxos = self.find_utxos(address);
        utxos.iter().map(|(_, _, value)| value).sum()
    }

    /// 处理交易（移除输入，添加输出）
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
