use crate::block::Block;
use crate::indexer::TransactionIndexer;
use crate::mempool::Mempool;
use crate::parallel_mining::ParallelMiner;
use crate::transaction::{Transaction, TxInput, TxOutput};
use crate::utxo::UTXOSet;
use crate::wallet::Wallet;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// 表示区块链
pub struct Blockchain {
    pub chain: Vec<Block>,           // 区块链（区块列表）
    pub difficulty: usize,           // 挖矿难度
    pub mempool: Mempool,            // 内存池（替代Vec<Transaction>）
    pub utxo_set: UTXOSet,           // UTXO集合
    pub mining_reward: u64,          // 挖矿奖励
    pub indexer: TransactionIndexer, // 交易索引器（加速查询）
    miner: ParallelMiner,            // 并行挖矿器
    pending_spent: HashSet<String>,  // 待确认交易已花费的UTXO ("txid:vout")
}

impl Blockchain {
    /// 创建新的区块链
    pub fn new() -> Blockchain {
        // 使用宽松验证的内存池（Blockchain自身已做UTXO验证）
        let mempool = Mempool::new_permissive();

        let mut blockchain = Blockchain {
            chain: vec![],
            difficulty: 3, // 设置挖矿难度
            mempool,
            utxo_set: UTXOSet::new(),
            mining_reward: 50,                  // 挖矿奖励50 BTC
            indexer: TransactionIndexer::new(), // 初始化索引器
            miner: ParallelMiner::default(),    // 多线程并行挖矿
            pending_spent: HashSet::new(),      // 待确认UTXO追踪
        };

        // 创建创世区块
        let genesis_block = blockchain.create_genesis_block();
        blockchain.indexer.index_block(&genesis_block); // 索引创世区块
        blockchain.chain.push(genesis_block);

        blockchain
    }

    /// 获取创世钱包（确定性，方便演示时花费创世币）
    pub fn genesis_wallet() -> Wallet {
        Wallet::genesis()
    }

    /// 创建创世区块
    fn create_genesis_block(&mut self) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 使用确定性创世钱包地址，保证可被签名花费
        let genesis_wallet = Wallet::genesis();
        let coinbase_tx = Transaction::new_coinbase(
            genesis_wallet.address,
            10_000_000, // 创世区块奖励 10M satoshi
            timestamp,
            0,
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
        fee: u64,
    ) -> Result<Transaction, String> {
        let total_needed = amount + fee;

        // 查找可用的UTXO（排除已被待确认交易花费的UTXO）
        let spendable = self.utxo_set.find_spendable_outputs_excluding(
            &from_wallet.address,
            total_needed,
            &self.pending_spent,
        );

        let (accumulated, utxos) = spendable.ok_or_else(|| "余额不足（包括交易费）".to_string())?;

        // 创建交易输入
        let mut inputs = Vec::new();
        for (txid, vout) in utxos {
            let signature = from_wallet.sign(&format!("{}{}", txid, vout));
            let input = TxInput::new(txid, vout, signature, from_wallet.public_key.clone());
            inputs.push(input);
        }

        // 创建交易输出
        let mut outputs = Vec::new();
        outputs.push(TxOutput::new(amount, to_address));

        // 如果有找零，创建找零输出（扣除费用）
        if accumulated > total_needed {
            outputs.push(TxOutput::new(
                accumulated - total_needed,
                from_wallet.address.clone(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Transaction::new(inputs, outputs, timestamp, fee))
    }

    /// 添加交易到待处理池（内存池）
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // 验证交易ECDSA签名
        if !transaction.verify() {
            return Err("交易验证失败".to_string());
        }

        // 如果不是coinbase交易，验证UTXO和余额
        if !transaction.is_coinbase() {
            let mut input_sum = 0u64;
            for input in &transaction.inputs {
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

            let output_sum: u64 = transaction.outputs.iter().map(|o| o.value).sum();
            if input_sum < output_sum {
                return Err("余额不足，交易无效".to_string());
            }
        }

        // 记录待确认交易消费的UTXO（用于防止连续交易冲突）
        if !transaction.is_coinbase() {
            for input in &transaction.inputs {
                self.pending_spent
                    .insert(format!("{}:{}", input.txid, input.vout));
            }
        }

        // 添加到内存池
        self.mempool
            .add_transaction(transaction)
            .map_err(|e| format!("内存池拒绝: {}", e))
    }

    /// 挖矿 - 从内存池选取交易打包成区块（并行挖矿）
    pub fn mine_pending_transactions(&mut self, miner_address: String) -> Result<(), String> {
        if self.mempool.is_empty() {
            return Err("没有待处理的交易".to_string());
        }

        // 从内存池获取交易（已按费率排序，高优先）
        let pending_txs = self.mempool.get_top_transactions(usize::MAX);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 计算总交易费
        let total_fees: u64 = pending_txs.iter().map(|tx| tx.fee).sum();

        // 创建coinbase交易（挖矿奖励 + 交易费）
        let coinbase_tx =
            Transaction::new_coinbase(miner_address, self.mining_reward, timestamp, total_fees);

        // 创建新区块
        let mut transactions = vec![coinbase_tx];
        transactions.extend(pending_txs.iter().cloned());

        let previous_hash = self
            .chain
            .last()
            .ok_or("blockchain is empty (no genesis block)")?
            .hash
            .clone();
        let mut block = Block::new(self.chain.len() as u32, transactions, previous_hash);

        // 并行挖矿（多线程PoW）
        self.miner
            .mine_block(&mut block, self.difficulty)
            .map_err(|e| format!("挖矿失败: {}", e))?;

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
        self.indexer.index_block(&block);
        self.chain.push(block);

        // 从内存池移除已确认的交易
        for tx in &pending_txs {
            let _ = self.mempool.remove_transaction(&tx.id);
        }

        // 清空待确认UTXO追踪
        self.pending_spent.clear();

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
            if current_block.hash[..self.difficulty] != target {
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
                } else {
                    println!("    交易费: {} satoshi", tx.fee);
                    println!("    费率: {:.2} sat/byte", tx.fee_rate());
                }
                println!("    输入数: {}", tx.inputs.len());
                println!("    输出数: {}", tx.outputs.len());
                for (j, output) in tx.outputs.iter().enumerate() {
                    println!(
                        "      输出 {}: {} -> {}",
                        j, output.value, output.pub_key_hash
                    );
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
