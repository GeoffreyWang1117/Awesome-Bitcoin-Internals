# 高级模块

SimpleBTC 的高级模块在核心区块链功能之上构建了一套完整的比特币协议特性实现。这些模块相互协作，覆盖了从数据完整性验证到复杂多方签名、从轻量级支付验证到脚本语言执行的完整功能栈。

---

## 模块概览

| 模块 | 源文件 | 核心功能 |
|------|--------|----------|
| [Merkle 树](merkle.md) | `src/merkle.rs` | 数据完整性验证、SPV 证明生成与验证 |
| [多重签名](multisig.md) | `src/multisig.rs` | M-of-N 多方签名地址与交易构建 |
| [高级交易](advanced-tx.md) | `src/advanced_tx.rs` | RBF 替换机制、时间锁、手续费估算 |
| 内存池 | `src/mempool.rs` | 未确认交易管理与优先级排序 |
| 脚本引擎 | `src/script.rs` | Bitcoin Script 子集解释执行 |
| SPV | `src/spv.rs` | 轻量级支付验证客户端 |

---

## Merkle 树

**源文件：** `src/merkle.rs` | **文档：** [Merkle API](merkle.md)

Merkle 树是区块链数据完整性的基础。SimpleBTC 使用 SHA256 构建二叉哈希树，将区块内所有交易哈希汇聚为单个 32 字节的 `merkle_root` 存储在区块头中。

核心价值在于支持 **SPV（简化支付验证）**：轻钱包无需下载完整区块（1-2 MB），只需获取区块头（80 字节）和 O(log n) 条哈希路径，即可以密码学方式证明某笔交易已被打包确认。

```rust
use simplebtc::merkle::MerkleTree;

let tx_ids = vec!["tx1_hash".to_string(), "tx2_hash".to_string()];
let tree = MerkleTree::new(&tx_ids);
let root = tree.get_root_hash();

// 生成并验证 SPV 证明
let proof = tree.get_proof("tx1_hash").unwrap();
let valid = MerkleTree::verify_proof("tx1_hash", &proof, &root, 0);
```

Merkle 树被 `Block::new()` 内部调用以计算 `merkle_root`，也被 `Block::verify_transaction_inclusion()` 用于 SPV 验证。

---

## 多重签名

**源文件：** `src/multisig.rs` | **文档：** [MultiSig API](multisig.md)

多重签名（MultiSig）实现了 Bitcoin 的 M-of-N 签名方案：N 个参与方各持一个 ECDSA 密钥对，需要其中至少 M 个人签名才能动用资金。这是比特币协议中实现分布式控制与风险分散的核心机制。

典型应用场景包括：2-of-3 企业资金管理（防止单人挪用）、2-of-3 第三方托管（买家-卖家-仲裁员）、个人多设备备份（主密钥丢失仍可恢复）。

```rust
use simplebtc::multisig::{MultiSigAddress, MultiSigTxBuilder};
use simplebtc::wallet::Wallet;

// 创建 2-of-3 多签地址
let (w1, w2, w3) = (Wallet::new(), Wallet::new(), Wallet::new());
let pub_keys = vec![w1.public_key.clone(), w2.public_key.clone(), w3.public_key.clone()];
let ms_addr = MultiSigAddress::new(2, pub_keys).unwrap();

// 收集签名（任意两人签名即可）
let mut builder = MultiSigTxBuilder::new(ms_addr);
builder.add_signature(&w1, "交易数据").unwrap();
builder.add_signature(&w2, "交易数据").unwrap();
assert!(builder.is_complete());
```

多签地址以 `"3"` 开头（对应比特币的 P2SH 地址格式），通过脚本哈希生成，最多支持 15 个参与密钥。

---

## 高级交易

**源文件：** `src/advanced_tx.rs` | **文档：** [Advanced TX API](advanced-tx.md)

高级交易模块提供三项关键功能，解决比特币网络中的实际工程问题：

**RBF（Replace-By-Fee）：** 允许用户用更高手续费的新交易替换尚未确认的旧交易，从而加速确认或取消错误交易。`RBFManager` 维护可替换交易列表并执行替换验证规则（输入相同、手续费更高、增量满足最低要求）。

**TimeLock（时间锁）：** 限制交易在指定时间或区块高度之前无法被矿工打包。支持两种类型：基于 Unix 时间戳（`new_time_based`）和基于区块高度（`new_height_based`）。常用于定期存款、遗产继承、智能合约等场景。

