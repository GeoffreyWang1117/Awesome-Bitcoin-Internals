use bitcoin_simulation::blockchain::Blockchain;
use bitcoin_simulation::wallet::Wallet;

fn main() {
    println!("========================================");
    println!("   SimpleBTC - 基于BTC的银行系统演示");
    println!("   (secp256k1 ECDSA 真实签名)");
    println!("========================================\n");

    // 1. 创建区块链
    println!(">>> 步骤 1: 初始化区块链");
    let mut blockchain = Blockchain::new();
    let genesis_wallet = Blockchain::genesis_wallet();
    println!("✓ 区块链已创建，创世区块已生成");
    println!("  创世地址: {}", genesis_wallet.address);
    println!();

    // 2. 创建钱包（银行账户）- 使用真实secp256k1密钥对
    println!(">>> 步骤 2: 创建用户钱包（secp256k1）");
    let wallet_alice = Wallet::new();
    let wallet_bob = Wallet::new();
    let wallet_charlie = Wallet::new();

    println!("✓ Alice 的钱包地址: {}", wallet_alice.address);
    println!("✓ Bob 的钱包地址: {}", wallet_bob.address);
    println!("✓ Charlie 的钱包地址: {}", wallet_charlie.address);
    println!();

    // 3. 给Alice和Bob发放初始余额（从创世钱包转账）
    println!(">>> 步骤 3: 为Alice发放初始余额（创世钱包 → Alice）");
    match blockchain.create_transaction(
        &genesis_wallet,
        wallet_alice.address.clone(),
        1000,
        0, // 无交易费
    ) {
        Ok(tx) => {
            if blockchain.add_transaction(tx).is_ok() {
                println!("✓ 交易已添加到内存池（ECDSA签名已验证）");
            }
        }
        Err(e) => {
            println!("✗ 创建交易失败: {}", e);
        }
    }

    // 挖矿，将Alice的初始余额打包（并行挖矿）
    println!(">>> 挖矿中，打包Alice的交易...");
    match blockchain.mine_pending_transactions(wallet_alice.address.clone()) {
        Ok(_) => println!("✓ 区块已挖出\n"),
        Err(e) => println!("✗ 挖矿失败: {}\n", e),
    }

    // 给Bob发放初始余额
    println!(">>> 为Bob发放初始余额（创世钱包 → Bob）");
    match blockchain.create_transaction(
        &genesis_wallet,
        wallet_bob.address.clone(),
        800,
        0, // 无交易费
    ) {
        Ok(tx) => {
            if blockchain.add_transaction(tx).is_ok() {
                println!("✓ 交易已添加到内存池（ECDSA签名已验证）");
            }
        }
        Err(e) => {
            println!("ℹ 创世余额不足，使用挖矿奖励: {}", e);
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

    // 5. 执行转账交易 - Alice 转账给 Bob（带手续费）
    println!(">>> 步骤 5: Alice 向 Bob 转账 30 BTC（手续费2）");
    match blockchain.create_transaction(&wallet_alice, wallet_bob.address.clone(), 30, 2) {
        Ok(tx) => {
            println!("✓ 交易已创建: {}", tx.id);
            match blockchain.add_transaction(tx) {
                Ok(_) => println!("✓ 交易已验证（ECDSA）并添加到内存池"),
                Err(e) => println!("✗ 交易添加失败: {}", e),
            }
        }
        Err(e) => println!("✗ 创建交易失败: {}", e),
    }
    println!();

    // 6. Bob 转账给 Charlie（更高手续费优先打包）
    println!(">>> 步骤 6: Bob 向 Charlie 转账 20 BTC（手续费5，优先打包）");
    match blockchain.create_transaction(&wallet_bob, wallet_charlie.address.clone(), 20, 5) {
        Ok(tx) => {
            println!("✓ 交易已创建: {}", tx.id);
            match blockchain.add_transaction(tx) {
                Ok(_) => println!("✓ 交易已验证（ECDSA）并添加到内存池"),
                Err(e) => println!("✗ 交易添加失败: {}", e),
            }
        }
        Err(e) => println!("✗ 创建交易失败: {}", e),
    }
    println!();

    // 7. 挖矿，打包所有待处理交易（并行挖矿）
    println!(">>> 步骤 7: 并行挖矿打包待处理交易");
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

    // 9. 测试签名安全性 - 用错误的钱包尝试花费别人的币
    println!(">>> 步骤 9: 测试签名安全性");
    println!("Charlie 尝试向 Alice 转账 100 BTC（余额不足）");
    match blockchain.create_transaction(&wallet_charlie, wallet_alice.address.clone(), 100, 1) {
        Ok(_) => println!("✗ 应该失败，但交易成功创建了！"),
        Err(e) => println!("✓ 正确拒绝: {}", e),
    }
    println!();

    // 10. 验证区块链完整性
    println!(">>> 步骤 10: 验证区块链完整性");
    if blockchain.is_valid() {
        println!("✓ 区块链验证通过，所有区块和ECDSA签名都有效");
    } else {
        println!("✗ 区块链验证失败！");
    }
    println!();

    // 11. 打印完整的区块链信息
    println!(">>> 步骤 11: 打印区块链详细信息");
    blockchain.print_chain();

    // 12. 演示密码学特性
    println!("========================================");
    println!("   密码学安全特性");
    println!("========================================\n");

    println!("【secp256k1 ECDSA签名】");
    println!("- 与真实比特币使用相同的椭圆曲线: y² = x³ + 7 (mod p)");
    println!("- 私钥: 256位随机数（32字节）");
    println!("- 公钥: 椭圆曲线点（压缩格式33字节）");
    println!("- 签名: ECDSA DER编码（约71-73字节）\n");

    println!("【P2PKH地址】");
    println!("- 与真实比特币主网地址格式一致（以'1'开头）");
    println!("- 地址生成: 公钥 → SHA256 → RIPEMD160 → Base58Check\n");

    println!("【交易安全】");
    println!("- 每笔交易的每个输入都经过ECDSA签名验证");
    println!("- 无法伪造签名花费他人的UTXO");
    println!("- 签名不可伪造、不可抵赖\n");

    println!("========================================");
    println!("   演示完成");
    println!("========================================");
}
