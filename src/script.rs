//! Bitcoin脚本系统
//!
//! 实现简化版的Bitcoin Script，用于交易验证和智能合约。
//!
//! # Bitcoin Script特性
//!
//! - **基于栈**: 所有操作在栈上进行
//! - **图灵不完备**: 没有循环，保证执行终止
//! - **简单但强大**: 支持多重签名、时间锁等高级功能
//!
//! # 标准脚本类型
//!
//! 1. **P2PKH** (Pay-to-Public-Key-Hash): 最常见
//!    - scriptPubKey: `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`
//!    - scriptSig: `<sig> <pubKey>`
//!
//! 2. **P2SH** (Pay-to-Script-Hash): 多签等
//!    - scriptPubKey: `OP_HASH160 <scriptHash> OP_EQUAL`
//!    - scriptSig: `<sig> ... <redeemScript>`
//!
//! 3. **P2PK** (Pay-to-Public-Key): 早期使用
//!    - scriptPubKey: `<pubKey> OP_CHECKSIG`
//!    - scriptSig: `<sig>`
//!
//! # 示例
//!
//! ```no_run
//! use bitcoin_simulation::script::{Script, OpCode};
//!
//! // 创建P2PKH锁定脚本
//! let script_pubkey = Script::p2pkh("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
//!
//! // 创建解锁脚本
//! let script_sig = Script::new(vec![
//!     OpCode::PushData("signature".to_string()),
//!     OpCode::PushData("public_key".to_string()),
//! ]);
//!
//! // 验证脚本
//! let result = Script::verify(&script_sig, &script_pubkey, "tx_hash");
//! # Ok::<(), bitcoin_simulation::error::BitcoinError>(())
//! ```

use crate::error::{BitcoinError, Result};
use crate::info;
use sha2::{Digest, Sha256};
use std::fmt;

/// 脚本操作码
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    // ===== 常量 =====
    /// 空字节数组
    Op0,

    /// 推送数据到栈
    PushData(String),

    // ===== 栈操作 =====
    /// 复制栈顶元素
    OpDup,

    /// 删除栈顶元素
    OpDrop,

    /// 交换栈顶两个元素
    OpSwap,

    // ===== 加密操作 =====
    /// SHA256哈希
    OpSha256,

    /// RIPEMD160(SHA256(x))
    OpHash160,

    // ===== 比较操作 =====
    /// 比较栈顶两个元素是否相等
    OpEqual,

    /// OpEqual + OpVerify (相等则继续，否则失败)
    OpEqualVerify,

    // ===== 验证操作 =====
    /// 验证栈顶为true，否则失败
    OpVerify,

    /// 验证签名
    OpCheckSig,

    /// OpCheckSig + OpVerify
    OpCheckSigVerify,

    /// 多重签名验证
    OpCheckMultiSig,

    // ===== 数值操作 =====
    /// 加法
    OpAdd,

    /// 减法
    OpSub,

    // ===== 控制流 =====
    /// 条件执行
    OpIf,
    OpElse,
    OpEndIf,

    /// 返回true
    OpTrue,

    /// 返回false
    OpFalse,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Op0 => write!(f, "OP_0"),
            OpCode::PushData(data) => write!(f, "<{}>", data),
            OpCode::OpDup => write!(f, "OP_DUP"),
            OpCode::OpDrop => write!(f, "OP_DROP"),
            OpCode::OpSwap => write!(f, "OP_SWAP"),
            OpCode::OpSha256 => write!(f, "OP_SHA256"),
            OpCode::OpHash160 => write!(f, "OP_HASH160"),
            OpCode::OpEqual => write!(f, "OP_EQUAL"),
            OpCode::OpEqualVerify => write!(f, "OP_EQUALVERIFY"),
            OpCode::OpVerify => write!(f, "OP_VERIFY"),
            OpCode::OpCheckSig => write!(f, "OP_CHECKSIG"),
            OpCode::OpCheckSigVerify => write!(f, "OP_CHECKSIGVERIFY"),
            OpCode::OpCheckMultiSig => write!(f, "OP_CHECKMULTISIG"),
            OpCode::OpAdd => write!(f, "OP_ADD"),
            OpCode::OpSub => write!(f, "OP_SUB"),
            OpCode::OpIf => write!(f, "OP_IF"),
            OpCode::OpElse => write!(f, "OP_ELSE"),
            OpCode::OpEndIf => write!(f, "OP_ENDIF"),
            OpCode::OpTrue => write!(f, "OP_TRUE"),
            OpCode::OpFalse => write!(f, "OP_FALSE"),
        }
    }
}

