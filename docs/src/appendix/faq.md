# 常见问题（FAQ）

## 安装和配置

### Q: 如何安装SimpleBTC？

**A:**
```bash
# 1. 确保已安装Rust
rustc --version

# 2. 克隆项目
git clone https://github.com/GeoffreyWang1117/SimpleBTC.git
cd SimpleBTC

# 3. 编译
cargo build --release

# 4. 运行
cargo run --bin btc-demo
```

详见[安装指南](../guide/installation.md)

---

### Q: 编译时报错 "linker 'cc' not found"

**A:** 需要安装C编译器：

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Windows
# 安装Visual Studio Build Tools
```

---

### Q: 如何修改挖矿难度？

**A:** 在 `src/blockchain.rs` 中修改：

```rust
pub fn new() -> Blockchain {
    Blockchain {
        difficulty: 3,  // 修改这个值
        // 3-4 适合演示
        // 5-6 更安全但慢
        // ...
    }
}
```

---

## 基础概念

### Q: 什么是UTXO？为什么不是账户余额？

**A:** UTXO（Unspent Transaction Output）是比特币的核心概念。

**账户模型**（以太坊）：
```
Alice账户: 100 BTC
转账后:
Alice: 70 BTC
Bob: 30 BTC
```

**UTXO模型**（比特币）：
```
Alice有UTXO: [50 BTC, 30 BTC, 20 BTC]
转账30 BTC给Bob:
  - 消费50 BTC的UTXO
  - 创建30 BTC给Bob
  - 创建20 BTC找零给Alice（50-30）
```

**优势**：
- ✅ 更好的隐私性（每次用新地址）
- ✅ 并行处理（不同UTXO可并发）
- ✅ 更简单的验证逻辑

详见[基本概念 - UTXO](../guide/concepts.md#utxo模型)

---

### Q: 为什么交易需要手续费？

**A:** 手续费的作用：

1. **防止垃圾攻击** - 发送交易有成本
2. **激励矿工** - 矿工优先打包高费率交易
3. **资源分配** - 网络拥堵时，愿意支付更高费用的优先

**手续费计算**：
```rust
手续费 = 输入总额 - 输出总额

// 示例
输入: 100 satoshi
输出: 90 satoshi
手续费: 10 satoshi
```

**费率建议**：
- 1-5 sat/byte: 低优先级（数小时）
- 10-20 sat/byte: 中优先级（30-60分钟）
- 50+ sat/byte: 高优先级（下一个区块）

---

### Q: 什么是工作量证明（PoW）？为什么需要挖矿？

**A:** PoW是比特币的共识机制。

**挖矿过程**：
```rust
target = "000..."  // 难度要求

while hash(block_data + nonce) >= target {
    nonce++;  // 不断尝试
}
// 找到有效nonce，区块被接受
```

**为什么需要**：
- 防止垃圾区块（创建区块需要计算成本）
- 去中心化共识（算力投票）
- 51%攻击成本极高（需要超过全网一半算力）

**难度与时间**：
- 难度3: 毫秒级（demo）
- 难度10: 秒级（私有链）
- 难度20: 分钟级（比特币级别）

详见[基本概念 - PoW](../guide/concepts.md#工作量证明proof-of-work)

---

## 使用问题

### Q: 如何创建钱包？

**A:**
```rust
use bitcoin_simulation::wallet::Wallet;

// 创建新钱包
let wallet = Wallet::new();

println!("地址: {}", wallet.address);
println!("公钥: {}", wallet.public_key);
// 私钥要保密！
```

**重要**：
- 私钥丢失 = 比特币永久丢失
- 私钥泄露 = 比特币被盗
- 建议备份私钥到安全的地方

---

### Q: 如何转账？

**A:**
```rust
use bitcoin_simulation::{blockchain::Blockchain, wallet::Wallet};

let mut blockchain = Blockchain::new();
let alice = Wallet::new();
let bob = Wallet::new();

// 1. 创建交易
let tx = blockchain.create_transaction(
    &alice,           // 发送者
    bob.address,      // 接收者
    1000,            // 金额（satoshi）
    10,              // 手续费
)?;

// 2. 添加到待处理池
blockchain.add_transaction(tx)?;

