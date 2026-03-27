# 架构概览

SimpleBTC 是一个功能完整的比特币区块链教育实现，用 Rust 编写。本章介绍项目的整体架构设计、模块组织方式、核心数据流以及关键抽象层。

---

## 项目结构

```
SimpleBTC/
├── src/
│   ├── lib.rs               # Crate 入口，模块导出与重新导出
│   ├── main.rs              # 可执行入口（演示 / 交互式 CLI）
│   │
│   │── ── 核心层 (Core) ──
│   ├── block.rs             # 区块结构 + 工作量证明
│   ├── blockchain.rs        # 区块链核心逻辑（UTXO 管理、挖矿、验证）
│   ├── transaction.rs       # 交易结构（TxInput / TxOutput / Transaction）
│   ├── wallet.rs            # 钱包 + secp256k1 密钥管理
│   ├── utxo.rs              # UTXO 集合（未花费交易输出管理）
│   ├── crypto.rs            # 扩展密码学（Bech32、WIF 导出）
│   │
│   │── ── 高级特性层 (Advanced) ──
│   ├── merkle.rs            # Merkle 树（交易包含证明）
│   ├── multisig.rs          # 多重签名（M-of-N）
│   ├── advanced_tx.rs       # 高级交易（RBF、TimeLock）
│   ├── mempool.rs           # 内存池（按费率排序）
│   ├── script.rs            # Bitcoin Script 脚本系统
│   ├── spv.rs               # SPV 轻客户端验证
│   ├── parallel_mining.rs   # 多线程并行 PoW 挖矿
│   ├── network.rs           # P2P 网络层
│   │
│   │── ── 基础设施层 (Infrastructure) ──
│   ├── storage.rs           # RocksDB 高性能持久化存储
│   ├── persistence.rs       # 序列化 / 反序列化辅助
│   ├── config.rs            # 全局配置（难度、奖励等）
│   ├── logging.rs           # 结构化日志（tracing）
│   ├── security.rs          # 安全验证
│   ├── indexer.rs           # 交易索引器（加速地址查询）
│   └── error.rs             # 统一错误类型
│
├── docs/                    # mdBook 文档
├── Cargo.toml
└── README.md
```

---

## 三层架构模型

SimpleBTC 的模块按职责分为三个层次：

```
┌─────────────────────────────────────────────────────────┐
│                     核心层 (Core)                        │
│  block  blockchain  transaction  wallet  utxo  crypto   │
│  ─── 实现比特币的基本数据结构和协议规则 ───               │
├─────────────────────────────────────────────────────────┤
│                   高级特性层 (Advanced)                   │
│  merkle  multisig  advanced_tx  mempool  script  spv    │
│  parallel_mining  network                               │
│  ─── 实现比特币的高级功能与扩展协议 ───                   │
├─────────────────────────────────────────────────────────┤
│                  基础设施层 (Infrastructure)              │
│  storage  persistence  config  logging  security        │
│  indexer  error                                         │
│  ─── 提供存储、日志、配置等通用基础能力 ───               │
└─────────────────────────────────────────────────────────┘
```

### 核心层模块详解

| 模块 | 文件 | 职责 |
|------|------|------|
| `block` | `block.rs` | 定义 `Block` 结构体，包含区块头字段（index、timestamp、nonce、merkle_root、previous_hash、hash）以及单线程 `mine_block()` 方法 |
| `blockchain` | `blockchain.rs` | `Blockchain` 主结构体，统筹区块链全部操作：创世区块、交易创建、内存池管理、并行挖矿、UTXO 更新、链验证 |
| `transaction` | `transaction.rs` | `TxInput`、`TxOutput`、`Transaction` 三个核心结构体；Coinbase 交易构造；ECDSA 签名验证 |
| `wallet` | `wallet.rs` | `Wallet` 结构体，使用 secp256k1 生成真实密钥对，P2PKH 地址推导，ECDSA 签名与验证 |
| `utxo` | `utxo.rs` | `UTXOSet` 管理所有未花费交易输出，支持余额查询、可花费 UTXO 检索 |
| `crypto` | `crypto.rs` | `CryptoWallet` 扩展实现：Bech32 地址、WIF 私钥格式导入导出 |

### 高级特性层模块详解

| 模块 | 文件 | 职责 |
|------|------|------|
| `merkle` | `merkle.rs` | Merkle 树构建与 Merkle 证明生成/验证（SPV 的基础） |
| `multisig` | `multisig.rs` | M-of-N 多重签名方案 |
| `advanced_tx` | `advanced_tx.rs` | RBF（Replace-By-Fee）费用替换、TimeLock 时间锁交易 |
| `mempool` | `mempool.rs` | 内存池，按费率（satoshi/byte）优先级排序待确认交易 |
| `script` | `script.rs` | Bitcoin Script 操作码解释器 |
| `spv` | `spv.rs` | 简单支付验证，使用 Merkle 证明在不下载完整链的情况下验证交易 |
| `parallel_mining` | `parallel_mining.rs` | `ParallelMiner`：多线程 PoW，充分利用多核 CPU |
| `network` | `network.rs` | P2P 网络消息传播层 |

### 基础设施层模块详解

