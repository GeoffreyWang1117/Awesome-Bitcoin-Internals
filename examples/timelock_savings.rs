//! 案例3: 时间锁定期存款
//!
//! 场景：用户创建定期存款，锁定期满后才能提取
//! 使用时间锁实现强制储蓄
//!
//! 运行: cargo run --example timelock_savings

use bitcoin_simulation::{
    advanced_tx::{AdvancedTxBuilder, TimeLock},
    blockchain::Blockchain,
    wallet::Wallet,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("========================================");
    println!("   时间锁定期存款案例");
    println!("========================================\n");

    // 初始化区块链
    let mut blockchain = Blockchain::new();

    // 创建用户钱包
    println!(">>> 第1步: 创建用户钱包");
    let user_wallet = Wallet::new();
    let savings_wallet = Wallet::new(); // 专用储蓄账户

    println!("用户钱包: {}", user_wallet.address);
    println!("储蓄账户: {}", savings_wallet.address);
    println!();

    // 给用户发放初始资金
    println!(">>> 第2步: 用户获得初始资金");
    if let Ok(tx) = blockchain.create_transaction(
        &Blockchain::genesis_wallet(),
        user_wallet.address.clone(),
        200000, // 20万 satoshi
        0,
    ) {
        blockchain.add_transaction(tx).ok();
        blockchain
            .mine_pending_transactions(user_wallet.address.clone())
            .ok();
        println!("✓ 用户获得 200,000 satoshi\n");
    }

    // 获取当前时间
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 创建不同期限的定期存款
    println!("========================================");
    println!("   定期存款产品");
    println!("========================================\n");

    // 产品1: 3个月定期（简化为3分钟演示）
    let three_months = 180; // 实际应该是 90 * 24 * 3600
    let unlock_time_3m = current_time + three_months;
    let timelock_3m = TimeLock::new_time_based(unlock_time_3m);

    println!("📅 产品1: 3个月定期存款");
    println!("   存入金额: 50,000 satoshi");
    println!("   锁定期: 3个月");
    println!("   解锁时间: {} (Unix时间戳)", unlock_time_3m);

    if timelock_3m.is_mature(current_time, 0) {
        println!("   状态: ✅ 已到期");
    } else {
        let remaining = timelock_3m.remaining(current_time, 0);
        println!("   状态: 🔒 锁定中 (剩余 {} 秒)", remaining);
    }
    println!();

    // 产品2: 1年定期（简化为1小时演示）
    let one_year = 3600; // 实际应该是 365 * 24 * 3600
    let unlock_time_1y = current_time + one_year;
    let timelock_1y = TimeLock::new_time_based(unlock_time_1y);

    println!("📅 产品2: 1年定期存款");
    println!("   存入金额: 100,000 satoshi");
    println!("   锁定期: 1年");
    println!("   解锁时间: {} (Unix时间戳)", unlock_time_1y);

    if timelock_1y.is_mature(current_time, 0) {
        println!("   状态: ✅ 已到期");
    } else {
        let remaining = timelock_1y.remaining(current_time, 0);
        println!("   状态: 🔒 锁定中 (剩余 {} 秒)", remaining);
    }
    println!();

    // 使用时间锁构建器创建交易
    println!("========================================");
    println!("   存入定期存款");
    println!("========================================\n");

    println!(">>> 用户存入3个月定期 (50,000 satoshi)");

    // 创建带时间锁的交易
    let tx_builder = AdvancedTxBuilder::new().with_timelock(timelock_3m.clone());

    println!("✓ 交易已创建");
    println!("✓ 时间锁已设置: 序列号 = {}", tx_builder.get_sequence());
    println!("✓ 资金将在 {} 后可用\n", format_duration(three_months));

    // 模拟存款
    if let Ok(tx) =
        blockchain.create_transaction(&user_wallet, savings_wallet.address.clone(), 50000, 100)
    {
        blockchain.add_transaction(tx).ok();
        blockchain
            .mine_pending_transactions(user_wallet.address.clone())
            .ok();
        println!("✓ 存款成功！\n");
    }

    // 查询余额
    let user_balance = blockchain.get_balance(&user_wallet.address);
    let savings_balance = blockchain.get_balance(&savings_wallet.address);

    println!("账户余额:");
    println!("  用户账户: {} satoshi", user_balance);
    println!("  储蓄账户: {} satoshi (🔒锁定中)\n", savings_balance);

    // 模拟提前支取失败
    println!("========================================");
    println!("   尝试提前支取");
    println!("========================================\n");

    println!(">>> 用户尝试提前支取...");

    if timelock_3m.is_mature(current_time, 0) {
        println!("✅ 时间锁已到期，可以支取");
    } else {
        let remaining = timelock_3m.remaining(current_time, 0);
        println!("❌ 时间锁未到期，无法支取");
        println!("⏰ 还需等待: {} 秒", remaining);
        println!("💡 强制储蓄机制生效\n");
    }

    // 模拟时间流逝后支取
    println!("========================================");
    println!("   到期后支取");
    println!("========================================\n");

    let future_time = current_time + three_months + 1; // 锁定期满后
    println!(">>> 时间流逝... (模拟)");
    println!(">>> 当前时间: {}", future_time);

    if timelock_3m.is_mature(future_time, 0) {
        println!("✅ 时间锁已到期！");
        println!("✅ 用户可以支取储蓄");
        println!("💰 支取金额: {} satoshi\n", savings_balance);
    }

    println!("========================================");
    println!("   时间锁优势");
    println!("========================================");
    println!("✅ 强制储蓄: 防止冲动消费");
    println!("✅ 资金安全: 区块链保护");
    println!("✅ 自动执行: 到期自动解锁");
    println!("✅ 透明可查: 所有人可验证");
    println!("========================================\n");

    println!("💡 实际应用场景:");
    println!("  • 定期存款产品");
    println!("  • 养老金账户");
    println!("  • 子女教育基金");
    println!("  • 项目资金锁定");
    println!("  • 代币解锁计划");
}

// 辅助函数：格式化时间
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{} 秒", seconds)
    } else if seconds < 3600 {
        format!("{} 分钟", seconds / 60)
    } else if seconds < 86400 {
        format!("{} 小时", seconds / 3600)
    } else {
        format!("{} 天", seconds / 86400)
    }
}
