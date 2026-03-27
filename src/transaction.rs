use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 交易输入（TxInput）
///
/// 在比特币系统中，交易输入用于引用之前某个交易的未花费输出（UTXO）。
/// 每个输入必须包含：
/// - 被花费的UTXO的交易ID（txid）
/// - 该交易中输出的索引（vout）
/// - 证明有权花费该UTXO的数字签名（signature）
/// - 用于验证签名的公钥（pub_key）
///
/// 比特币采用UTXO模型而非账户余额模型，所有的币都是以UTXO的形式存在。
/// 要花费比特币，必须消费完整的UTXO，不能部分花费。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub txid: String,      // 被引用的交易ID（32字节哈希）
    pub vout: usize,       // 输出索引（从0开始）
    pub signature: String, // 数字签名（简化版本，实际应使用ECDSA签名）
    pub pub_key: String,   // 公钥（用于验证签名，对应发送者地址）
}

impl TxInput {
    /// 创建新的交易输入
    ///
    /// # 参数
    /// * `txid` - 被引用的交易ID
    /// * `vout` - 该交易中输出的索引号
    /// * `signature` - 使用私钥生成的数字签名
    /// * `pub_key` - 对应的公钥（用于验证签名）
    pub fn new(txid: String, vout: usize, signature: String, pub_key: String) -> Self {
        TxInput {
            txid,
            vout,
            signature,
            pub_key,
        }
    }
}

/// 交易输出（TxOutput）
///
/// 交易输出代表一笔可以被花费的比特币金额（UTXO - Unspent Transaction Output）。
/// 每个输出包含：
/// - 金额（value）：以satoshi为单位（1 BTC = 100,000,000 satoshi）
/// - 锁定脚本（pub_key_hash）：定义谁可以花费这笔输出
///
/// 在实际比特币中，锁定脚本是一段Script脚本，这里简化为接收者的地址。
/// 只有拥有对应私钥的人才能创建有效签名来花费这个输出。
///
/// UTXO一旦被创建，就会一直存在于UTXO集合中，直到被某个交易的输入引用并花费。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: u64,           // 金额（单位：satoshi，1 BTC = 10^8 satoshi）
    pub pub_key_hash: String, // 接收者的公钥哈希（地址），用于锁定输出
}

impl TxOutput {
    /// 创建新的交易输出
    ///
    /// # 参数
    /// * `value` - 输出金额（satoshi）
    /// * `address` - 接收者地址（公钥哈希）
    pub fn new(value: u64, address: String) -> Self {
        TxOutput {
            value,
            pub_key_hash: address,
        }
    }

    /// 检查是否可以被指定的地址解锁
    ///
    /// 在实际比特币中，这会执行锁定脚本验证。
    /// 这里简化为检查地址是否匹配。
    ///
    /// # 参数
    /// * `address` - 要检查的地址
    ///
    /// # 返回值
    /// 如果地址匹配返回true，否则返回false
    pub fn can_be_unlocked_with(&self, address: &str) -> bool {
        self.pub_key_hash == address
    }
}

/// 交易结构（Transaction）
///
/// 比特币交易是价值转移的基本单位。每笔交易包含：
/// - ID：交易的唯一标识符（通过哈希计算得出）
/// - 输入（inputs）：花费哪些UTXO（来源）
/// - 输出（outputs）：创建哪些新UTXO（去向）
/// - 时间戳：交易创建时间
/// - 手续费（fee）：支付给矿工的费用
///
/// 交易规则：
/// 1. 输入总额 = 输出总额 + 手续费
/// 2. 所有输入必须是有效的未花费UTXO
/// 3. 每个输入必须提供有效的签名证明
/// 4. Coinbase交易（挖矿奖励）除外，它没有输入
///
/// 手续费 = 输入总额 - 输出总额
/// 手续费越高，交易被矿工优先打包的概率越大
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,             // 交易ID（通过对交易内容哈希计算得出）
    pub inputs: Vec<TxInput>,   // 输入列表（花费哪些UTXO）
    pub outputs: Vec<TxOutput>, // 输出列表（创建哪些新UTXO）
    pub timestamp: u64,         // Unix时间戳（秒）
    pub fee: u64,               // 交易手续费（satoshi）
}

