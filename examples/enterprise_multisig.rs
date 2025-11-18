/// 案例1: 企业多签钱包
///
/// 场景：一家公司需要CEO和CFO双签才能转账
/// 使用2-of-2多重签名确保资金安全
///
/// 运行: cargo run --example enterprise_multisig

use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    multisig::{MultiSigAddress, MultiSigTxBuilder},
};

fn main() {
    println!("========================================");
    println!("   企业多签钱包案例");
    println!("========================================\n");

    // 初始化区块链
    let mut blockchain = Blockchain::new();
    println!("✓ 区块链初始化完成\n");

    // 创建企业钱包
    println!(">>> 第1步: 创建企业管理层钱包");
    let ceo_wallet = Wallet::new();
    let cfo_wallet = Wallet::new();
    let cto_wallet = Wallet::new();

    println!("CEO钱包: {}", ceo_wallet.address);
    println!("CFO钱包: {}", cfo_wallet.address);
    println!("CTO钱包: {}", cto_wallet.address);
    println!();

    // 创建2-of-3多签钱包（任意2人签名即可）
    println!(">>> 第2步: 创建公司多签钱包 (2-of-3)");
    let public_keys = vec![
        ceo_wallet.public_key.clone(),
        cfo_wallet.public_key.clone(),
        cto_wallet.public_key.clone(),
    ];

    let company_multisig = MultiSigAddress::new(2, public_keys)
        .expect("创建多签地址失败");

    println!("公司多签地址: {}", company_multisig.address);
    println!("需要签名数: {}/{}", company_multisig.required_sigs, company_multisig.total_keys);
    println!();

    // 给公司账户发放初始资金
    println!(">>> 第3步: 为公司账户注资");
    if let Ok(tx) = blockchain.create_transaction(
        &Wallet::from_address("genesis_address".to_string()),
        company_multisig.address.clone(),
        1000000, // 100万 satoshi
        0,
    ) {
        blockchain.add_transaction(tx).ok();
        blockchain.mine_pending_transactions(ceo_wallet.address.clone()).ok();
        println!("✓ 公司账户已注资 1,000,000 satoshi\n");
    }

    // 查询余额
    let balance = blockchain.get_balance(&company_multisig.address);
    println!("公司账户余额: {} satoshi\n", balance);

    // 模拟公司转账（需要2个签名）
    println!(">>> 第4步: 公司向供应商付款 (需要2个高管签名)");
    let supplier_wallet = Wallet::new();
    println!("供应商地址: {}", supplier_wallet.address);

    // 创建交易
    let payment_amount = 50000; // 5万 satoshi
    println!("\n准备支付: {} satoshi", payment_amount);

    // 构建多签交易
    let mut multisig_builder = MultiSigTxBuilder::new(company_multisig.clone());

    // CEO签名
    println!("✓ CEO已签名");
    multisig_builder.add_signature(&ceo_wallet, "payment_to_supplier").ok();

    // CFO签名
    println!("✓ CFO已签名");
    multisig_builder.add_signature(&cfo_wallet, "payment_to_supplier").ok();

    if multisig_builder.is_complete() {
        println!("\n✓ 已收集到足够的签名 (2/3)");
        println!("✓ 交易可以广播到网络\n");
    }

    // 验证签名
    let signatures = multisig_builder.get_signatures();
    if company_multisig.verify_signatures(&signatures) {
        println!("✓ 多签验证通过！");
    }

    println!("\n========================================");
    println!("   案例总结");
    println!("========================================");
    println!("✅ 创建了2-of-3多签钱包");
    println!("✅ 任意2位高管签名即可转账");
    println!("✅ 提高了企业资金安全性");
    println!("✅ 防止单点故障和内部欺诈");
    println!("========================================\n");

    println!("💡 实际应用场景:");
    println!("  • 公司财务管理");
    println!("  • 风险投资基金");
    println!("  • DAO组织治理");
    println!("  • 联合账户管理");
}
