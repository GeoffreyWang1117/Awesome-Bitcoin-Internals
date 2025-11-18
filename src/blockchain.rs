use crate::block::Block;
use crate::transaction::{Transaction, TxInput, TxOutput};
use crate::utxo::UTXOSet;
use crate::wallet::Wallet;
use std::time::{SystemTime, UNIX_EPOCH};

/// 表示区块链
pub struct Blockchain {
    pub chain: Vec<Block>,          // 区块链（区块列表）
    pub difficulty: usize,          // 挖矿难度
    pub pending_transactions: Vec<Transaction>, // 待处理交易池
    pub utxo_set: UTXOSet,          // UTXO集合
    pub mining_reward: u64,         // 挖矿奖励
}

impl Blockchain {
    /// 创建新的区块链
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: vec![],
            difficulty: 3,  // 设置挖矿难度
            pending_transactions: vec![],
            utxo_set: UTXOSet::new(),
            mining_reward: 50,  // 挖矿奖励50 BTC
        };

        // 创建创世区块
        let genesis_block = blockchain.create_genesis_block();
        blockchain.chain.push(genesis_block);

        blockchain
    }

    /// 创建创世区块
    fn create_genesis_block(&mut self) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 创世区块包含一个coinbase交易
        let coinbase_tx = Transaction::new_coinbase(
            "genesis_address".to_string(),
            100,
            timestamp,
        );

        // 将创世交易添加到UTXO集合
        self.utxo_set.add_transaction(&coinbase_tx);

        Block::new(0, vec![coinbase_tx], "0".to_string())
    }

    /// 创建新交易
    pub fn create_transaction(
        &self,
        from_wallet: &Wallet,
        to_address: String,
        amount: u64,
    ) -> Result<Transaction, String> {
        // 查找可用的UTXO
        let spendable = self.utxo_set.find_spendable_outputs(&from_wallet.address, amount);

        if spendable.is_none() {
            return Err("余额不足".to_string());
        }

        let (accumulated, utxos) = spendable.unwrap();

        // 创建交易输入
        let mut inputs = Vec::new();
        for (txid, vout) in utxos {
            let signature = from_wallet.sign(&format!("{}{}", txid, vout));
            let input = TxInput::new(
                txid,
                vout,
                signature,
                from_wallet.public_key.clone(),
            );
            inputs.push(input);
        }

        // 创建交易输出
        let mut outputs = Vec::new();
        outputs.push(TxOutput::new(amount, to_address));

        // 如果有找零，创建找零输出
        if accumulated > amount {
            outputs.push(TxOutput::new(
                accumulated - amount,
                from_wallet.address.clone(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Transaction::new(inputs, outputs, timestamp))
    }

    /// 添加交易到待处理池（事务处理）
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // 验证交易
        if !transaction.verify() {
            return Err("交易验证失败".to_string());
        }

        // 如果不是coinbase交易，验证余额
        if !transaction.is_coinbase() {
            // 计算输入总额
            let mut input_sum = 0u64;
            for input in &transaction.inputs {
                // 查找输入引用的UTXO
                if let Some(outputs) = self.find_transaction_outputs(&input.txid) {
                    if let Some((_, output)) = outputs.iter().find(|(idx, _)| *idx == input.vout) {
                        input_sum += output.value;
                    } else {
                        return Err("UTXO不存在".to_string());
                    }
                } else {
                    return Err("引用的交易不存在".to_string());
                }
            }

            // 计算输出总额
            let output_sum: u64 = transaction.outputs.iter().map(|o| o.value).sum();

            // 验证输入≥输出
            if input_sum < output_sum {
                return Err("余额不足，交易无效".to_string());
            }
        }

        // 将交易添加到待处理池
        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// 挖矿 - 将待处理交易打包成区块
    pub fn mine_pending_transactions(&mut self, miner_address: String) -> Result<(), String> {
        if self.pending_transactions.is_empty() {
            return Err("没有待处理的交易".to_string());
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 创建coinbase交易（挖矿奖励）
        let coinbase_tx = Transaction::new_coinbase(
            miner_address,
            self.mining_reward,
            timestamp,
        );

        // 创建新区块，包含所有待处理交易
        let mut transactions = vec![coinbase_tx];
        transactions.append(&mut self.pending_transactions.clone());

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut block = Block::new(
            self.chain.len() as u32,
            transactions,
            previous_hash,
        );

        // 挖矿
        block.mine_block(self.difficulty);

        // 验证区块
        if !block.validate_transactions() {
            return Err("区块包含无效交易".to_string());
        }

        // 更新UTXO集合（原子操作）
        for tx in &block.transactions {
            if !self.utxo_set.process_transaction(tx) {
                return Err("UTXO更新失败".to_string());
            }
        }

        // 添加区块到链
        self.chain.push(block);

        // 清空待处理交易池
        self.pending_transactions.clear();

        Ok(())
    }

    /// 查找交易的输出
    fn find_transaction_outputs(&self, txid: &str) -> Option<Vec<(usize, TxOutput)>> {
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.id == txid {
                    let mut outputs = Vec::new();
                    for (idx, output) in tx.outputs.iter().enumerate() {
                        outputs.push((idx, output.clone()));
                    }
                    return Some(outputs);
                }
            }
        }
        None
    }

    /// 获取地址余额
    pub fn get_balance(&self, address: &str) -> u64 {
        self.utxo_set.get_balance(address)
    }

    /// 验证区块链完整性
    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // 验证当前区块的哈希
            if current_block.hash != current_block.calculate_hash() {
                println!("区块 {} 哈希无效", i);
                return false;
            }

            // 验证前一个区块的哈希引用
            if current_block.previous_hash != previous_block.hash {
                println!("区块 {} 的前向引用无效", i);
                return false;
            }

            // 验证工作量证明
            let target = "0".repeat(self.difficulty);
            if &current_block.hash[..self.difficulty] != target {
                println!("区块 {} 工作量证明无效", i);
                return false;
            }

            // 验证交易
            if !current_block.validate_transactions() {
                println!("区块 {} 包含无效交易", i);
                return false;
            }
        }

        true
    }

    /// 打印区块链信息
    pub fn print_chain(&self) {
        println!("\n========== 区块链信息 ==========");
        for block in &self.chain {
            println!("\n--- 区块 #{} ---", block.index);
            println!("时间戳: {}", block.timestamp);
            println!("哈希: {}", block.hash);
            println!("前一个哈希: {}", block.previous_hash);
            println!("Nonce: {}", block.nonce);
            println!("交易数量: {}", block.transactions.len());
            for (i, tx) in block.transactions.iter().enumerate() {
                println!("  交易 #{}: {}", i, tx.id);
                if tx.is_coinbase() {
                    println!("    类型: Coinbase（挖矿奖励）");
                }
                println!("    输入数: {}", tx.inputs.len());
                println!("    输出数: {}", tx.outputs.len());
                for (j, output) in tx.outputs.iter().enumerate() {
                    println!("      输出 {}: {} -> {}", j, output.value, output.pub_key_hash);
                }
            }
        }
        println!("\n================================\n");
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}
