# SimpleBTC - 比特币区块链教育项目

<div align="center">

**一个功能完整的比特币区块链实现，用于学习和教育**

使用 Rust 构建 | 完整中文文档 | 生产级代码质量

[![Version](https://img.shields.io/badge/version-1.0.0-blue)](https://github.com/GeoffreyWang1117/SimpleBTC)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)
[![Docs](https://img.shields.io/badge/docs-完整文档-brightgreen)](./docs/src/introduction/README.md)

[快速开始](#快速开始) • [完整文档](./docs/src/introduction/README.md) • [API参考](./docs/src/api/core.md) • [示例代码](./examples/)

</div>

---

## ✨ 项目亮点

- 📚 **15,000+字完整中文文档** - 从入门到精通的系统教程
- 🎯 **真实比特币特性** - Merkle树、多签、RBF、TimeLock等高级功能
- 💻 **三种交互方式** - 命令行、REST API、Electron GUI
- 🔧 **生产级代码** - 完整注释、类型安全、错误处理
- 📖 **实战案例** - 企业多签、托管服务、定期存款等场景
- 🚀 **开箱即用** - 5分钟快速开始，零配置运行

---

## 📖 目录

- [核心特性](#核心特性)
- [快速开始](#快速开始)
- [完整文档](#完整文档)
- [项目结构](#项目结构)
- [学习路径](#学习路径)
- [示例代码](#示例代码)
- [技术栈](#技术栈)
- [贡献指南](#贡献指南)

## 🎯 核心特性

### 区块链基础

<table>
<tr>
<td width="50%">

**UTXO模型**
- ✅ 比特币原生的未花费交易输出模型
- ✅ 完整的输入输出验证
- ✅ 防双花攻击
- 📚 [UTXO API文档](./docs/src/api/utxo.md)

**工作量证明 (PoW)**
- ✅ SHA256哈希算法
- ✅ 可配置挖矿难度
- ✅ Nonce搜索机制
- 📚 [区块链API](./docs/src/api/blockchain.md)

</td>
<td width="50%">

**钱包系统**
- ✅ 密钥对生成
- ✅ 地址管理
- ✅ 交易签名与验证
- 📚 [钱包API文档](./docs/src/api/wallet.md)

**交易处理**
- ✅ 转账交易
- ✅ Coinbase交易（挖矿奖励）
- ✅ 手续费机制
- 📚 [交易API文档](./docs/src/api/transaction.md)

</td>
</tr>
</table>

### 🚀 真实比特币高级特性

SimpleBTC实现了真实比特币的核心高级功能：

| 特性 | 说明 | 应用场景 | 文档 |
|------|------|----------|------|
| 🌳 **Merkle树** | 高效交易验证，SPV支持 | 轻钱包、快速验证 | [教程](./docs/src/advanced/merkle.md) |
| 🔐 **多重签名** | M-of-N多签钱包 | 企业财务、托管服务 | [教程](./docs/src/advanced/multisig.md) |
| 🔄 **RBF** | Replace-By-Fee交易替换 | 加速确认、取消交易 | [教程](./docs/src/advanced/rbf.md) |
| ⏰ **TimeLock** | 时间锁定交易 | 定期存款、遗产继承 | [教程](./docs/src/advanced/timelock.md) |
| 📊 **优先级算法** | 综合费率和币龄 | 交易排序优化 | [API参考](./docs/src/api/transaction.md) |

### 💼 ACID事务特性

SimpleBTC完全符合数据库ACID特性，保证交易的可靠性：

| 特性 | 实现 | 保证 |
|------|------|------|
| **A**tomicity 原子性 | 交易全部成功或全部失败 | 无部分执行 |
| **C**onsistency 一致性 | 输入≥输出，余额守恒 | 无凭空造币 |
| **I**solation 隔离性 | 待确认池隔离 | 无脏读脏写 |
| **D**urability 持久性 | 区块链永久存储 | 不可篡改 |

📚 **深入阅读**: [核心概念详解](./docs/src/guide/concepts.md)

## 📁 项目结构

```
SimpleBTC/
├── src/                      # 核心源代码
│   ├── lib.rs               # 库入口，导出所有模块
│   ├── block.rs             # 区块结构和PoW实现
│   ├── blockchain.rs        # 区块链主逻辑
│   ├── transaction.rs       # 交易和签名
│   ├── wallet.rs            # 钱包和密钥管理
│   ├── utxo.rs              # UTXO集合管理
│   ├── merkle.rs            # Merkle树实现
│   ├── multisig.rs          # 多重签名
│   ├── advanced_tx.rs       # RBF和TimeLock
│   ├── persistence.rs       # 数据持久化
│   └── bin/
│       ├── demo.rs          # 命令行演示程序
│       └── server.rs        # REST API服务器
│
├── examples/                # 实战示例代码
│   ├── enterprise_multisig.rs   # 企业多签钱包
│   ├── escrow_service.rs        # 托管服务系统
│   └── timelock_savings.rs      # 定期存款系统
│
├── docs/                    # 完整文档（15,000+字）
│   ├── book.toml            # mdBook配置
│   └── src/
│       ├── introduction/    # 项目介绍
│       ├── guide/           # 入门指南
│       ├── api/             # API参考文档
│       ├── advanced/        # 高级特性教程
│       ├── examples/        # 案例详解
│       └── appendix/        # 附录（FAQ、术语表）
│
├── frontend/                # Electron GUI（可选）
│   ├── index.html
│   ├── renderer.js
│   └── package.json
│
├── Cargo.toml               # Rust项目配置
├── README.md                # 本文件
└── ADVANCED_FEATURES.md     # 高级特性快速参考
```

## 🚀 快速开始

### 环境要求

- **Rust**: 1.70 或更高版本（[安装指南](https://www.rust-lang.org/tools/install)）
- **Cargo**: Rust包管理器（通常随Rust一起安装）
- **Node.js**: 16+ （仅GUI需要）

### 5分钟快速体验

#### 方式1: 命令行演示（推荐入门）

```bash
# 克隆项目
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC

# 运行演示
cargo run --bin demo --release
```

**你将看到**：完整的区块链操作演示，包括创建钱包、转账、挖矿、验证等。

#### 方式2: REST API服务器

```bash
# 启动API服务器
cargo run --bin server --release

# 在另一个终端测试API
curl http://127.0.0.1:3000/api/blockchain/info
```

**适用于**: Web开发者、API集成测试

📚 **完整API文档**: [REST API参考](./docs/src/api/rest-api.md)

#### 方式3: Electron GUI（可视化）

```bash
# 1. 启动API服务器
cargo run --bin server --release

# 2. 在新终端启动GUI
cd frontend
npm install
npm start
```

**适用于**: 图形化演示、非技术用户

### 示例代码运行

```bash
# 企业多签钱包
cargo run --example enterprise_multisig

# 托管服务系统
cargo run --example escrow_service

# 定期存款系统
cargo run --example timelock_savings
```

## 📚 完整文档

SimpleBTC提供15,000+字的系统化中文文档，涵盖从入门到精通的所有内容。

### 文档导航

| 分类 | 内容 | 链接 |
|------|------|------|
| 📖 **入门指南** | 安装、快速开始、核心概念 | [查看](./docs/src/guide/installation.md) |
| 📘 **API参考** | 完整的API文档和使用示例 | [查看](./docs/src/api/core.md) |
| 🎓 **高级教程** | Merkle树、多签、RBF、TimeLock | [查看](./docs/src/advanced/multisig.md) |
| 💼 **实战案例** | 企业多签、托管服务、定期存款 | [查看](./docs/src/examples/enterprise-multisig.md) |
| 🌐 **REST API** | HTTP API完整参考 | [查看](./docs/src/api/rest-api.md) |
| ❓ **FAQ** | 常见问题解答 | [查看](./docs/src/appendix/faq.md) |

### 浏览文档网站

```bash
# 安装mdBook
cargo install mdbook

# 启动文档服务器
cd docs
mdbook serve --open
```

然后访问 `http://localhost:3000` 浏览完整文档。

## 🎓 学习路径

### 路径1: 区块链初学者（1-2周）

```
1. 阅读项目介绍 → docs/src/introduction/README.md
2. 理解核心概念 → docs/src/guide/concepts.md
3. 运行命令行演示 → cargo run --bin demo
4. 学习基础API → docs/src/api/blockchain.md
5. 完成简单练习 → 创建钱包、转账、查余额
```

### 路径2: 比特币开发者（2-3周）

```
1. 深入UTXO模型 → docs/src/api/utxo.md
2. 学习高级特性 → 多签、RBF、TimeLock
3. 研究实战案例 → examples/目录
4. 集成REST API → docs/src/api/rest-api.md
5. 构建自己的应用 → 基于SimpleBTC开发
```

### 路径3: 源码贡献者（3-4周）

```
1. 熟悉项目架构 → src/目录结构
2. 阅读核心代码 → blockchain.rs, transaction.rs
3. 理解PoW实现 → block.rs
4. 学习测试方法 → cargo test
5. 提交PR改进 → 参考CONTRIBUTING.md
```

## 💡 示例代码

### Rust API基础用法

```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

fn main() -> Result<(), String> {
    // 1. 创建区块链
    let mut blockchain = Blockchain::new();

    // 2. 创建钱包
    let alice = Wallet::new();
    let bob = Wallet::new();

    // 3. 创建交易
    let tx = blockchain.create_transaction(
        &alice,
        bob.address.clone(),
        5000,  // 金额
        10,    // 手续费
    )?;

    // 4. 添加到交易池
    blockchain.add_transaction(tx)?;

    // 5. 挖矿确认
    blockchain.mine_pending_transactions(alice.address.clone())?;

    // 6. 查询余额
    println!("Alice: {}", blockchain.get_balance(&alice.address));
    println!("Bob: {}", blockchain.get_balance(&bob.address));

    Ok(())
}
```

### REST API基础用法

```bash
# 创建钱包
curl -X POST http://127.0.0.1:3000/api/wallet/create

# 查询余额
curl http://127.0.0.1:3000/api/wallet/balance/ADDRESS

# 创建交易
curl -X POST http://127.0.0.1:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{"from_address":"ADDR1","to_address":"ADDR2","amount":5000,"fee":10}'

# 挖矿
curl -X POST http://127.0.0.1:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address":"ADDRESS"}'
```

📚 **更多示例**: 查看 [examples/](./examples/) 目录

## 🛠️ 技术栈

<table>
<tr>
<td width="50%">

### 后端 (Rust)

- **核心语言**: Rust 2021 Edition
- **Web框架**: Axum (高性能异步)
- **运行时**: Tokio (异步runtime)
- **加密**: SHA2 (哈希算法)
- **序列化**: Serde (JSON支持)
- **测试**: 单元测试 + 集成测试

</td>
<td width="50%">

### 前端 (可选)

- **桌面**: Electron
- **界面**: 原生HTML/CSS/JavaScript
- **样式**: CSS3 Grid/Flexbox
- **通信**: Fetch API

### 文档

- **生成器**: mdBook
- **格式**: Markdown

</td>
</tr>
</table>

## 🆚 与真实比特币的对比

| 特性 | SimpleBTC | 真实比特币 | 说明 |
|------|-----------|------------|------|
| **UTXO模型** | ✅ | ✅ | 完全一致 |
| **工作量证明** | ✅ SHA256 | ✅ Double SHA256 | 原理相同 |
| **Merkle树** | ✅ | ✅ | 完整实现 |
| **多重签名** | ✅ M-of-N | ✅ P2SH/P2WSH | 简化实现 |
| **RBF** | ✅ | ✅ BIP125 | 核心功能 |
| **TimeLock** | ✅ nLockTime | ✅ CLTV/CSV | 基础实现 |
| **签名算法** | ⚠️ SHA256 | ✅ ECDSA/Schnorr | 简化版 |
| **地址格式** | ⚠️ 十六进制 | ✅ Base58/Bech32 | 教学简化 |
| **P2P网络** | ❌ | ✅ | 未实现 |
| **脚本系统** | ❌ | ✅ Script | 未实现 |

✅ = 已实现 | ⚠️ = 简化实现 | ❌ = 未实现

## ⚠️ 重要提示

**SimpleBTC是教育演示项目，不适用于生产环境。**

### 简化之处

- 🔐 **密钥生成**: 使用简化的SHA256，非真实的secp256k1椭圆曲线
- 🌐 **网络层**: 无P2P网络通信和节点同步
- 💾 **持久化**: 基础文件存储，非生产级数据库
- 🔒 **安全性**: 演示级别，未经安全审计

### 学习价值

尽管有简化，SimpleBTC仍然是**优秀的学习资源**：
- ✅ 核心概念与真实比特币一致
- ✅ 代码清晰易懂，注释完整
- ✅ 可运行、可调试、可修改
- ✅ 适合理解区块链原理

## 🤝 贡献指南

欢迎贡献！以下是一些建议：

### 可以改进的方向

1. **安全增强**
   - [ ] 实现真正的ECDSA签名
   - [ ] 添加地址校验和
   - [ ] 改进密钥存储机制

2. **功能扩展**
   - [ ] P2P网络通信
   - [ ] 脚本系统 (Script)
   - [ ] SegWit支持
   - [ ] 闪电网络Layer2

3. **性能优化**
   - [ ] 数据库索引优化
   - [ ] 并行验证交易
   - [ ] UTXO缓存策略

4. **文档完善**
   - [ ] 英文文档翻译
   - [ ] 视频教程
   - [ ] 更多实战案例

### 提交流程

```bash
# 1. Fork项目
# 2. 创建分支
git checkout -b feature/your-feature

# 3. 提交更改
git commit -m "Add: your feature description"

# 4. 推送分支
git push origin feature/your-feature

# 5. 提交Pull Request
```

详见: [贡献指南](./docs/src/appendix/contributing.md)

## 📄 许可证

本项目采用 **MIT License** 开源协议。

- ✅ 商业使用
- ✅ 修改
- ✅ 分发
- ✅ 私有使用

查看完整许可证: [LICENSE](./LICENSE)

## 📞 联系方式

- **项目主页**: [GitHub Repository](https://github.com/GeoffreyWang1117/SimpleBTC)
- **问题反馈**: [Issues](https://github.com/GeoffreyWang1117/SimpleBTC/issues)
- **讨论交流**: [Discussions](https://github.com/GeoffreyWang1117/SimpleBTC/discussions)

## 🙏 致谢

本项目受益于以下资源：

- [比特币白皮书](https://bitcoin.org/bitcoin.pdf) - Satoshi Nakamoto
- [精通比特币](https://github.com/bitcoinbook/bitcoinbook) - Andreas M. Antonopoulos
- [Rust程序设计语言](https://doc.rust-lang.org/book/) - Steve Klabnik & Carol Nichols
- [比特币开发者文档](https://developer.bitcoin.org/)

## ⭐ Star历史

如果这个项目对你有帮助，请给一个Star⭐️支持一下！

---

<div align="center">

**SimpleBTC** - 学习比特币的最佳起点

Made with ❤️ by the SimpleBTC Team

[⬆ 回到顶部](#simplebtc---比特币区块链教育项目)

</div>
