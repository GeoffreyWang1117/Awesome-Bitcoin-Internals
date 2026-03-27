//! 挖矿性能基准测试

use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_mining(c: &mut Criterion) {
    let mut group = c.benchmark_group("mining");

    // 测试不同难度的挖矿性能
    for difficulty in [2, 3, 4].iter() {
        group.bench_with_input(
            BenchmarkId::new("difficulty", difficulty),
            difficulty,
            |b, &diff| {
                b.iter(|| {
                    let mut blockchain = Blockchain::new();
                    blockchain.difficulty = diff;

                    let wallet = Wallet::new();
                    let tx = blockchain
                        .create_transaction(&wallet, wallet.address.clone(), 100, 10)
                        .unwrap();
                    blockchain.add_transaction(tx).unwrap();

                    blockchain
                        .mine_pending_transactions(wallet.address)
                        .unwrap();
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

fn bench_block_validation(c: &mut Criterion) {
    c.benchmark_group("validation")
        .bench_function("validate_chain_10_blocks", |b| {
            // 预先创建包含10个区块的链
            let mut blockchain = Blockchain::new();
            let wallet = Wallet::new();

            for _ in 0..10 {
                let tx = blockchain
                    .create_transaction(&wallet, wallet.address.clone(), 100, 10)
                    .unwrap();
                blockchain.add_transaction(tx).unwrap();
                blockchain
                    .mine_pending_transactions(wallet.address.clone())
                    .unwrap();
            }

            b.iter(|| {
                black_box(blockchain.is_valid());
            });
        });
}

criterion_group!(benches, bench_mining, bench_block_validation);
criterion_main!(benches);