| 模块 | 文件 | 职责 |
|------|------|------|
| `storage` | `storage.rs` | 基于 RocksDB 的高性能键值存储 |
| `persistence` | `persistence.rs` | 区块链数据序列化与反序列化 |
| `config` | `config.rs` | 全局参数（挖矿难度、区块奖励、网络参数等） |
| `logging` | `logging.rs` | 结构化日志（基于 `tracing` crate） |
| `security` | `security.rs` | 额外的安全验证逻辑 |
| `indexer` | `indexer.rs` | `TransactionIndexer`：为地址 → 交易ID 建立索引，加速余额查询 |
| `error` | `error.rs` | `BitcoinError` 统一错误枚举，`Result<T>` 类型别名 |

---

## 核心数据流

### 完整的价值转移流程

```
用户创建交易请求
       │
       ▼
┌─────────────────────────────────────┐
│  Blockchain::create_transaction()   │
│  1. 在 UTXOSet 中查找可花费输出      │
│  2. 用 Wallet::sign() 对输入签名     │
│  3. 构造 TxInput + TxOutput         │
│  4. 生成 Transaction（含哈希 ID）    │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Blockchain::add_transaction()      │
│  1. Transaction::verify() 验证签名  │
│  2. 检查 UTXO 存在 + 余额足够       │
│  3. 记录 pending_spent 防双花       │
│  4. 添加到 Mempool（按费率排序）     │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Blockchain::mine_pending_transactions() │
│  1. 从 Mempool 取出高费率交易        │
│  2. 构造 Coinbase 交易（奖励+费用） │
│  3. Block::new() 计算 Merkle Root   │
│  4. ParallelMiner 多线程 PoW 挖矿   │
│  5. 验证区块中所有交易              │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  UTXO 集合原子更新                  │
│  1. 消费输入中引用的 UTXO           │
│  2. 将输出添加为新 UTXO             │
│  3. 清空 pending_spent              │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  区块上链                           │
│  1. indexer.index_block() 建索引    │
│  2. chain.push(block)               │
│  3. 从 Mempool 删除已确认交易       │
└─────────────────────────────────────┘
```

### 数据结构关系图

```
Blockchain
├── chain: Vec<Block>
│   └── Block
│       ├── index, timestamp, nonce
│       ├── previous_hash → 上一区块 hash（链式连接）
│       ├── merkle_root   → MerkleTree 计算
│       ├── hash          → SHA256(index+timestamp+merkle_root+prev+nonce)
│       └── transactions: Vec<Transaction>
│           └── Transaction
│               ├── id      → SHA256(交易内容)
│               ├── inputs: Vec<TxInput>
│               │   └── TxInput {txid, vout, signature, pub_key}
│               └── outputs: Vec<TxOutput>
│                   └── TxOutput {value, pub_key_hash}
│
├── utxo_set: UTXOSet   ← 快速余额查询与 UTXO 检索
├── mempool: Mempool    ← 待确认交易（按费率排序）
├── indexer: TransactionIndexer  ← 地址→交易索引
└── miner: ParallelMiner         ← 多线程 PoW
```

---

## 快速开始

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

// 1. 创建区块链（含创世区块，创世钱包获得 10M satoshi 初始资金）
let mut blockchain = Blockchain::new();

// 2. 获取预置资金的创世钱包 + 创建新用户钱包
let genesis = Blockchain::genesis_wallet();
let alice = Wallet::new();
let bob = Wallet::new();

// 3. 创建交易：genesis → alice，转账 1000 satoshi，手续费 10
let tx = blockchain.create_transaction(&genesis, alice.address.clone(), 1000, 10)?;
blockchain.add_transaction(tx)?;

// 4. 挖矿（alice 作为矿工地址接收奖励）
blockchain.mine_pending_transactions(alice.address.clone())?;

// 5. 查询余额
println!("Alice 余额: {} satoshi", blockchain.get_balance(&alice.address));

// 6. 验证整个链的完整性
assert!(blockchain.is_valid());
# Ok::<(), String>(())
```

---

## 密码学选型

| 算法 | 库 | 用途 |
|------|----|------|
| secp256k1 ECDSA | `secp256k1` crate | 私钥生成、交易签名、签名验证 |
| SHA-256 | `bitcoin_hashes` | 区块哈希、交易哈希、地址推导 |
| RIPEMD-160 | `ripemd` crate | 公钥哈希（P2PKH 地址中间步骤） |
| Base58Check | `bs58` crate | P2PKH 地址编码、WIF 私钥编码 |
| Bech32 | `bech32` crate | 原生隔离见证（SegWit）地址 |
| SHA-256d | `bitcoin_hashes` | 双重哈希（校验和计算） |

所有密码学实现均与比特币主网兼容——`Wallet::genesis()` 生成的地址可以在真实比特币协议中合法使用。

---

## 并发设计

挖矿（`ParallelMiner`）是项目中唯一大量使用多线程的模块。区块链状态本身（`Blockchain` 结构体）采用单线程所有权模型，通过 Rust 借用检查器在编译期保证数据安全，无需运行时锁开销。

```rust
// 并行挖矿：根据 CPU 核心数自动分配 nonce 搜索范围
self.miner
    .mine_block(&mut block, self.difficulty)
    .map_err(|e| format!("挖矿失败: {}", e))?;
```

---

## 错误处理

所有公共 API 返回 `Result<T, String>` 或 `crate::error::Result<T>`（即 `Result<T, BitcoinError>`）。`BitcoinError` 是统一的枚举类型，覆盖：

- `PrivateKeyError` — 密钥格式错误
- 余额不足、UTXO 不存在、签名验证失败等业务错误

```rust
use bitcoin_simulation::{BitcoinError, Result};

fn example() -> Result<()> {
    let blockchain = Blockchain::new();
    // ...
    Ok(())
}
```