/// 脚本
#[derive(Debug, Clone)]
pub struct Script {
    /// 操作码序列
    pub ops: Vec<OpCode>,
}

impl Script {
    /// 创建新脚本
    pub fn new(ops: Vec<OpCode>) -> Self {
        Self { ops }
    }

    /// 创建空脚本
    pub fn empty() -> Self {
        Self { ops: vec![] }
    }

    /// 创建P2PKH锁定脚本
    ///
    /// `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`
    pub fn p2pkh(pub_key_hash: &str) -> Self {
        Self::new(vec![
            OpCode::OpDup,
            OpCode::OpHash160,
            OpCode::PushData(pub_key_hash.to_string()),
            OpCode::OpEqualVerify,
            OpCode::OpCheckSig,
        ])
    }

    /// 创建P2PK锁定脚本
    ///
    /// `<pubKey> OP_CHECKSIG`
    pub fn p2pk(pub_key: &str) -> Self {
        Self::new(vec![
            OpCode::PushData(pub_key.to_string()),
            OpCode::OpCheckSig,
        ])
    }

    /// 创建多重签名脚本 (M-of-N)
    ///
    /// 例如 2-of-3: `2 <pubKey1> <pubKey2> <pubKey3> 3 OP_CHECKMULTISIG`
    pub fn multisig(m: usize, pub_keys: Vec<String>) -> Self {
        let n = pub_keys.len();
        let mut ops = vec![OpCode::PushData(m.to_string())];

        for key in pub_keys {
            ops.push(OpCode::PushData(key));
        }

        ops.push(OpCode::PushData(n.to_string()));
        ops.push(OpCode::OpCheckMultiSig);

        Self::new(ops)
    }

    /// 验证脚本
    ///
    /// # 参数
    /// * `script_sig` - 解锁脚本（输入）
    /// * `script_pubkey` - 锁定脚本（输出）
    /// * `tx_hash` - 交易哈希（用于签名验证）
    pub fn verify(script_sig: &Script, script_pubkey: &Script, tx_hash: &str) -> Result<bool> {
        // 创建执行引擎
        let mut engine = ScriptEngine::new(tx_hash.to_string());

        // 1. 执行解锁脚本（scriptSig）
        info!("执行解锁脚本: {}", script_sig);
        engine.execute(&script_sig.ops)?;

        // 2. 执行锁定脚本（scriptPubKey）
        info!("执行锁定脚本: {}", script_pubkey);
        engine.execute(&script_pubkey.ops)?;

        // 3. 检查最终栈状态
        if engine.stack.is_empty() {
            return Ok(false);
        }

        // 栈顶必须为true
        let result = engine.stack.last().unwrap();
        Ok(result == "1" || result == "true")
    }

