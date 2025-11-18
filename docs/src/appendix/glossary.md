# 术语表

比特币和区块链相关的核心术语解释。

## A

### Address（地址）
用于接收比特币的唯一标识符。由公钥通过哈希和编码生成。

**示例**: `1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa`

**类型**:
- P2PKH（以1开头）：传统地址
- P2SH（以3开头）：脚本/多签地址
- Bech32（以bc1开头）：SegWit地址

---

### ACID
数据库事务的四个特性：
- **Atomicity（原子性）**: 全部成功或全部失败
- **Consistency（一致性）**: 保持数据一致状态
- **Isolation（隔离性）**: 并发事务互不影响
- **Durability（持久性）**: 已提交的持久保存

SimpleBTC的交易处理符合ACID特性。

---

## B

### Block（区块）
包含交易列表的数据结构，通过哈希链接形成区块链。

**包含**:
- 区块头（index, timestamp, hash, previous_hash, merkle_root, nonce）
- 交易列表（第一笔是Coinbase）

**大小**: 比特币约1-4MB

---

### Blockchain（区块链）
按时间顺序链接的区块序列，通过密码学确保不可篡改。

**特性**:
- 去中心化
- 不可篡改
- 透明可验证
- 无需信任第三方

---

### Block Height（区块高度）
区块在链中的位置。创世区块高度为0。

**示例**: 当前区块链有100个区块，最新区块高度为99。

---

### BIP (Bitcoin Improvement Proposal)
比特币改进提案，用于提出比特币协议的改进。

**重要BIP**:
- BIP11: M-of-N多签
- BIP16: P2SH（脚本哈希支付）
- BIP32: 分层确定性钱包
- BIP39: 助记词
- BIP125: RBF（替换手续费）

---

## C

### Coinbase Transaction
区块的第一笔交易，用于向矿工发放奖励。

**特点**:
- 无有效输入（不消费UTXO）
- 创造新的比特币
- 包含区块奖励 + 交易手续费

**示例**:
```rust
Transaction::new_coinbase(
    miner_address,
    50,           // 区块奖励
    timestamp,
    total_fees,   // 手续费总和
)
```

---

### Cold Wallet（冷钱包）
离线存储私钥的钱包，不连接互联网。

**类型**:
- 硬件钱包（Ledger, Trezor）
- 纸钱包
- 气隙计算机

**优势**: 极高的安全性
**劣势**: 使用不便

---

### Confirmation（确认）
交易被包含在区块中并被后续区块延续的次数。

**确认数**:
- 0确认：在待处理池，未打包
- 1确认：已打包到区块
- 6确认：非常安全（比特币标准）

**时间**: 比特币约10分钟/确认

---

## D

### Difficulty（难度）
挖矿的计算难度，决定了找到有效区块哈希的难度。

**SimpleBTC**:
```rust
blockchain.difficulty = 3;  // 3个前导0
```

**比特币**: 动态调整，每2016个区块（约2周）调整一次，目标是保持10分钟出块时间。

---

### Double Spending（双花）
试图将同一笔比特币花费两次的攻击。

**防御机制**:
1. UTXO模型（每个UTXO只能花费一次）
2. 区块确认（6确认后几乎不可能）
3. 工作量证明（需要51%算力重写历史）

---

## E

### ECDSA (Elliptic Curve Digital Signature Algorithm)
椭圆曲线数字签名算法，比特币使用的签名方案。

**曲线**: secp256k1

**过程**:
```
私钥 → ECDSA → 公钥 → Hash → 地址
```

SimpleBTC使用简化的SHA256签名。

---

## F

### Fee（手续费）
支付给矿工的费用，激励其打包交易。

**计算**:
```
手续费 = 输入总额 - 输出总额
```

**费率**:
```
费率 = 手续费 / 交易大小（sat/byte）
```

**推荐**:
- 低: 1-5 sat/byte
- 中: 10-20 sat/byte
- 高: 50+ sat/byte

---

### Fork（分叉）
区块链出现多个有效分支。

**类型**:
- **暂时性分叉**: 两个矿工同时挖出区块，最长链原则解决
- **硬分叉**: 协议不兼容升级（如BCH）
- **软分叉**: 向后兼容升级（如SegWit）

---

## G

### Genesis Block（创世区块）
区块链的第一个区块，索引为0。

**比特币创世区块**:
- 日期: 2009年1月3日
- 奖励: 50 BTC（无法花费）
- 消息: "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"

**SimpleBTC**:
```rust
fn create_genesis_block() {
    // 创建索引为0的区块
    // previous_hash = "0"
}
```

---

## H

### Hash（哈希）
将任意数据转换为固定长度字符串的函数。

**比特币使用**:
- SHA256（交易ID、区块哈希）
- RIPEMD160（地址生成）

**特性**:
- 确定性
- 单向性
- 抗碰撞
- 雪崩效应

**示例**:
```
SHA256("hello") = 2cf24dba5fb0a30e...
```

---

### Hash Rate（算力）
每秒计算哈希的次数。

**单位**:
- H/s (hashes per second)
- KH/s = 1,000 H/s
- MH/s = 1,000,000 H/s
- GH/s = 1,000,000,000 H/s
- TH/s = 1,000,000,000,000 H/s
- EH/s = 1,000,000,000,000,000,000 H/s

**比特币全网**: 约300+ EH/s

---

### Hot Wallet（热钱包）
连接互联网的钱包，便于日常使用。

**类型**:
- 手机钱包
- 桌面钱包
- Web钱包

**优势**: 使用方便
**劣势**: 安全性较低

