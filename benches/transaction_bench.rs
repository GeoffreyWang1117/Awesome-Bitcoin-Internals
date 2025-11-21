//! 交易处理性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn bench_transaction_creation(c: &mut Criterion) {
    c.benchmark_group("transaction")
        .bench_function("create_transaction", |b| {
            let mut blockchain = Blockchain::new();
            let wallet = Wallet::new();

            // 给钱包初始余额
            let genesis = Wallet::from_address("genesis".to_string());
            let init_tx = blockchain
                .create_transaction(&genesis, wallet.address.clone(), 100000, 0)
                .unwrap();
            blockchain.add_transaction(init_tx).unwrap();
            blockchain
                .mine_pending_transactions(wallet.address.clone())
                .unwrap();

            b.iter(|| {
                let tx = black_box(
                    blockchain
                        .create_transaction(&wallet, "target_address".to_string(), 100, 10)
                        .unwrap(),
                );
                black_box(tx);
            });
        });
}

fn bench_transaction_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification");

    // 预先创建交易
    let mut blockchain = Blockchain::new();
    let wallet = Wallet::new();

    let genesis = Wallet::from_address("genesis".to_string());
    let init_tx = blockchain
        .create_transaction(&genesis, wallet.address.clone(), 100000, 0)
        .unwrap();
    blockchain.add_transaction(init_tx).unwrap();
    blockchain
        .mine_pending_transactions(wallet.address.clone())
        .unwrap();

    let tx = blockchain
        .create_transaction(&wallet, "target".to_string(), 100, 10)
        .unwrap();

    group.bench_function("verify_transaction", |b| {
        b.iter(|| {
            // 验证交易签名和UTXO
            black_box(tx.verify_signature(&wallet.public_key));
        });
    });

    group.finish();
}

fn bench_batch_transactions(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_add", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut blockchain = Blockchain::new();
                    let wallet = Wallet::new();

                    // 初始化余额
                    let genesis = Wallet::from_address("genesis".to_string());
                    let init_tx = blockchain
                        .create_transaction(&genesis, wallet.address.clone(), 1000000, 0)
                        .unwrap();
                    blockchain.add_transaction(init_tx).unwrap();
                    blockchain
                        .mine_pending_transactions(wallet.address.clone())
                        .unwrap();

                    // 创建并添加多笔交易
                    for i in 0..size {
                        let tx = blockchain
                            .create_transaction(
                                &wallet,
                                format!("address_{}", i),
                                100,
                                1,
                            )
                            .unwrap();
                        black_box(blockchain.add_transaction(tx).unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_balance_query(c: &mut Criterion) {
    // 预先创建有许多交易的区块链
    let mut blockchain = Blockchain::new();
    let wallet = Wallet::new();

    let genesis = Wallet::from_address("genesis".to_string());
    for i in 0..100 {
        let tx = blockchain
            .create_transaction(&genesis, wallet.address.clone(), 1000, 0)
            .unwrap();
        blockchain.add_transaction(tx).unwrap();
        if i % 10 == 0 {
            blockchain
                .mine_pending_transactions(wallet.address.clone())
                .unwrap();
        }
    }

    c.benchmark_group("query").bench_function("get_balance", |b| {
        b.iter(|| {
            black_box(blockchain.get_balance(&wallet.address));
        });
    });
}

criterion_group!(
    benches,
    bench_transaction_creation,
    bench_transaction_verification,
    bench_batch_transactions,
    bench_balance_query
);
criterion_main!(benches);
