//! 案例2: 托管服务
//!
//! 场景：买卖双方通过第三方仲裁者进行交易
//! 使用2-of-3多签实现托管机制
//!
//! 运行: cargo run --example escrow_service

use bitcoin_simulation::{
    blockchain::Blockchain,
    multisig::{MultiSigAddress, MultiSigTxBuilder},
    wallet::Wallet,
};

fn main() {
    println!("========================================");
    println!("   BTC托管服务案例");
    println!("========================================\n");

    // 初始化区块链
    let mut blockchain = Blockchain::new();

    // 创建参与方钱包
    println!(">>> 第1步: 创建交易参与方");
    let buyer = Wallet::new(); // 买家
    let seller = Wallet::new(); // 卖家
    let arbitrator = Wallet::new(); // 仲裁者

    println!("👤 买家地址: {}", buyer.address);
    println!("👤 卖家地址: {}", seller.address);
    println!("⚖️  仲裁者地址: {}", arbitrator.address);
    println!();

    // 给买家发放初始资金
    println!(">>> 第2步: 买家获得初始资金");
    if let Ok(tx) = blockchain.create_transaction(
        &Blockchain::genesis_wallet(),
        buyer.address.clone(),
        100000, // 10万 satoshi
        0,
    ) {
        blockchain.add_transaction(tx).ok();
        blockchain
            .mine_pending_transactions(buyer.address.clone())
            .ok();
        println!("✓ 买家获得 100,000 satoshi\n");
    }

    // 创建托管多签地址（2-of-3）
    println!(">>> 第3步: 创建托管多签地址");
    let public_keys = vec![
        buyer.public_key.clone(),
        seller.public_key.clone(),
        arbitrator.public_key.clone(),
    ];

    let escrow_address = MultiSigAddress::new(2, public_keys).expect("创建托管地址失败");

    println!("🔒 托管地址: {}", escrow_address.address);
    println!("📋 规则: 任意2方签名可释放资金");
    println!("   • 买家 + 卖家 = 正常交易");
    println!("   • 买家 + 仲裁 = 退款");
    println!("   • 卖家 + 仲裁 = 纠纷解决");
    println!();

    // 买家将资金存入托管账户
    println!(">>> 第4步: 买家存入托管款");
    let escrow_amount = 50000; // 5万 satoshi (商品价格)

    if let Ok(tx) = blockchain.create_transaction(
        &buyer,
        escrow_address.address.clone(),
        escrow_amount,
        100, // 手续费
    ) {
        blockchain.add_transaction(tx).ok();
        blockchain
            .mine_pending_transactions(buyer.address.clone())
            .ok();
        println!("✓ {} satoshi 已存入托管", escrow_amount);
    }

    let escrow_balance = blockchain.get_balance(&escrow_address.address);
    println!("🔒 托管账户余额: {} satoshi\n", escrow_balance);

    // 场景A: 正常交易完成（买家和卖家都满意）
    println!("========================================");
    println!("   场景A: 交易顺利完成");
    println!("========================================\n");

    println!(">>> 卖家已发货，买家确认收货");

    let mut release_builder = MultiSigTxBuilder::new(escrow_address.clone());

    // 买家签名（确认收货）
    println!("✅ 买家签名（确认收货）");
    release_builder
        .add_signature(&buyer, "release_to_seller")
        .ok();

    // 卖家签名（同意释放）
    println!("✅ 卖家签名（请求付款）");
    release_builder
        .add_signature(&seller, "release_to_seller")
        .ok();

    if release_builder.is_complete() {
        println!("\n✓ 收集到2个签名，资金释放给卖家");
        println!("💰 卖家收到 {} satoshi\n", escrow_amount);
    }

    // 场景B: 买家不满意，申请仲裁
    println!("========================================");
    println!("   场景B: 交易纠纷，仲裁介入");
    println!("========================================\n");

    println!(">>> 买家不满意商品质量，申请退款");

    let mut refund_builder = MultiSigTxBuilder::new(escrow_address.clone());

    // 买家签名（申请退款）
    println!("❌ 买家签名（要求退款）");
    refund_builder.add_signature(&buyer, "refund_to_buyer").ok();

    println!("⚖️  仲裁者审查证据...");
    println!("⚖️  仲裁者判定：退款合理");

    // 仲裁者签名（支持退款）
    println!("✅ 仲裁者签名（批准退款）");
    refund_builder
        .add_signature(&arbitrator, "refund_to_buyer")
        .ok();

    if refund_builder.is_complete() {
        println!("\n✓ 收集到2个签名，资金退回买家");
        println!("💰 买家收到退款 {} satoshi\n", escrow_amount);
    }

    println!("========================================");
    println!("   托管服务优势");
    println!("========================================");
    println!("✅ 买家保护: 商品不符可退款");
    println!("✅ 卖家保护: 确认收货后付款");
    println!("✅ 公正仲裁: 第三方解决纠纷");
    println!("✅ 资金安全: 区块链托管");
    println!("========================================\n");

    println!("💡 实际应用场景:");
    println!("  • 电商平台交易");
    println!("  • 自由职业者服务");
    println!("  • 房产交易托管");
    println!("  • 国际贸易结算");
    println!("  • 众筹项目管理");
}
