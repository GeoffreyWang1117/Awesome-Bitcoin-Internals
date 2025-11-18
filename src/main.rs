mod block;          // 区块模块
mod blockchain;     // 区块链模块
mod transaction;    // 交易模块
mod wallet;         // 钱包模块
mod utxo;           // UTXO模块

use blockchain::Blockchain;
use wallet::Wallet;

fn main() {
    println!("========================================");
    println!("   SimpleBTC - 基于BTC的银行系统演示");
    println!("========================================\n");

    // 1. 创建区块链
    println!(">>> 步骤 1: 初始化区块链");
    let mut blockchain = Blockchain::new();
    println!("✓ 区块链已创建，创世区块已生成\n");

    // 2. 创建钱包（银行账户）
    println!(">>> 步骤 2: 创建用户钱包");
    let wallet_alice = Wallet::new();
    let wallet_bob = Wallet::new();
    let wallet_charlie = Wallet::new();

    println!("✓ Alice 的钱包地址: {}", wallet_alice.address);
    println!("✓ Bob 的钱包地址: {}", wallet_bob.address);
    println!("✓ Charlie 的钱包地址: {}", wallet_charlie.address);
    println!();

    // 3. 给Alice和Bob发放初始余额（通过挖矿）
    println!(">>> 步骤 3: 为Alice发放初始余额");
    match blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        wallet_alice.address.clone(),
        100,
    ) {
        Ok(tx) => {
            if blockchain.add_transaction(tx).is_ok() {
                println!("✓ 交易已添加到待处理池");
            }
        }
        Err(_) => {
            println!("ℹ 使用coinbase交易给Alice发放初始余额");
        }
    }

    // 挖矿，将Alice的初始余额打包
    println!(">>> 挖矿中，打包Alice的交易...");
    match blockchain.mine_pending_transactions(wallet_alice.address.clone()) {
        Ok(_) => println!("✓ 区块已挖出\n"),
        Err(e) => println!("✗ 挖矿失败: {}\n", e),
    }

    // 给Bob发放初始余额
    println!(">>> 为Bob发放初始余额");
    match blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        wallet_bob.address.clone(),
        80,
    ) {
        Ok(tx) => {
            if blockchain.add_transaction(tx).is_ok() {
                println!("✓ 交易已添加到待处理池");
            }
        }
        Err(_) => {
            println!("ℹ 使用coinbase交易给Bob发放初始余额");
        }
    }

    println!(">>> 挖矿中，打包Bob的交易...");
    match blockchain.mine_pending_transactions(wallet_bob.address.clone()) {
        Ok(_) => println!("✓ 区块已挖出\n"),
        Err(e) => println!("✗ 挖矿失败: {}\n", e),
    }

    // 4. 查询余额
    println!(">>> 步骤 4: 查询账户余额");
    let alice_balance = blockchain.get_balance(&wallet_alice.address);
    let bob_balance = blockchain.get_balance(&wallet_bob.address);
    let charlie_balance = blockchain.get_balance(&wallet_charlie.address);

    println!("Alice 的余额: {} BTC", alice_balance);
    println!("Bob 的余额: {} BTC", bob_balance);
    println!("Charlie 的余额: {} BTC", charlie_balance);
    println!();

    // 5. 执行转账交易 - Alice 转账给 Bob
    println!(">>> 步骤 5: Alice 向 Bob 转账 30 BTC");
    match blockchain.create_transaction(&wallet_alice, wallet_bob.address.clone(), 30) {
        Ok(tx) => {
            println!("✓ 交易已创建: {}", tx.id);
            match blockchain.add_transaction(tx) {
                Ok(_) => println!("✓ 交易已验证并添加到待处理池"),
                Err(e) => println!("✗ 交易添加失败: {}", e),
            }
        }
        Err(e) => println!("✗ 创建交易失败: {}", e),
    }
    println!();

    // 6. Bob 转账给 Charlie
    println!(">>> 步骤 6: Bob 向 Charlie 转账 20 BTC");
    match blockchain.create_transaction(&wallet_bob, wallet_charlie.address.clone(), 20) {
        Ok(tx) => {
            println!("✓ 交易已创建: {}", tx.id);
            match blockchain.add_transaction(tx) {
                Ok(_) => println!("✓ 交易已验证并添加到待处理池"),
                Err(e) => println!("✗ 交易添加失败: {}", e),
            }
        }
        Err(e) => println!("✗ 创建交易失败: {}", e),
    }
    println!();

    // 7. 挖矿，打包所有待处理交易
    println!(">>> 步骤 7: 挖矿打包待处理交易");
    let miner_wallet = Wallet::new();
    println!("矿工地址: {}", miner_wallet.address);

    match blockchain.mine_pending_transactions(miner_wallet.address.clone()) {
        Ok(_) => println!("✓ 区块已成功挖出并添加到链中"),
        Err(e) => println!("✗ 挖矿失败: {}", e),
    }
    println!();

    // 8. 再次查询余额（事务处理后）
    println!(">>> 步骤 8: 查询交易后的账户余额");
    let alice_balance = blockchain.get_balance(&wallet_alice.address);
    let bob_balance = blockchain.get_balance(&wallet_bob.address);
    let charlie_balance = blockchain.get_balance(&wallet_charlie.address);
    let miner_balance = blockchain.get_balance(&miner_wallet.address);

    println!("Alice 的余额: {} BTC (应该减少30)", alice_balance);
    println!("Bob 的余额: {} BTC (应该增加30减少20)", bob_balance);
    println!("Charlie 的余额: {} BTC (应该增加20)", charlie_balance);
    println!("矿工的余额: {} BTC (挖矿奖励)", miner_balance);
    println!();

    // 9. 测试余额不足的情况（事务处理规则）
    println!(">>> 步骤 9: 测试余额不足情况");
    println!("Charlie 尝试向 Alice 转账 100 BTC（余额不足）");
    match blockchain.create_transaction(&wallet_charlie, wallet_alice.address.clone(), 100) {
        Ok(_) => println!("✗ 应该失败，但交易成功创建了！"),
        Err(e) => println!("✓ 正确拒绝: {}", e),
    }
    println!();

    // 10. 验证区块链完整性
    println!(">>> 步骤 10: 验证区块链完整性");
    if blockchain.is_valid() {
        println!("✓ 区块链验证通过，所有区块和交易都有效");
    } else {
        println!("✗ 区块链验证失败！");
    }
    println!();

    // 11. 打印完整的区块链信息
    println!(">>> 步骤 11: 打印区块链详细信息");
    blockchain.print_chain();

    // 12. 演示事务特性（ACID）
    println!("========================================");
    println!("   事务处理特性演示");
    println!("========================================\n");

    println!("【原子性 Atomicity】");
    println!("- 交易要么全部执行，要么全部不执行");
    println!("- 余额不足时，交易被完全拒绝，不会部分执行\n");

    println!("【一致性 Consistency】");
    println!("- 交易前后，所有账户余额总和保持一致");
    println!("- 每笔交易都经过验证，确保输入 ≥ 输出\n");

    println!("【隔离性 Isolation】");
    println!("- 交易在待处理池中等待，不影响当前状态");
    println!("- 只有在挖矿成功后，交易才会被确认并更新UTXO集合\n");

    println!("【持久性 Durability】");
    println!("- 一旦交易被打包进区块并挖矿成功");
    println!("- 交易记录永久保存在区块链中，不可篡改\n");

    println!("========================================");
    println!("   演示完成");
    println!("========================================");
}