    /// 序列化脚本为字符串
    pub fn to_string(&self) -> String {
        self.ops.iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// 脚本执行引擎
struct ScriptEngine {
    /// 数据栈
    stack: Vec<String>,

    /// 交易哈希（用于签名验证）
    tx_hash: String,

    /// 是否在条件分支中
    in_if_branch: bool,

    /// 条件分支的值
    if_condition: bool,
}

impl ScriptEngine {
    /// 创建新的执行引擎
    fn new(tx_hash: String) -> Self {
        Self {
            stack: Vec::new(),
            tx_hash,
            in_if_branch: false,
            if_condition: false,
        }
    }

    /// 执行操作码序列
    fn execute(&mut self, ops: &[OpCode]) -> Result<()> {
        for op in ops {
            self.execute_op(op)?;
        }
        Ok(())
    }

    /// 执行单个操作码
    fn execute_op(&mut self, op: &OpCode) -> Result<()> {
        match op {
            OpCode::Op0 => self.stack.push("0".to_string()),

            OpCode::PushData(data) => self.stack.push(data.clone()),

            OpCode::OpDup => {
                let top = self.pop()?;
                self.stack.push(top.clone());
                self.stack.push(top);
            }

            OpCode::OpDrop => {
                self.pop()?;
            }

            OpCode::OpSwap => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.stack.push(a);
                self.stack.push(b);
            }

            OpCode::OpSha256 => {
                let data = self.pop()?;
                let hash = self.sha256(&data);
                self.stack.push(hash);
            }

            OpCode::OpHash160 => {
                let data = self.pop()?;
                let hash = self.hash160(&data);
                self.stack.push(hash);
            }

            OpCode::OpEqual => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.stack.push(if a == b { "1" } else { "0" }.to_string());
            }

            OpCode::OpEqualVerify => {
                let a = self.pop()?;
                let b = self.pop()?;
                if a != b {
                    return Err(BitcoinError::InvalidTransaction {
                        reason: format!("OP_EQUALVERIFY 失败: {} != {}", a, b),
                    });
                }
            }

            OpCode::OpVerify => {
                let value = self.pop()?;
                if value != "1" && value != "true" {
                    return Err(BitcoinError::InvalidTransaction {
                        reason: "OP_VERIFY 失败".to_string(),
                    });
                }
            }

            OpCode::OpCheckSig => {
                let pub_key = self.pop()?;
                let signature = self.pop()?;

                // 简化的签名验证（实际应使用ECDSA）
                let valid = self.verify_signature(&signature, &pub_key)?;
                self.stack.push(if valid { "1" } else { "0" }.to_string());
            }

            OpCode::OpCheckSigVerify => {
                let pub_key = self.pop()?;
                let signature = self.pop()?;

                let valid = self.verify_signature(&signature, &pub_key)?;
                if !valid {
                    return Err(BitcoinError::InvalidSignature {
                        txid: self.tx_hash.clone(),
                        input_index: 0,
                    });
                }
            }

            OpCode::OpCheckMultiSig => {
                // 简化的多重签名验证
                let n = self.pop()?.parse::<usize>().unwrap_or(0);
                let mut pub_keys = Vec::new();
                for _ in 0..n {
                    pub_keys.push(self.pop()?);
                }

                let m = self.pop()?.parse::<usize>().unwrap_or(0);
                let mut signatures = Vec::new();
                for _ in 0..m {
                    signatures.push(self.pop()?);
                }

                // 验证至少m个签名有效
                let mut valid_count = 0;
                for sig in &signatures {
                    for key in &pub_keys {
                        if self.verify_signature(sig, key)? {
                            valid_count += 1;
                            break;
                        }
                    }
                }

                self.stack.push(if valid_count >= m { "1" } else { "0" }.to_string());
            }

            OpCode::OpAdd => {
                let a = self.pop()?.parse::<i64>().unwrap_or(0);
                let b = self.pop()?.parse::<i64>().unwrap_or(0);
                self.stack.push((a + b).to_string());
            }

            OpCode::OpSub => {
                let b = self.pop()?.parse::<i64>().unwrap_or(0);
                let a = self.pop()?.parse::<i64>().unwrap_or(0);
                self.stack.push((a - b).to_string());
            }

            OpCode::OpIf => {
                let condition = self.pop()?;
                self.in_if_branch = true;
                self.if_condition = condition == "1" || condition == "true";
            }

            OpCode::OpElse => {
                self.if_condition = !self.if_condition;
            }

            OpCode::OpEndIf => {
                self.in_if_branch = false;
            }

            OpCode::OpTrue => self.stack.push("1".to_string()),

            OpCode::OpFalse => self.stack.push("0".to_string()),
        }

        Ok(())
    }

    /// 从栈中弹出元素
    fn pop(&mut self) -> Result<String> {
        self.stack.pop().ok_or_else(|| {
            BitcoinError::InvalidTransaction {
                reason: "脚本栈为空".to_string(),
            }
        })
    }