impl Transaction {
    /// 创建新交易
    ///
    /// 普通交易用于在用户之间转移比特币。
    /// 交易ID通过对整个交易内容进行SHA256哈希计算得出。
    ///
    /// # 参数
    /// * `inputs` - 交易输入列表（要花费的UTXO）
    /// * `outputs` - 交易输出列表（创建的新UTXO）
    /// * `timestamp` - Unix时间戳
    /// * `fee` - 交易手续费（satoshi）
    ///
    /// # 返回值
    /// 返回新创建的交易实例
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>, timestamp: u64, fee: u64) -> Self {
        let mut tx = Transaction {
            id: String::new(),
            inputs,
            outputs,
            timestamp,
            fee,
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 创建Coinbase交易（挖矿奖励 + 交易费用）
    ///
    /// Coinbase交易是每个区块的第一笔交易，用于奖励矿工。
    /// 特点：
    /// - 没有有效的输入（不消费任何UTXO）
    /// - 创造新的比特币（区块奖励）
    /// - 收集区块中所有交易的手续费
    /// - 输入的txid为空字符串，signature包含区块高度（BIP34）
    ///
    /// 比特币的区块奖励每210,000个区块减半一次（约4年）。
    /// 最初是50 BTC，目前（2024年后）是6.25 BTC。
    ///
    /// # 参数
    /// * `to` - 矿工的地址（接收奖励）
    /// * `reward` - 区块奖励（不包括手续费）
    /// * `timestamp` - Unix时间戳
    /// * `total_fees` - 区块中所有交易的手续费总和
    ///
    /// # 返回值
    /// 返回Coinbase交易实例
    pub fn new_coinbase(to: String, reward: u64, timestamp: u64, total_fees: u64) -> Self {
        // 使用计数器确保每个coinbase交易ID唯一（类似BIP34的区块高度编码）
        use std::sync::atomic::{AtomicU64, Ordering};
        static COINBASE_COUNTER: AtomicU64 = AtomicU64::new(0);
        let nonce = COINBASE_COUNTER.fetch_add(1, Ordering::Relaxed);

        let tx_out = TxOutput::new(reward + total_fees, to);
        let tx_in = TxInput {
            txid: String::new(),
            vout: 0,
            signature: format!("coinbase:{}", nonce),
            pub_key: String::from("coinbase"),
        };

        let mut tx = Transaction {
            id: String::new(),
            inputs: vec![tx_in],
            outputs: vec![tx_out],
            timestamp,
            fee: 0, // Coinbase交易没有费用
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 计算交易哈希（交易ID）
    ///
    /// 比特币使用双重SHA256哈希（SHA256(SHA256(data))）来计算交易ID。
    /// 这里简化为单次SHA256。
    ///
    /// 交易哈希包括所有交易数据：输入、输出、时间戳等。
    /// 任何数据的改变都会导致完全不同的哈希值（雪崩效应）。
    ///
    /// # 返回值
    /// 返回64字符的十六进制哈希字符串
    pub fn calculate_hash(&self) -> String {
        let tx_data = serde_json::to_string(&self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 检查是否是Coinbase交易
    ///
    /// Coinbase交易的识别特征：
    /// - 只有一个输入
    /// - 该输入的txid为空（不引用任何之前的交易）
    ///
    /// # 返回值
    /// 如果是Coinbase交易返回true，否则返回false
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].txid.is_empty()
    }

    /// 验证交易的ECDSA数字签名
    ///
    /// 验证流程：
    /// 1. Coinbase交易（挖矿奖励）免验证
    /// 2. 检查是否有输入和输出
    /// 3. 对每个输入，使用secp256k1验证ECDSA签名：
    ///    - 从输入中提取公钥和签名（十六进制编码）
    ///    - 重建签名时的原始数据：`"{txid}{vout}"`
    ///    - 调用secp256k1验证签名的数学正确性
    ///
    /// # 返回值
    /// 如果所有签名都有效返回true，否则返回false
    pub fn verify(&self) -> bool {
        // Coinbase交易总是有效的
        if self.is_coinbase() {
            return true;
        }

        // 检查是否有输入和输出
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        // 使用secp256k1验证每个输入的ECDSA签名
        for input in &self.inputs {
            let signed_data = format!("{}{}", input.txid, input.vout);
            if !Wallet::verify_signature(&input.pub_key, &signed_data, &input.signature) {
                return false;
            }
        }

        true
    }

    /// 计算交易大小（字节）
    ///
    /// 交易大小影响手续费计算。比特币网络中，手续费通常以"satoshi/byte"计算。
    /// 交易包含的输入和输出越多，交易越大，需要的手续费越高。
    ///
    /// 实际比特币交易的大小计算更复杂，需要考虑：
    /// - 版本号（4字节）
    /// - 输入数量（变长整数）
    /// - 每个输入的大小（约148字节，取决于签名类型）
    /// - 输出数量（变长整数）
    /// - 每个输出的大小（约34字节）
    /// - 锁定时间（4字节）
    ///
    /// # 返回值
    /// 返回交易的字节大小
    pub fn size(&self) -> usize {
        serde_json::to_string(self).unwrap_or_default().len()
    }

    /// 计算交易费率（satoshi/byte）
    ///
    /// 费率是矿工选择交易的主要标准。费率越高，交易越可能被快速确认。
    ///
    /// 典型费率参考（会根据网络拥堵情况波动）：
    /// - 低优先级：1-5 sat/byte（可能需要数小时）
    /// - 中优先级：5-20 sat/byte（30-60分钟）
    /// - 高优先级：20-50 sat/byte（10-20分钟）
    /// - 紧急：50+ sat/byte（下一个区块）
    ///
    /// # 返回值
    /// 返回费率（satoshi/byte）
    pub fn fee_rate(&self) -> f64 {
        let size = self.size();
        if size == 0 {
            return 0.0;
        }
        self.fee as f64 / size as f64
    }

    /// 获取交易的输入总额
    ///
    /// 注意：输入的金额信息存储在被引用的UTXO中，而不是输入本身。
    /// 因此需要查询UTXO集合才能获得实际金额。
    /// 这里返回0是占位实现。
    ///
    /// # 返回值
    /// 返回输入总额（需要完整实现）
    pub fn input_sum(&self) -> u64 {
        // 注意：这需要从UTXO集合中查询，这里只是占位
        0
    }

    /// 获取交易的输出总额
    ///
    /// 输出总额 = 所有输出的金额之和
    /// 交易手续费 = 输入总额 - 输出总额
    ///
    /// # 返回值
    /// 返回所有输出的金额总和
    pub fn output_sum(&self) -> u64 {
        self.outputs.iter().map(|o| o.value).sum()
    }
}
