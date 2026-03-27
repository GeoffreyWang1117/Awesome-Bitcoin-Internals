//! 配置管理模块
//!
//! 本模块提供SimpleBTC的所有配置项，支持从文件、环境变量和代码中加载配置。

use crate::error::{BitcoinError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// SimpleBTC完整配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// 区块链配置
    pub blockchain: BlockchainConfig,

    /// 网络配置
    pub network: NetworkConfig,

    /// 存储配置
    pub storage: StorageConfig,

    /// API配置
    pub api: ApiConfig,

    /// 日志配置
    pub logging: LoggingConfig,
}

/// 区块链配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// 挖矿难度（前导零个数，1-10）
    #[serde(default = "default_difficulty")]
    pub difficulty: usize,

    /// 区块奖励（satoshi）
    #[serde(default = "default_mining_reward")]
    pub mining_reward: u64,

    /// 最小交易费（satoshi）
    #[serde(default = "default_min_tx_fee")]
    pub min_transaction_fee: u64,

    /// 区块最大交易数
    #[serde(default = "default_max_txs_per_block")]
    pub max_transactions_per_block: usize,

    /// 区块最大大小（bytes）
    #[serde(default = "default_max_block_size")]
    pub max_block_size: usize,

    /// 最小费率（satoshi/byte）
    #[serde(default = "default_min_fee_rate")]
    pub min_fee_rate: u64,

    /// 是否启用RBF（Replace-By-Fee）
    #[serde(default = "default_true")]
    pub enable_rbf: bool,

    /// 是否启用TimeLock
    #[serde(default = "default_true")]
    pub enable_timelock: bool,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P监听端口
    #[serde(default = "default_p2p_port")]
    pub p2p_port: u16,

    /// 最大连接节点数
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,

    /// 连接超时（秒）
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// 是否启用节点发现
    #[serde(default = "default_true")]
    pub enable_discovery: bool,

    /// 引导节点列表
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 数据目录
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// 区块链数据文件名
    #[serde(default = "default_blockchain_file")]
    pub blockchain_file: String,

    /// 钱包目录
    #[serde(default = "default_wallet_dir")]
    pub wallet_dir: String,

    /// 是否启用自动备份
    #[serde(default = "default_true")]
    pub enable_auto_backup: bool,

    /// 备份间隔（秒）
    #[serde(default = "default_backup_interval")]
    pub backup_interval: u64,
}

/// API服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API监听地址
    #[serde(default = "default_api_host")]
    pub host: String,

    /// API监听端口
    #[serde(default = "default_api_port")]
    pub port: u16,

    /// 是否启用CORS
    #[serde(default = "default_true")]
    pub enable_cors: bool,

    /// 允许的源（空表示允许所有）
    #[serde(default)]
    pub allowed_origins: Vec<String>,

    /// 请求超时（秒）
    #[serde(default = "default_api_timeout")]
    pub request_timeout: u64,

    /// 是否启用速率限制
    #[serde(default = "default_false")]
    pub enable_rate_limit: bool,

    /// 速率限制（请求/分钟）
    #[serde(default = "default_rate_limit")]
    pub rate_limit: usize,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别：trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// 日志格式：pretty, compact, json
    #[serde(default = "default_log_format")]
    pub format: String,

    /// 是否显示代码位置
    #[serde(default = "default_true")]
    pub show_location: bool,

    /// 是否显示线程ID
    #[serde(default = "default_false")]
    pub show_thread_id: bool,

    /// 日志文件路径（空表示只输出到控制台）
    #[serde(default)]
    pub file_path: Option<String>,
}

// ========== 默认值函数 ==========

fn default_difficulty() -> usize {
    4
}
fn default_mining_reward() -> u64 {
    5000
}
fn default_min_tx_fee() -> u64 {
    1
}
fn default_max_txs_per_block() -> usize {
    100
}
fn default_max_block_size() -> usize {
    1_000_000
} // 1MB
fn default_min_fee_rate() -> u64 {
    1
} // 1 sat/byte

fn default_p2p_port() -> u16 {
    8333
}
fn default_max_peers() -> usize {
    8
}
fn default_connection_timeout() -> u64 {
    30
}

fn default_data_dir() -> String {
    "./data".to_string()
}
fn default_blockchain_file() -> String {
    "blockchain.json".to_string()
}
fn default_wallet_dir() -> String {
    "wallets".to_string()
}
fn default_backup_interval() -> u64 {
    3600
} // 1小时

fn default_api_host() -> String {
    "127.0.0.1".to_string()
}
fn default_api_port() -> u16 {
    3000
}
fn default_api_timeout() -> u64 {
    30
}
fn default_rate_limit() -> usize {
    100
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

// ========== 实现 ==========

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            difficulty: default_difficulty(),
            mining_reward: default_mining_reward(),
            min_transaction_fee: default_min_tx_fee(),
            max_transactions_per_block: default_max_txs_per_block(),
            max_block_size: default_max_block_size(),
            min_fee_rate: default_min_fee_rate(),
            enable_rbf: true,
            enable_timelock: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            p2p_port: default_p2p_port(),
            max_peers: default_max_peers(),
            connection_timeout: default_connection_timeout(),
            enable_discovery: true,
            bootstrap_nodes: vec![],
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            blockchain_file: default_blockchain_file(),
            wallet_dir: default_wallet_dir(),
            enable_auto_backup: true,
            backup_interval: default_backup_interval(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_api_host(),
            port: default_api_port(),
            enable_cors: true,
            allowed_origins: vec![],
            request_timeout: default_api_timeout(),
            enable_rate_limit: false,
            rate_limit: default_rate_limit(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            show_location: true,
            show_thread_id: false,
            file_path: None,
        }
    }
}

