use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub difficulty: usize,
    pub mining_reward: u64,
    pub data_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            difficulty: 3,
            mining_reward: 50,
            data_dir: String::from("./data"),
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn load(path: &str) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(content) => {
                serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))
            }
            Err(e) => Err(format!("读取配置文件失败: {}", e)),
        }
    }

    /// 保存配置到文件
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(path, json).map_err(|e| format!("写入配置文件失败: {}", e))
    }
}

/// 存储管理器
pub struct StorageManager {
    pub data_dir: String,
}

impl StorageManager {
    pub fn new(data_dir: String) -> Self {
        // 确保数据目录存在
        if let Err(e) = fs::create_dir_all(&data_dir) {
            eprintln!("创建数据目录失败: {}", e);
        }

        StorageManager { data_dir }
    }

    /// 保存区块链到文件
    pub fn save_blockchain(&self, blockchain: &Blockchain) -> Result<(), String> {
        let path = Path::new(&self.data_dir).join("blockchain.json");
        let json = serde_json::to_string_pretty(&blockchain.chain)
            .map_err(|e| format!("序列化区块链失败: {}", e))?;

        fs::write(path, json).map_err(|e| format!("写入区块链文件失败: {}", e))?;

        println!("✓ 区块链已保存");
        Ok(())
    }

    /// 从文件加载区块链
    pub fn load_blockchain(&self) -> Result<String, String> {
        let path = Path::new(&self.data_dir).join("blockchain.json");

        fs::read_to_string(path).map_err(|e| format!("读取区块链文件失败: {}", e))
    }

    /// 保存钱包到文件
    pub fn save_wallet(&self, wallet: &Wallet, name: &str) -> Result<(), String> {
        let wallets_dir = Path::new(&self.data_dir).join("wallets");
        fs::create_dir_all(&wallets_dir).map_err(|e| format!("创建钱包目录失败: {}", e))?;

        let path = wallets_dir.join(format!("{}.json", name));
        let json =
            serde_json::to_string_pretty(wallet).map_err(|e| format!("序列化钱包失败: {}", e))?;

        fs::write(path, json).map_err(|e| format!("写入钱包文件失败: {}", e))?;

        println!("✓ 钱包 '{}' 已保存", name);
        Ok(())
    }

    /// 从文件加载钱包
    pub fn load_wallet(&self, name: &str) -> Result<Wallet, String> {
        let path = Path::new(&self.data_dir)
            .join("wallets")
            .join(format!("{}.json", name));

        let content = fs::read_to_string(path).map_err(|e| format!("读取钱包文件失败: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("解析钱包文件失败: {}", e))
    }

    /// 列出所有钱包
    pub fn list_wallets(&self) -> Result<Vec<String>, String> {
        let wallets_dir = Path::new(&self.data_dir).join("wallets");

        if !wallets_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(wallets_dir).map_err(|e| format!("读取钱包目录失败: {}", e))?;

        let mut wallets = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    wallets.push(name.trim_end_matches(".json").to_string());
                }
            }
        }

        Ok(wallets)
    }

    /// 导出钱包（包含私钥）
    pub fn export_wallet(&self, wallet: &Wallet, export_path: &str) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(wallet).map_err(|e| format!("序列化钱包失败: {}", e))?;

        fs::write(export_path, json).map_err(|e| format!("导出钱包失败: {}", e))?;

        println!("✓ 钱包已导出到: {}", export_path);
        Ok(())
    }

    /// 导入钱包
    pub fn import_wallet(&self, import_path: &str, name: &str) -> Result<Wallet, String> {
        let content =
            fs::read_to_string(import_path).map_err(|e| format!("读取导入文件失败: {}", e))?;

        let wallet: Wallet =
            serde_json::from_str(&content).map_err(|e| format!("解析钱包数据失败: {}", e))?;

        // 保存到钱包目录
        self.save_wallet(&wallet, name)?;

        println!("✓ 钱包已导入: {}", name);
        Ok(wallet)
    }

    /// 保存交易缓存（加速查询）
    pub fn save_tx_index(
        &self,
        tx_map: &std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        let path = Path::new(&self.data_dir).join("tx_index.json");
        let json = serde_json::to_string_pretty(tx_map)
            .map_err(|e| format!("序列化交易索引失败: {}", e))?;

        fs::write(path, json).map_err(|e| format!("写入交易索引失败: {}", e))
    }

    /// 加载交易缓存
    pub fn load_tx_index(&self) -> Result<std::collections::HashMap<String, String>, String> {
        let path = Path::new(&self.data_dir).join("tx_index.json");

        let content = fs::read_to_string(path).map_err(|e| format!("读取交易索引失败: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("解析交易索引失败: {}", e))
    }
}
