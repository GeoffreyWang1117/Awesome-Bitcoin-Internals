# 安装与配置

本章节将指导您完成SimpleBTC的安装和基本配置。

## 系统要求

### 最低要求
- **操作系统**: Linux, macOS, 或 Windows (WSL2)
- **Rust版本**: 1.70.0 或更高
- **内存**: 至少 2GB RAM
- **存储**: 至少 500MB 可用空间

### 推荐配置
- **操作系统**: Linux/macOS
- **Rust版本**: 最新稳定版
- **内存**: 4GB+ RAM
- **存储**: 1GB+ 可用空间
- **CPU**: 多核处理器（挖矿性能更好）

## 安装Rust

如果您还没有安装Rust，请访问[rust-lang.org](https://www.rust-lang.org/)或使用以下命令：

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

## 克隆项目

```bash
# 使用HTTPS克隆
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git

# 或使用SSH
git clone git@github.com:GeoffreyWang1117/SimpleBTC.git

# 进入项目目录
cd SimpleBTC
```

## 编译项目

### 开发模式编译

```bash
# 快速编译（未优化，编译快）
cargo build

# 运行测试
cargo test

# 运行Demo
cargo run --bin btc-demo
```

### 生产模式编译

```bash
# 优化编译（性能最佳，编译慢）
cargo build --release

# 运行优化后的程序
./target/release/btc-demo
./target/release/btc-server
```

## 运行示例

SimpleBTC提供了三个实战示例：

```bash
# 1. 企业多签钱包（2-of-3）
cargo run --example enterprise_multisig

# 2. 托管服务（买家/卖家/仲裁员）
cargo run --example escrow_service

# 3. 定期存款（时间锁）
cargo run --example timelock_savings
```

## 启动REST API服务器

```bash
# 开发模式
cargo run --bin btc-server

# 生产模式
cargo run --release --bin btc-server
```

服务器将在 `http://localhost:3000` 启动

### API端点

- `GET /api/blockchain/info` - 获取区块链信息
- `POST /api/wallet/create` - 创建新钱包
- `POST /api/transaction/create` - 创建交易
- `POST /api/mine` - 挖矿
- `GET /api/balance/:address` - 查询余额

## 启动Electron GUI

```bash
# 安装Node.js依赖
cd frontend
npm install

# 启动Electron应用
npm start
```

GUI提供了可视化界面，包括：
- 区块链浏览器
- 钱包管理
- 交易创建
- 实时挖矿
- 一键Demo模式

## 项目结构

```
SimpleBTC/
├── src/                    # 源代码
│   ├── lib.rs             # 库入口
│   ├── main.rs            # CLI Demo
│   ├── transaction.rs     # 交易模块
│   ├── block.rs           # 区块模块
│   ├── blockchain.rs      # 区块链逻辑
│   ├── wallet.rs          # 钱包管理
│   ├── utxo.rs           # UTXO管理
│   ├── merkle.rs         # Merkle树
│   ├── multisig.rs       # 多重签名
│   ├── advanced_tx.rs    # 高级交易特性
│   ├── persistence.rs    # 持久化
│   └── indexer.rs        # 索引器
├── examples/              # 示例程序
│   ├── enterprise_multisig.rs
│   ├── escrow_service.rs
│   └── timelock_savings.rs
├── frontend/              # Electron GUI
│   ├── main.js
│   ├── app.js
│   └── index.html
├── docs/                  # 文档
├── Cargo.toml            # Rust项目配置
└── README.md             # 项目说明
```

## 配置选项

### 挖矿难度

在 `src/blockchain.rs` 中修改：

```rust
pub fn new() -> Blockchain {
    let mut blockchain = Blockchain {
        difficulty: 3,  // 修改这里：3-5适合演示，6+更安全但慢
        // ...
    }
}
```

### 区块奖励

```rust
pub fn new() -> Blockchain {
    let mut blockchain = Blockchain {
        mining_reward: 50,  // 修改挖矿奖励（satoshi）
        // ...
    }
}
```

### API服务器端口

在 `src/bin/server.rs` 中修改：

```rust
let listener = TcpListener::bind("0.0.0.0:3000") // 修改端口
    .await
    .unwrap();
```

## 常见问题

### 编译错误

**问题**: `error: failed to fetch`
```bash
# 解决方案：更新Cargo索引
cargo update
```

**问题**: `error: linker 'cc' not found`
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS (安装Xcode命令行工具)
xcode-select --install
```

### 运行时错误

**问题**: `Address already in use (os error 98)`
```bash
# 端口3000被占用，杀死占用进程或修改端口
lsof -ti:3000 | xargs kill
```

**问题**: 挖矿太慢
```bash
# 降低难度
# 在 blockchain.rs 中设置 difficulty: 2
```

## 下一步

- 📖 阅读[快速入门](./quickstart.md)了解基本使用
- 🎓 学习[基本概念](./concepts.md)理解原理
- 🔨 查看[实战案例](../examples/enterprise-multisig.md)

## 获取帮助

- GitHub Issues: https://github.com/GeoffreyWang1117/SimpleBTC/issues
- 项目文档: 本站
- Rust社区: https://users.rust-lang.org/