impl Config {
    /// 从TOML文件加载配置
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use bitcoin_simulation::config::Config;
    ///
    /// let config = Config::from_file("config.toml")?;
    /// # Ok::<(), bitcoin_simulation::error::BitcoinError>(())
    /// ```
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| BitcoinError::ConfigError {
                reason: format!("读取配置文件失败: {}", e),
            })?;

        let config: Config = toml::from_str(&content).map_err(|e| BitcoinError::ConfigError {
            reason: format!("解析配置文件失败: {}", e),
        })?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 保存配置到TOML文件
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use bitcoin_simulation::config::Config;
    ///
    /// let config = Config::default();
    /// config.save_to_file("config.toml")?;
    /// # Ok::<(), bitcoin_simulation::error::BitcoinError>(())
    /// ```
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| BitcoinError::ConfigError {
            reason: format!("序列化配置失败: {}", e),
        })?;

        std::fs::write(path.as_ref(), content).map_err(|e| BitcoinError::ConfigError {
            reason: format!("写入配置文件失败: {}", e),
        })?;

        Ok(())
    }

    /// 验证配置的有效性
    fn validate(&self) -> Result<()> {
        // 验证难度
        if self.blockchain.difficulty == 0 || self.blockchain.difficulty > 10 {
            return Err(BitcoinError::ConfigError {
                reason: format!(
                    "挖矿难度必须在1-10之间，当前值: {}",
                    self.blockchain.difficulty
                ),
            });
        }

        // 验证端口
        if self.network.p2p_port == 0 {
            return Err(BitcoinError::ConfigError {
                reason: "P2P端口不能为0".to_string(),
            });
        }

        if self.api.port == 0 {
            return Err(BitcoinError::ConfigError {
                reason: "API端口不能为0".to_string(),
            });
        }

        // 验证数据目录
        if self.storage.data_dir.is_empty() {
            return Err(BitcoinError::ConfigError {
                reason: "数据目录不能为空".to_string(),
            });
        }

        Ok(())
    }

    /// 获取完整的数据目录路径
    pub fn get_data_dir(&self) -> &str {
        &self.storage.data_dir
    }

    /// 获取区块链文件的完整路径
    pub fn get_blockchain_path(&self) -> String {
        format!("{}/{}", self.storage.data_dir, self.storage.blockchain_file)
    }

    /// 获取钱包目录的完整路径
    pub fn get_wallet_dir(&self) -> String {
        format!("{}/{}", self.storage.data_dir, self.storage.wallet_dir)
    }

    /// 获取API监听地址
    pub fn get_api_addr(&self) -> String {
        format!("{}:{}", self.api.host, self.api.port)
    }
}

// ========== 预设配置 ==========

impl Config {
    /// 开发环境配置
    pub fn development() -> Self {
        let mut config = Self::default();
        config.blockchain.difficulty = 3; // 更低的难度便于快速测试
        config.logging.level = "debug".to_string();
        config.logging.format = "pretty".to_string();
        config.api.enable_rate_limit = false;
        config
    }

    /// 测试环境配置
    pub fn test() -> Self {
        let mut config = Self::default();
        config.blockchain.difficulty = 2; // 最低难度
        config.storage.data_dir = "./test_data".to_string();
        config.api.port = 3001; // 避免与开发环境冲突
        config.logging.level = "trace".to_string();
        config.logging.show_thread_id = false;
        config
    }

    /// 生产环境配置
    pub fn production() -> Self {
        let mut config = Self::default();
        config.blockchain.difficulty = 6; // 更高的难度
        config.logging.level = "info".to_string();
        config.logging.format = "json".to_string();
        config.logging.show_location = false;
        config.logging.show_thread_id = true;
        config.api.enable_rate_limit = true;
        config.api.rate_limit = 60; // 更严格的限流
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.blockchain.difficulty, 4);
        assert_eq!(config.blockchain.mining_reward, 5000);
        assert_eq!(config.api.port, 3000);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // 测试无效难度
        config.blockchain.difficulty = 0;
        assert!(config.validate().is_err());

        config.blockchain.difficulty = 11;
        assert!(config.validate().is_err());

        // 恢复有效难度
        config.blockchain.difficulty = 4;
        assert!(config.validate().is_ok());

        // 测试无效端口
        config.network.p2p_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // 保存配置
        let original_config = Config::default();
        original_config.save_to_file(&config_path)?;

        // 加载配置
        let loaded_config = Config::from_file(&config_path)?;

        // 验证
        assert_eq!(
            original_config.blockchain.difficulty,
            loaded_config.blockchain.difficulty
        );
        assert_eq!(original_config.api.port, loaded_config.api.port);

        Ok(())
    }

    #[test]
    fn test_preset_configs() {
        let dev = Config::development();
        assert_eq!(dev.blockchain.difficulty, 3);
        assert_eq!(dev.logging.level, "debug");

        let test = Config::test();
        assert_eq!(test.blockchain.difficulty, 2);
        assert_eq!(test.logging.level, "trace");

        let prod = Config::production();
        assert_eq!(prod.blockchain.difficulty, 6);
        assert_eq!(prod.logging.level, "info");
        assert!(prod.api.enable_rate_limit);
    }

    #[test]
    fn test_path_helpers() {
        let config = Config::default();
        assert_eq!(config.get_data_dir(), "./data");
        assert_eq!(config.get_blockchain_path(), "./data/blockchain.json");
        assert_eq!(config.get_wallet_dir(), "./data/wallets");
        assert_eq!(config.get_api_addr(), "127.0.0.1:3000");
    }
}
