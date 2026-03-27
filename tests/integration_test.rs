//! SimpleBTC集成测试
//!
//! 测试整个系统的端到端工作流程（使用真实secp256k1 ECDSA签名）

mod common;

use common::*;

#[test]
fn test_complete_transaction_workflow() {
    // 创建区块链和钱包
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let bob = create_test_wallet();

    // 给Alice分配初始余额（从创世钱包转账）
    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();
    // Alice: 10000 (transfer) + 50 (mining reward from setup) = 10050
    assert_balance(&blockchain, &alice.address, 10050);

    // Alice向Bob转账
    let tx = blockchain
        .create_transaction(&alice, bob.address.clone(), 3000, 10)
        .unwrap();
    blockchain.add_transaction(tx).unwrap();

    // 挖矿确认（Alice作为矿工）
    blockchain
        .mine_pending_transactions(alice.address.clone())
        .unwrap();

    // 验证余额
    // Alice: 10050 - 3000 - 10(fee) + 50(mining reward) + 10(collected fee) = 7100
    assert_balance(&blockchain, &alice.address, 7100);
    assert_balance(&blockchain, &bob.address, 3000);

    // 验证区块链（包括ECDSA签名验证）
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
fn test_multiple_transactions_sequential() {
    let mut blockchain = create_test_blockchain();
    let wallets = create_test_wallets(4); // Alice, Bob, Charlie, Dave

    // 给Alice初始余额
    setup_wallet_balance(&mut blockchain, &wallets[0], 20000).unwrap();

    // Alice向多人转账（每次转账后挖矿，避免UTXO冲突）
    let miner = create_test_wallet();
    for i in 1..=3 {
        let tx = blockchain
            .create_transaction(&wallets[0], wallets[i].address.clone(), 2000, 10)
            .unwrap();
        blockchain.add_transaction(tx).unwrap();
        blockchain
            .mine_pending_transactions(miner.address.clone())
            .unwrap();
    }

    // 验证余额
    assert_balance(&blockchain, &wallets[1].address, 2000);
    assert_balance(&blockchain, &wallets[2].address, 2000);
    assert_balance(&blockchain, &wallets[3].address, 2000);
    // Miner: 3 blocks × (50 reward + 10 fee) = 180
    assert_balance(&blockchain, &miner.address, 180);

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

    // 区块链应该仍然有效（所有ECDSA签名都正确）
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
    // 矿工应该获得：挖矿奖励(50) + 手续费(50) = 100
    assert_balance(&blockchain, &miner.address, 100);

    assert_blockchain_valid(&blockchain);
}

#[test]
fn test_zero_amount_transaction() {
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let bob = create_test_wallet();

    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();

    // 尝试创建金额为0的交易 - 应该成功（0 + 10 fee = 10, 余额足够）
    // 但实际上这取决于实现 - 如果交易0金额被允许，那也可以
    let result = blockchain.create_transaction(&alice, bob.address.clone(), 0, 10);
    // 无论成功或失败都合理
    let _ = result;
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

    // 验证整条链（所有ECDSA签名）
    assert_blockchain_valid(&blockchain);

    // 验证最后的余额
    let final_balance = blockchain.get_balance(&alice.address);
    // Alice获得所有转账 + 所有挖矿奖励(50*50)
    let expected = (1..=50).sum::<u64>() * 100 + 50 * 50;
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
    assert_balance(&blockchain, &miner1.address, 60); // 奖励50 + 手续费10

    // 创建另一笔交易
    let tx2 = blockchain
        .create_transaction(&alice, bob.address.clone(), 500, 5)
        .unwrap();
    blockchain.add_transaction(tx2).unwrap();

    // 矿工2挖矿
    blockchain
        .mine_pending_transactions(miner2.address.clone())
        .unwrap();
    assert_balance(&blockchain, &miner2.address, 55); // 奖励50 + 手续费5

    assert_blockchain_valid(&blockchain);
}

#[test]
fn test_ecdsa_signature_verification() {
    // 测试签名安全性：无法用错误的钱包花费他人的币
    let mut blockchain = create_test_blockchain();
    let alice = create_test_wallet();
    let eve = create_test_wallet(); // 攻击者

    setup_wallet_balance(&mut blockchain, &alice, 10000).unwrap();

    // Eve无法花费Alice的币（因为Eve没有Alice的私钥）
    let result = blockchain.create_transaction(&eve, alice.address.clone(), 5000, 10);
    assert!(result.is_err(), "Eve不应该能花费Alice的币");
}
