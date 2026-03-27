//! SimpleBTC - 比特币区块链教育项目
//!
//! SimpleBTC是一个功能完整的比特币区块链实现，用于学习和教育。
//! 它实现了比特币的核心特性，包括UTXO模型、工作量证明、多重签名等。
//!
//! # 快速开始
//!
//! ```no_run
//! use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};
//!
//! // 创建区块链（包含创世区块）
//! let mut blockchain = Blockchain::new();
//!
//! // 使用创世钱包（有预置资金）和新钱包
//! let genesis = Blockchain::genesis_wallet();
//! let bob = Wallet::new();
//!
//! // 创建交易（真实ECDSA签名）
//! let tx = blockchain.create_transaction(&genesis, bob.address.clone(), 100, 10)?;
//! blockchain.add_transaction(tx)?;
//!
//! // 挖矿
//! blockchain.mine_pending_transactions(bob.address.clone())?;
//! # Ok::<(), String>(())
//! ```
//!
//! # 模块说明
//!
//! - [`block`] - 区块结构和工作量证明
//! - [`blockchain`] - 区块链核心逻辑
//! - [`transaction`] - 交易处理
//! - [`wallet`] - 钱包和密钥管理
//! - [`utxo`] - UTXO集合管理
//! - [`merkle`] - Merkle树实现
//! - [`multisig`] - 多重签名
//! - [`advanced_tx`] - 高级交易特性（RBF、TimeLock）
//! - [`error`] - 错误处理

// 核心模块
pub mod block;
pub mod blockchain;
pub mod crypto;
pub mod transaction;
pub mod utxo;
pub mod wallet; // 真实的ECDSA密码学

// 高级特性
pub mod advanced_tx;
pub mod mempool; // 内存池
pub mod merkle;
pub mod multisig;
pub mod network;
pub mod parallel_mining; // 多线程并行挖矿
pub mod script; // Bitcoin脚本系统
pub mod spv; // SPV轻客户端 // P2P网络层

// 基础设施
pub mod config;
pub mod error;
pub mod indexer;
pub mod logging;
pub mod persistence;
pub mod security;
pub mod storage; // RocksDB高性能存储 // 安全验证

// 重新导出常用类型
pub use error::{BitcoinError, Result};

// 重新导出日志宏（方便使用）
pub use tracing::{debug, error, info, instrument, trace, warn};