    /// SHA256哈希
    fn sha256(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// HASH160 (RIPEMD160(SHA256(x)))
    fn hash160(&self, data: &str) -> String {
        // 简化版：只做SHA256
        // 实际应该: RIPEMD160(SHA256(data))
        use ripemd::{Ripemd160, Digest as RipemdDigest};

        let sha256_hash = {
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            hasher.finalize()
        };

        let mut hasher = Ripemd160::new();
        hasher.update(&sha256_hash);
        format!("{:x}", hasher.finalize())
    }

    /// 验证签名（简化版）
    ///
    /// 实际应使用ECDSA验证
    fn verify_signature(&self, _signature: &str, _pub_key: &str) -> Result<bool> {
        // 简化版：总是返回true
        // 实际实现应该使用secp256k1进行ECDSA验证
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut engine = ScriptEngine::new("test_tx".to_string());

        // 测试PUSH和DUP
        engine.execute_op(&OpCode::PushData("hello".to_string())).unwrap();
        engine.execute_op(&OpCode::OpDup).unwrap();

        assert_eq!(engine.stack.len(), 2);
        assert_eq!(engine.stack[0], "hello");
        assert_eq!(engine.stack[1], "hello");
    }

    #[test]
    fn test_hash_operations() {
        let mut engine = ScriptEngine::new("test_tx".to_string());

        // 测试SHA256
        engine.execute_op(&OpCode::PushData("test".to_string())).unwrap();
        engine.execute_op(&OpCode::OpSha256).unwrap();

        assert_eq!(engine.stack.len(), 1);
        assert!(!engine.stack[0].is_empty());
    }

    #[test]
    fn test_comparison_operations() {
        let mut engine = ScriptEngine::new("test_tx".to_string());

        // 测试EQUAL
        engine.execute_op(&OpCode::PushData("a".to_string())).unwrap();
        engine.execute_op(&OpCode::PushData("a".to_string())).unwrap();
        engine.execute_op(&OpCode::OpEqual).unwrap();

        assert_eq!(engine.stack.last().unwrap(), "1");
    }

    #[test]
    fn test_p2pkh_script() {
        let pub_key_hash = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

        // 创建P2PKH脚本
        let script_pubkey = Script::p2pkh(pub_key_hash);

        assert_eq!(script_pubkey.ops.len(), 5);
        assert!(matches!(script_pubkey.ops[0], OpCode::OpDup));
        assert!(matches!(script_pubkey.ops[4], OpCode::OpCheckSig));
    }

    #[test]
    fn test_p2pk_script() {
        let pub_key = "02abc123";

        // 创建P2PK脚本
        let script_pubkey = Script::p2pk(pub_key);

        assert_eq!(script_pubkey.ops.len(), 2);
        assert!(matches!(script_pubkey.ops[1], OpCode::OpCheckSig));
    }

    #[test]
    fn test_multisig_script() {
        let pub_keys = vec![
            "key1".to_string(),
            "key2".to_string(),
            "key3".to_string(),
        ];

        // 创建2-of-3多签脚本
        let script = Script::multisig(2, pub_keys);

        assert!(matches!(script.ops.last().unwrap(), OpCode::OpCheckMultiSig));
    }

    #[test]
    fn test_script_verification_success() {
        // 创建简单的解锁/锁定脚本
        let script_sig = Script::new(vec![
            OpCode::PushData("sig".to_string()),
            OpCode::PushData("pubkey".to_string()),
        ]);

        let script_pubkey = Script::new(vec![
            OpCode::OpCheckSig,
        ]);

        let result = Script::verify(&script_sig, &script_pubkey, "tx_hash").unwrap();
        assert!(result);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut engine = ScriptEngine::new("test_tx".to_string());

        // 测试加法: 2 + 3 = 5
        engine.execute_op(&OpCode::PushData("2".to_string())).unwrap();
        engine.execute_op(&OpCode::PushData("3".to_string())).unwrap();
        engine.execute_op(&OpCode::OpAdd).unwrap();

        assert_eq!(engine.stack.last().unwrap(), "5");
    }
}
