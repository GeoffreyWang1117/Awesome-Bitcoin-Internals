//! SimpleBTC集成测试
//!
//! 测试整个系统的端到端工作流程

mod common;

use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};
use common::*;

#[test]
fn test_complete_transaction_workflow() {
    // 创建区块链和钱包
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let bob = create_test_wallet();

    // 给Alice分配初始余额
    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();
    assert_balance(&blockchain, &alice.address, 10000);

    // Alice向Bob转账
    let tx = blockchain
        .create_transaction(&alice, bob.address.clone(), 3000, 10)
        .unwrap();
    blockchain.add_transaction(tx).unwrap();

    // 挖矿确认
    blockchain
        .mine_pending_transactions(alice.address.clone())
        .unwrap();

    // 验证余额
    // Alice: 10000 - 3000 - 10(手续费) + 5000(挖矿奖励) = 11990
    assert_balance(&blockchain, &alice.address, 11990);
    assert_balance(&blockchain, &bob.address, 3000);

    // 验证区块链
    assert_blockchain_valid(&blockchain);
}

#[test]
fn test_insufficient_balance() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let bob = create_test_wallet();

    // 给Alice少量余额
    setup_wallet_balance(&mut blockchain, &alice, 100).unwrap();

    // 尝试转账超过余额的金额
    let result = blockchain.create_transaction(&alice, bob.address.clone(), 1000, 10);

    // 应该失败
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("余额不足"));
}

#[test]
fn test_multiple_transactions_in_block() {
    let mut blockchain = create_test_blockchain();
    let wallets = create_test_wallets(4); // Alice, Bob, Charlie, Dave

    // 给Alice初始余额
    setup_wallet_balance(&mut blockchain, &wallets[0], 20000).unwrap();

    // Alice向多人转账
    for i in 1..=3 {
        let tx = blockchain
            .create_transaction(&wallets[0], wallets[i].address.clone(), 2000, 10)
            .unwrap();
        blockchain.add_transaction(tx).unwrap();
    }

    // 挖矿确认所有交易
    blockchain
        .mine_pending_transactions(wallets[0].address.clone())
        .unwrap();

    // 验证余额
    // Alice: 20000 - 3*2000 - 3*10 + 5000 = 18970
    assert_balance(&blockchain, &wallets[0].address, 18970);
    assert_balance(&blockchain, &wallets[1].address, 2000);
    assert_balance(&blockchain, &wallets[2].address, 2000);
    assert_balance(&blockchain, &wallets[3].address, 2000);

    assert_blockchain_valid(&blockchain);
}

#[test]
fn test_chain_validation() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();

    // 进行几笔交易
    for i in 1..=5 {
        setup_wallet_balance(&mut blockchain, &alice, 1000 * i).unwrap();
    }

    // 区块链应该仍然有效
    assert_blockchain_valid(&blockchain);
    assert_eq!(blockchain.chain.len(), 6); // 创世块 + 5个新块
}

#[test]
fn test_transaction_fee_handling() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let bob = create_test_wallet();
    let miner = create_test_wallet();

    // 给Alice初始余额
    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();

    // Alice向Bob转账，支付手续费
    let tx = blockchain
        .create_transaction(&alice, bob.address.clone(), 1000, 50)
        .unwrap();
    blockchain.add_transaction(tx).unwrap();

    // 矿工挖矿
    blockchain
        .mine_pending_transactions(miner.address.clone())
        .unwrap();

    // 验证余额
    assert_balance(&blockchain, &bob.address, 1000);
    // 矿工应该获得：挖矿奖励(5000) + 手续费(50) = 5050
    assert_balance(&blockchain, &miner.address, 5050);

    assert_blockchain_valid(&blockchain);
}

#[test]
fn test_zero_amount_transaction_rejected() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let bob = create_test_wallet();

    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();

    // 尝试创建金额为0的交易
    let result = blockchain.create_transaction(&alice, bob.address.clone(), 0, 10);

    // 应该失败
    assert!(result.is_err());
}

#[test]
fn test_long_chain() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();

    // 创建长链
    for i in 1..=50 {
        setup_wallet_balance(&mut blockchain, &alice, i * 100).unwrap();
    }

    // 验证链的长度
    assert_eq!(blockchain.chain.len(), 51); // 创世块 + 50个块

    // 验证整条链
    assert_blockchain_valid(&blockchain);

    // 验证最后的余额
    let final_balance = blockchain.get_balance(&alice.address);
    // Alice应该获得所有的转账 + 挖矿奖励
    let expected = (1..=50).sum::<u64>() * 100 + 50 * 5000;
    assert_eq!(final_balance, expected);
}

#[test]
fn test_concurrent_miners() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let miner1 = create_test_wallet();
    let miner2 = create_test_wallet();

    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();

    // 创建交易
    let bob = create_test_wallet();
    let tx = blockchain
        .create_transaction(&alice, bob.address.clone(), 1000, 10)
        .unwrap();
    blockchain.add_transaction(tx).unwrap();

    // 矿工1挖矿
    blockchain
        .mine_pending_transactions(miner1.address.clone())
        .unwrap();
    assert_balance(&blockchain, &miner1.address, 5010); // 奖励 + 手续费

    // 创建另一笔交易
    let tx2 = blockchain
        .create_transaction(&alice, bob.address.clone(), 500, 5)
        .unwrap();
    blockchain.add_transaction(tx2).unwrap();

    // 矿工2挖矿
    blockchain
        .mine_pending_transactions(miner2.address.clone())
        .unwrap();
    assert_balance(&blockchain, &miner2.address, 5005);

    assert_blockchain_valid(&blockchain);
}