// 3. 挖矿确认
blockchain.mine_pending_transactions(miner.address)?;
```

---

### Q: 余额不足怎么办？

**A:** 检查以下几点：

1. **查询余额**：
```rust
let balance = blockchain.get_balance(&address);
println!("余额: {}", balance);
```

2. **确保有UTXO**：
```rust
let utxos = blockchain.utxo_set.find_utxos(&address);
println!("UTXO数量: {}", utxos.len());
```

3. **检查是否包含手续费**：
```rust
let total_needed = amount + fee;
if balance < total_needed {
    return Err("余额不足（包括手续费）");
}
```

4. **等待交易确认**：
刚发送的交易需要挖矿确认后才能使用。

---

### Q: 交易长时间未确认怎么办？

**A:** 可能原因和解决方案：

**原因1：手续费太低**
```rust
// 提高手续费
let tx = blockchain.create_transaction(
    &alice,
    bob.address,
    1000,
    50,  // 提高手续费
)?;
```

**原因2：没有矿工挖矿**
```bash
# 手动挖矿
cargo run --bin btc-demo
# 或在代码中
blockchain.mine_pending_transactions(miner.address)?;
```

**原因3：交易无效**
```rust
// 验证交易
if !tx.verify() {
    println!("交易无效，检查：");
    println!("- 输入UTXO是否存在");
    println!("- 签名是否正确");
    println!("- 余额是否足够");
}
```

**使用RBF加速**：
```rust
// 创建更高费率的替换交易
let faster_tx = blockchain.create_transaction(
    &alice,
    bob.address,
    1000,
    100,  // 更高的手续费
)?;
```

---

## 高级功能

### Q: 如何使用多重签名？

**A:**
```rust
use bitcoin_simulation::multisig::MultiSigAddress;

// 1. 创建参与者钱包
let alice = Wallet::new();
let bob = Wallet::new();
let charlie = Wallet::new();

// 2. 创建2-of-3多签地址
let multisig = MultiSigAddress::new(
    2,  // 需要2个签名
    vec![
        alice.public_key,
        bob.public_key,
        charlie.public_key,
    ]
)?;

// 3. 向多签地址发送资金
let tx = blockchain.create_transaction(
    &funder,
    multisig.address.clone(),
    10000,
    0,
)?;

// 4. 从多签地址支出（需要2个签名）
let alice_sig = alice.sign(&payment_data);
let bob_sig = bob.sign(&payment_data);

if vec![alice_sig, bob_sig].len() >= multisig.required_sigs {
    // 执行交易
}
```

详见[多重签名教程](../advanced/multisig.md)

---

### Q: 什么是Merkle树？有什么用？

**A:** Merkle树是交易的哈希树，存储在区块头中。

**结构**：
```
        Root Hash
       /         \
     H(AB)      H(CD)
    /    \      /    \
  H(A)  H(B)  H(C)  H(D)
   tx1   tx2   tx3   tx4
```

**用途**：

1. **SPV验证** - 轻钱包无需下载完整区块
```rust
// 只需区块头 + Merkle证明
let proof = merkle_tree.get_proof(&tx_hash)?;
let valid = MerkleTree::verify_proof(
    &tx_hash,
    &proof,
    &block.merkle_root,
    tx_index
);
```

2. **数据完整性** - 任何交易改变都会改变根哈希

3. **高效验证** - O(log n)复杂度

详见[Merkle树教程](../advanced/merkle.md)

---

### Q: 什么是时间锁？如何使用？

**A:** 时间锁限制交易在特定时间前不能被确认。

**两种类型**：

1. **基于时间戳**：
```rust
use bitcoin_simulation::advanced_tx::TimeLock;

// 3个月后解锁
let three_months = 90 * 24 * 3600;
let unlock_time = current_time + three_months;
let timelock = TimeLock::new_time_based(unlock_time);

// 检查是否到期
if timelock.is_mature(current_time, 0) {
    println!("已到期，可以使用");
}
```

2. **基于区块高度**：
```rust
// 在第100,000个区块后解锁
let timelock = TimeLock::new_block_based(100000);

if timelock.is_mature(current_time, current_block_height) {
    println!("区块高度已达到");
}
```

**应用场景**：
- 定期存款
- 遗产继承
- 工资发放
- 项目锁定期

详见[时间锁教程](../advanced/timelock.md)

---

## 开发问题

### Q: 如何集成SimpleBTC到我的项目？

**A:** SimpleBTC可以作为库使用：

```toml
# Cargo.toml
[dependencies]
bitcoin_simulation = { path = "../SimpleBTC" }
```

```rust
// 在你的代码中
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
};

fn my_app() {
    let blockchain = Blockchain::new();
    // ... 你的业务逻辑
}
```

---

### Q: 如何使用REST API？

**A:**

**启动服务器**：
```bash
cargo run --bin btc-server
# 服务器运行在 http://localhost:3000
```

**API调用示例**：

```bash
# 创建钱包
curl -X POST http://localhost:3000/api/wallet/create

