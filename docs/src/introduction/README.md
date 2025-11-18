# SimpleBTC - 基于比特币的银行系统Demo

欢迎来到SimpleBTC项目文档！

## 项目简介

SimpleBTC是一个用Rust实现的比特币银行系统演示项目，完整实现了比特币的核心原理和高级特性。本项目不仅是一个学习工具，也是一个功能完备的区块链系统demo。

### 核心特性

#### 🔐 完整的UTXO模型
- 未花费交易输出（UTXO）管理
- 防双花机制
- 余额计算
- UTXO选择算法

#### ⛓️ 区块链核心功能
- 工作量证明（Proof of Work）
- 区块链验证
- Merkle树实现
- 链式哈希结构

#### 💼 钱包系统
- 密钥对生成（简化版）
- 地址生成
- 数字签名
- 交易创建

#### 📊 高级交易特性
- **Replace-By-Fee (RBF)**: 替换未确认交易，加速确认
- **时间锁（TimeLock）**: 定期存款、遗产继承
- **多重签名（MultiSig）**: 2-of-3企业钱包、托管服务
- **交易优先级**: 基于手续费的优先排序

#### 🌳 Merkle树与SPV
- 高效的交易验证
- 轻量级客户端支持
- Merkle证明生成与验证

#### 🔧 工程特性
- REST API服务器（Axum框架）
- 持久化存储（JSON）
- 交易索引器
- Electron可视化界面

### 为什么选择SimpleBTC？

1. **教育价值**
   - 深入理解比特币原理
   - 学习Rust区块链开发
   - 掌握密码学基础知识

2. **完整实现**
   - 符合ACID事务特性
   - 实现比特币核心协议
   - 包含高级BIP特性

3. **实战案例**
   - 企业资金管理
   - 托管交易服务
   - 定期存款系统

4. **易于扩展**
   - 模块化设计
   - 清晰的代码结构
   - 详细的中文注释

## 快速开始

```bash
# 克隆项目
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC

# 编译项目
cargo build --release

# 运行Demo
cargo run --bin btc-demo

# 运行REST API服务器
cargo run --bin btc-server

# 运行示例
cargo run --example enterprise_multisig
cargo run --example escrow_service
cargo run --example timelock_savings
```

## 系统架构

```
SimpleBTC/
├── src/
│   ├── transaction.rs     # 交易模块（UTXO模型）
│   ├── block.rs          # 区块结构
│   ├── blockchain.rs     # 区块链核心逻辑
│   ├── wallet.rs         # 钱包管理
│   ├── utxo.rs          # UTXO集合管理
│   ├── merkle.rs        # Merkle树实现
│   ├── multisig.rs      # 多重签名
│   ├── advanced_tx.rs   # RBF、时间锁、优先级
│   ├── persistence.rs   # 持久化存储
│   └── indexer.rs       # 交易索引
├── examples/            # 实战案例
├── frontend/            # Electron GUI
└── docs/               # 本文档
```

## 技术栈

- **语言**: Rust (Edition 2021)
- **核心库**:
  - sha2 - SHA256哈希
  - serde - 序列化
  - rand - 随机数生成
- **Web框架**: Axum (异步REST API)
- **前端**: Electron + JavaScript
- **文档**: mdBook

## 学习路径

### 初级：理解基础概念
1. [基本概念](../guide/concepts.md) - UTXO、区块、哈希
2. [钱包管理](../guide/wallet.md) - 创建钱包、发送交易
3. [交易处理](../guide/transactions.md) - 交易结构、验证

### 中级：掌握核心机制
1. [区块链操作](../guide/blockchain.md) - 挖矿、验证
2. [UTXO管理](../guide/utxo.md) - UTXO选择、双花防护
3. [Merkle树](../advanced/merkle.md) - SPV验证

### 高级：实现复杂应用
1. [多重签名](../advanced/multisig.md) - 企业钱包
2. [时间锁](../advanced/timelock.md) - 定期存款
3. [RBF机制](../advanced/rbf.md) - 交易加速

## 与比特币的差异

SimpleBTC是教育性质的简化实现，与真实比特币的主要差异：

| 特性 | SimpleBTC | 真实比特币 |
|------|-----------|------------|
| 密码学 | 简化的SHA256 | secp256k1椭圆曲线 |
| 签名 | 简化验证 | ECDSA签名 |
| 脚本 | 简化脚本 | 完整Script语言 |
| P2P网络 | 无网络层 | 完整P2P协议 |
| 存储 | JSON文件 | LevelDB数据库 |
| 难度调整 | 固定难度 | 动态难度调整 |

## 项目状态

- ✅ UTXO模型
- ✅ 工作量证明
- ✅ Merkle树
- ✅ 多重签名
- ✅ RBF机制
- ✅ 时间锁
- ✅ REST API
- ✅ GUI界面
- ✅ 完整文档

## 贡献

欢迎贡献代码、文档或报告问题！

详见[贡献指南](../appendix/contributing.md)

## 许可证

本项目采用MIT许可证

---

让我们开始探索比特币的世界吧！ 🚀
