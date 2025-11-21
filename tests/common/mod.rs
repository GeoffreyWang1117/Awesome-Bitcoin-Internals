//! 测试通用工具模块
//!
//! 提供测试中常用的辅助函数和工具

use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    config::Config,
};

/// 创建测试用的区块链实例
pub fn create_test_blockchain() -> Blockchain {
    Blockchain::new()
}

/// 创建测试钱包
pub fn create_test_wallet() -> Wallet {
    Wallet::new()
}

/// 创建多个测试钱包
pub fn create_test_wallets(count: usize) -> Vec<Wallet> {
    (0..count).map(|_| Wallet::new()).collect()
}

/// 为测试钱包分配初始余额
pub fn setup_wallet_balance(
    blockchain: &mut Blockchain,
    wallet: &Wallet,
    amount: u64,
) -> Result<(), String> {
    let genesis = Wallet::from_address("genesis".to_string());
    let tx = blockchain.create_transaction(&genesis, wallet.address.clone(), amount, 0)?;
    blockchain.add_transaction(tx)?;
    blockchain.mine_pending_transactions(wallet.address.clone())?;
    Ok(())
}

/// 断言区块链有效
pub fn assert_blockchain_valid(blockchain: &Blockchain) {
    assert!(blockchain.is_valid(), "区块链应该是有效的");
}

/// 断言余额
pub fn assert_balance(blockchain: &Blockchain, address: &str, expected: u64) {
    let actual = blockchain.get_balance(address);
    assert_eq!(
        actual, expected,
        "地址 {} 的余额应该是 {}, 实际是 {}",
        address, expected, actual
    );
}

/// 创建测试配置
pub fn test_config() -> Config {
    Config::test()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_blockchain() {
        let blockchain = create_test_blockchain();
        assert_eq!(blockchain.chain.len(), 1); // 只有创世区块
    }

    #[test]
    fn test_create_test_wallets() {
        let wallets = create_test_wallets(3);
        assert_eq!(wallets.len(), 3);
        // 确保地址不同
        assert_ne!(wallets[0].address, wallets[1].address);
        assert_ne!(wallets[1].address, wallets[2].address);
    }

    #[test]
    fn test_setup_wallet_balance() {
        let mut blockchain = create_test_blockchain();
        let wallet = create_test_wallet();

        let result = setup_wallet_balance(&mut blockchain, &wallet, 10000);
        assert!(result.is_ok());

        let balance = blockchain.get_balance(&wallet.address);
        assert_eq!(balance, 10000);
    }
}