# 创建交易
curl -X POST http://localhost:3000/api/transaction/create \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice_address",
    "to": "bob_address",
    "amount": 1000,
    "fee": 10
  }'

# 查询余额
curl http://localhost:3000/api/balance/alice_address

# 挖矿
curl -X POST http://localhost:3000/api/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address": "miner_address"}'
```

详见[REST API文档](../api/rest.md)

---

### Q: 如何运行测试？

**A:**
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_blockchain

# 显示输出
cargo test -- --nocapture

# 运行示例
cargo run --example enterprise_multisig
cargo run --example escrow_service
cargo run --example timelock_savings
```

---

### Q: 如何部署文档网站？

**A:**

**本地预览**：
```bash
cd docs
mdbook serve --open
```

**GitHub Pages部署**：
```bash
# 构建
mdbook build

# 部署到gh-pages分支
# 详见 docs/README.md
```

**Docker部署**：
```dockerfile
FROM nginx:alpine
COPY docs/book /usr/share/nginx/html
EXPOSE 80
```

详见[文档部署指南](../../../docs/README.md)

---

## 性能问题

### Q: 挖矿太慢怎么办？

**A:** 调整难度：

```rust
// 在 blockchain.rs 中
blockchain.difficulty = 3;  // 降低难度
// 3: 毫秒级
// 4: 秒级
// 5: 数秒
// 6+: 可能很慢
```

或者使用Release模式：
```bash
cargo run --release --bin btc-demo
# Release模式比Debug快很多
```

---

### Q: 余额查询很慢？

**A:** 使用索引器加速：

```rust
// SimpleBTC已经内置了索引器
let txs = blockchain.indexer.get_transactions_by_address(&address);

// 或者缓存余额
let balance_cache: HashMap<String, u64> = HashMap::new();
```

---

## 安全问题

### Q: SimpleBTC安全吗？可以用于生产吗？

**A:** ⚠️ **SimpleBTC是教育项目，不建议用于生产！**

**与真实比特币的差异**：
- ❌ 简化的密码学（SHA256代替ECDSA）
- ❌ 无P2P网络层
- ❌ 简化的脚本系统
- ❌ 无完整的SPV实现
- ❌ JSON存储（应该用LevelDB）

**用于生产需要**：
- 实现完整的secp256k1椭圆曲线
- 实现ECDSA签名验证
- 添加P2P网络协议
- 使用专业的数据库
- 完整的Script脚本引擎
- 经过安全审计

---

### Q: 如何保护私钥？

**A:** 私钥安全建议：

1. **永不分享私钥**
2. **多重备份**：
   - 纸钱包（防火防水）
   - 硬件钱包
   - 加密U盘
3. **分散存储**：
   - 家中保险柜
   - 银行保险箱
   - 异地备份
4. **使用多签**：
   - 2-of-3减少单点风险
5. **定期测试恢复**

---

## 其他问题

### Q: SimpleBTC与真实比特币的区别？

**A:**

| 特性 | SimpleBTC | 真实比特币 |
|------|-----------|------------|
| 密码学 | SHA256（简化） | secp256k1 ECDSA |
| 共识 | PoW（简化） | PoW（完整） |
| 脚本 | 简化 | 完整Script语言 |
| 网络 | 无 | P2P网络 |
| 存储 | JSON | LevelDB |
| 难度调整 | 固定 | 每2016区块调整 |

**SimpleBTC的价值**：
- ✅ 学习比特币原理
- ✅ 理解UTXO模型
- ✅ 实践区块链开发
- ✅ 快速原型验证

---

### Q: 如何贡献代码？

**A:**

1. Fork项目
2. 创建功能分支
3. 提交Pull Request
4. 等待Review

详见[贡献指南](./contributing.md)

---

### Q: 遇到Bug怎么办？

**A:**

1. 在GitHub提Issue：
   https://github.com/GeoffreyWang1117/SimpleBTC/issues

2. 提供以下信息：
   - 操作系统
   - Rust版本
   - 错误信息
   - 复现步骤
   - 相关代码

---

### Q: 在哪里获取帮助？

**A:**

- 📖 **文档**: 本站
- 💬 **GitHub Issues**: 报告问题和建议
- 📚 **Rust社区**: https://users.rust-lang.org/
- 📖 **比特币白皮书**: https://bitcoin.org/bitcoin.pdf

---

## 更多资源

- [快速入门](../guide/quickstart.md)
- [基本概念](../guide/concepts.md)
- [API文档](../api/core.md)
- [高级特性](../advanced/multisig.md)
- [实战案例](../examples/enterprise-multisig.md)

---

**没找到你的问题？** [在GitHub提Issue](https://github.com/GeoffreyWang1117/SimpleBTC/issues)