**TxPriorityCalculator（手续费计算器）：** 根据交易大小和紧急程度（`Low`/`Medium`/`High`/`Urgent`）推荐合理手续费，并计算综合优先级分数（70% 费率权重 + 30% 优先级权重）。

```rust
use simplebtc::advanced_tx::{AdvancedTxBuilder, TimeLock, RBFManager, TxPriorityCalculator, FeeUrgency};

// RBF + 时间锁组合使用
let timelock = TimeLock::new_height_based(850_000);
let builder = AdvancedTxBuilder::new()
    .with_rbf()
    .with_timelock(timelock);

// 推荐手续费
let fee = TxPriorityCalculator::recommend_fee(250, FeeUrgency::High); // 250 字节交易
println!("推荐手续费: {} satoshi", fee); // ~5000 satoshi
```

---

## 内存池（Mempool）

**源文件：** `src/mempool.rs`

内存池（Memory Pool，简称 Mempool）存储所有已广播但尚未打包进区块的交易。矿工从内存池中按优先级选取交易来构建新区块。

主要职责：
- 接收并暂存广播的交易
- 按手续费率排序，为矿工提供高价值交易优先选择
- 检测并拒绝双花（Double Spend）尝试
- 配合 RBF 机制处理交易替换
- 区块确认后从池中移除已打包交易

```rust
use simplebtc::mempool::Mempool;

let mut pool = Mempool::new();
pool.add_transaction(tx);
let pending = pool.get_pending_transactions(10); // 获取手续费最高的 10 笔
```

---

## 脚本引擎（Script）

**源文件：** `src/script.rs`

Bitcoin Script 是一种基于栈的简单脚本语言，用于定义交易的花费条件。SimpleBTC 实现了 Script 的核心子集，支持最常见的交易类型。

支持的脚本类型：
- **P2PKH**（Pay-to-Public-Key-Hash）：最常见的普通地址交易，锁定脚本格式为 `OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`。
- **P2SH**（Pay-to-Script-Hash）：多签和复杂合约的基础，以 `"3"` 开头的地址。
- **OP_RETURN**：在链上写入不可花费的任意数据（最多 80 字节）。

脚本引擎为多重签名模块提供底层支撑：`MultiSigAddress` 的 `script` 字段存储的就是简化版的 Script 锁定脚本。

---

## SPV（简化支付验证）

**源文件：** `src/spv.rs`

SPV 模拟比特币轻钱包的工作方式：在不下载完整区块链的情况下验证交易的合法性。这对于资源受限的设备（手机、嵌入式系统）至关重要。

SPV 验证流程：
1. 仅下载区块头（每个约 80 字节，所有区块头约 60 MB）
2. 验证区块头的工作量证明（PoW）
3. 请求目标交易的 Merkle 证明（约几百字节）
4. 本地执行 `MerkleTree::verify_proof()` 验证

```
全节点模式：下载全部区块链 (~500 GB) → 本地完整验证
SPV 模式：  下载区块头 (~60 MB) + Merkle 证明 (几 KB) → O(log n) 验证
```

SPV 模块与 Merkle 模块深度集成，依赖 `MerkleTree::get_proof()` 和 `MerkleTree::verify_proof()` 实现轻量验证。

---

## 模块依赖关系

```
核心模块
├── Transaction (src/transaction.rs)
├── Wallet      (src/wallet.rs)
└── Block       (src/block.rs)
        │
        ▼
高级模块（构建在核心模块之上）
├── Merkle      ← Block 内部使用（计算 merkle_root 和 SPV 证明）
├── MultiSig    ← 依赖 Wallet（ECDSA 签名）+ Script（锁定脚本）
├── AdvancedTx  ← 依赖 Transaction（RBF 替换验证）
├── Mempool     ← 依赖 Transaction + AdvancedTx（RBF 支持）
├── Script      ← MultiSig 和 SPV 的基础
└── SPV         ← 依赖 Merkle（证明验证）+ Block（区块头）
```

---

## 快速导航

- [Merkle API](merkle.md) — Merkle 树数据结构与 SPV 证明
- [MultiSig API](multisig.md) — M-of-N 多重签名
- [Advanced TX API](advanced-tx.md) — RBF、时间锁、手续费计算
- [Block API](block.md) — 区块结构与挖矿
- [Transaction API](transaction.md) — 交易构建与验证
- [Wallet API](wallet.md) — 钱包与密钥管理