---

## M

### Merkle Tree（Merkle树）
交易的二叉哈希树，根哈希存储在区块头。

**结构**:
```
        Root
       /    \
     H12    H34
    /  \   /  \
   H1  H2 H3  H4
```

**用途**:
- SPV轻量级验证
- 证明交易存在于区块中
- O(log n)验证复杂度

---

### Mining（挖矿）
通过工作量证明创建新区块的过程。

**步骤**:
1. 收集待处理交易
2. 创建Coinbase交易
3. 计算Merkle根
4. 调整nonce寻找有效哈希
5. 广播区块

**奖励**: 区块奖励 + 交易手续费

---

### Multisig (M-of-N)
需要M个签名（共N个密钥）才能花费的地址。

**示例**:
- 2-of-3: CEO + CFO + CTO，任意2人即可
- 3-of-5: 董事会5人，需要3人同意

**地址**: 以"3"开头（P2SH）

---

## N

### Node（节点）
运行比特币客户端软件的计算机。

**类型**:
- **全节点**: 存储完整区块链，验证所有交易
- **轻节点**: 只存储区块头，使用SPV验证
- **矿工节点**: 参与挖矿的全节点

---

### Nonce
挖矿时调整的随机数，用于改变区块哈希。

**作用**: 工作量证明
```rust
while hash(block_data + nonce) >= target {
    nonce++;  // 不断尝试
}
```

---

## P

### P2P (Peer-to-Peer)
点对点网络，节点之间直接通信，无需中心服务器。

**比特币网络**:
- 去中心化
- 抗审查
- 无单点故障

---

### P2PKH (Pay-to-Public-Key-Hash)
传统的比特币地址类型，以"1"开头。

**流程**:
```
公钥 → SHA256 → RIPEMD160 → Base58 → 地址
```

---

### P2SH (Pay-to-Script-Hash)
脚本哈希支付，用于多签等高级功能，以"3"开头。

**优势**:
- 支持复杂脚本
- 隐藏脚本细节
- 费用由接收方承担

---

### Private Key（私钥）
用于签名交易的秘密数字。

**特性**:
- 256位随机数
- 拥有私钥 = 拥有比特币
- 丢失不可恢复

**保护**:
- 永不分享
- 加密存储
- 多重备份

---

### Proof of Work (PoW)
工作量证明，比特币的共识机制。

**原理**: 找到满足难度的哈希值需要大量计算

**目的**:
- 防止垃圾攻击
- 去中心化共识
- 51%攻击成本极高

---

### Public Key（公钥）
从私钥派生的公开数字，用于生成地址和验证签名。

**推导**:
```
私钥 → 椭圆曲线运算 → 公钥 → 哈希 → 地址
```

---

## R

### RBF (Replace-By-Fee)
允许替换未确认交易的机制（BIP125）。

**用途**:
- 加速交易（提高手续费）
- 取消交易
- 批量优化

**标记**: nSequence < 0xFFFFFFFE

---

## S

### Satoshi (sat)
比特币的最小单位。

```
1 BTC = 100,000,000 satoshi
1 sat = 0.00000001 BTC
```

**命名**: 以比特币创始人Satoshi Nakamoto命名

---

### Script
比特币的脚本语言，定义花费条件。

**操作码**:
- OP_DUP
- OP_HASH160
- OP_EQUALVERIFY
- OP_CHECKSIG
- OP_CHECKMULTISIG

SimpleBTC使用简化版本。

---

### SPV (Simplified Payment Verification)
轻量级验证，无需下载完整区块链。

**原理**: 使用Merkle证明验证交易

**优势**:
- 只需区块头（~80字节）
- 手机钱包可用
- O(log n)验证

---

## T

### Timelock (nLockTime)
限制交易在特定时间前不能被确认。

**类型**:
- 时间戳（≥ 500,000,000）
- 区块高度（< 500,000,000）

**应用**:
- 定期存款
- 遗产继承
- 工资发放

---

### Transaction (TX)
价值转移的基本单位。

**包含**:
- 输入（花费哪些UTXO）
- 输出（创建哪些新UTXO）
- 时间戳
- 手续费

---

## U

### UTXO (Unspent Transaction Output)
未花费的交易输出，代表可以被花费的比特币。

**生命周期**:
1. 创建（交易输出）
2. 存在（UTXO集合）
3. 花费（交易输入引用）
4. 移除（从UTXO集合删除）

**余额**: 所有UTXO的总和

---

## W

### Wallet（钱包）
管理私钥、公钥和地址的软件。

**类型**:
- 热钱包（联网）
- 冷钱包（离线）
- 硬件钱包
- 纸钱包

**功能**:
- 生成密钥对
- 创建地址
- 签名交易
- 查询余额

---

## 数字

### 51% Attack
攻击者控制超过50%算力，可以重写区块链历史。

**后果**:
- 双花攻击
- 阻止交易确认

**防御**: 比特币算力太大，攻击成本极高

---

### 6 Confirmations
比特币交易的标准安全确认数。

**时间**: 约60分钟（6个区块 × 10分钟）

**原因**: 6个区块后，重写历史几乎不可能

---

## 参考资源

- [比特币白皮书](https://bitcoin.org/bitcoin.pdf)
- [比特币Wiki](https://en.bitcoin.it/wiki/Main_Page)
- [精通比特币](https://github.com/bitcoinbook/bitcoinbook)
- [BIP列表](https://github.com/bitcoin/bips)

---

[返回文档首页](../introduction/README.md) | [基本概念](../guide/concepts.md)
