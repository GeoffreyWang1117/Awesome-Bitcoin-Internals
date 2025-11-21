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
//! // 创建区块链
//! let mut blockchain = Blockchain::new();
//!
//! // 创建钱包
//! let alice = Wallet::new();
//! let bob = Wallet::new();
//!
//! // 创建并添加交易
//! let tx = blockchain.create_transaction(&alice, bob.address.clone(), 100, 10)?;
//! blockchain.add_transaction(tx)?;
//!
//! // 挖矿
//! blockchain.mine_pending_transactions(alice.address.clone())?;
//! # Ok::<(), bitcoin_simulation::error::BitcoinError>(())
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
pub mod transaction;
pub mod wallet;
pub mod utxo;

// 高级特性
pub mod merkle;
pub mod multisig;
pub mod advanced_tx;

// 基础设施
pub mod error;
pub mod logging;
pub mod config;
pub mod persistence;
pub mod indexer;

// 重新导出常用类型
pub use error::{BitcoinError, Result};

// 重新导出日志宏（方便使用）
pub use tracing::{debug, error, info, trace, warn, instrument};
