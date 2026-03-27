# Block API

`Block` 是区块链的基本组成单位，定义在 `src/block.rs` 中。每个区块包含一批已确认的交易，并通过哈希链与前一个区块相连，共同构成不可篡改的账本。

---

## Block 结构体

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,                     // 区块高度（索引），创世区块为 0
    pub timestamp: u64,                 // Unix 时间戳（秒）
    pub transactions: Vec<Transaction>, // 交易列表（第一笔必须是 Coinbase 交易）
    pub previous_hash: String,          // 父区块哈希（SHA256，64 字符十六进制）
    pub hash: String,                   // 当前区块哈希（通过挖矿找到）
    pub nonce: u64,                     // 工作量证明的随机数（挖矿时调整此值）
    pub merkle_root: String,            // 交易 Merkle 树根哈希
}
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `index` | `u32` | 区块高度。创世区块为 `0`，之后每个区块递增 1。 |
| `timestamp` | `u64` | 区块创建时的 Unix 时间戳（秒）。由 `SystemTime::now()` 自动填写。 |
| `transactions` | `Vec<Transaction>` | 区块包含的交易列表。第一笔**必须**是 Coinbase 交易（矿工奖励）。 |
| `previous_hash` | `String` | 父区块的 SHA256 哈希（64 字符十六进制）。创世区块此字段为 `"0"`。 |
| `hash` | `String` | 当前区块的 SHA256 哈希。由 `calculate_hash()` 计算，在挖矿过程中持续更新直到满足难度要求。 |
| `nonce` | `u64` | 工作量证明的随机数。矿工通过递增 `nonce` 来寻找满足难度条件的哈希。 |
| `merkle_root` | `String` | 区块内所有交易的 Merkle 树根哈希。任何交易被篡改都会导致此值改变。 |

### 区块链式结构

```
创世区块 (index=0)   ->   区块 1          ->   区块 2
prev: "0"                prev: abc123...       prev: def456...
hash: abc123...          hash: def456...       hash: ghi789...
```

由于每个区块的 `hash` 依赖于 `previous_hash`，以及所有交易（通过 `merkle_root`），任何历史区块的修改都需要重新计算其后所有区块的哈希，这在计算上是不可行的。

---

## 方法

### `Block::new`

创建一个新区块。自动设置时间戳并计算 Merkle 根，但 `nonce` 初始为 `0`，`hash` 为初始计算值（尚未满足挖矿难度要求）。

```rust
pub fn new(
    index: u32,
    transactions: Vec<Transaction>,
    previous_hash: String,
) -> Block
```

**参数：**
- `index` — 新区块的高度。
- `transactions` — 要打包进区块的交易列表（第一笔应为 Coinbase 交易）。
- `previous_hash` — 父区块的哈希字符串。

**返回值：** 初始化好的 `Block` 实例（尚未完成挖矿）。

**内部流程：**
1. 获取当前 Unix 时间戳。
2. 从 `transactions` 的 `id` 列表构建 `MerkleTree`，计算 `merkle_root`。
3. 以 `nonce = 0` 构建区块并调用 `calculate_hash()` 得到初始哈希。

```rust
use simplebtc::block::Block;
use simplebtc::transaction::Transaction;

let coinbase = Transaction::new_coinbase("miner_address", 3125000); // 3.125 BTC（satoshi）
let block = Block::new(1, vec![coinbase], "abc123...".to_string());
println!("区块 #{}: {}", block.index, block.hash);
```

---

### `Block::calculate_hash`

计算区块的 SHA256 哈希值。哈希输入包含 `index`、`timestamp`、`merkle_root`、`previous_hash` 和 `nonce`。

```rust
pub fn calculate_hash(&self) -> String
```

**返回值：** 64 字符的小写十六进制 SHA256 哈希字符串。

**哈希输入格式：**
```
"{index}{timestamp}{merkle_root}{previous_hash}{nonce}"
```

使用 `merkle_root` 而非完整交易数据，使区块头保持轻量（约 80 字节），同时保证所有交易内容的完整性。

```rust
let mut block = Block::new(1, transactions, prev_hash);
// 修改 nonce 后重新计算哈希（挖矿核心逻辑）
block.nonce += 1;
block.hash = block.calculate_hash();
println!("新哈希: {}", block.hash);
```

---

### `Block::validate_transactions`

验证区块中所有交易的签名有效性。依次调用每笔交易的 `verify()` 方法。

```rust
pub fn validate_transactions(&self) -> bool
```

**返回值：**
- `true` — 所有交易签名均有效。
- `false` — 存在至少一笔无效交易。

```rust
let block = Block::new(1, transactions, prev_hash);

if block.validate_transactions() {
    println!("所有交易有效，可以上链");
} else {
    println!("区块包含无效交易，拒绝");
}
```

> **注意：** 此方法仅验证签名，不验证 UTXO 余额。余额验证由 `Blockchain` 层负责。

---

### `Block::verify_transaction_inclusion`

使用 Merkle 证明验证某笔交易是否确实包含在该区块中。这是 SPV（简化支付验证）的核心功能，无需遍历所有交易，时间复杂度为 O(log n)。

```rust
pub fn verify_transaction_inclusion(
    &self,
    tx_id: &str,
    index: usize,
) -> bool
```

**参数：**
- `tx_id` — 要验证的交易 ID（哈希字符串）。
- `index` — 该交易在区块交易列表中的位置索引（从 0 开始）。

**返回值：**
- `true` — 交易确实包含在该区块中，且 Merkle 证明有效。
- `false` — 交易不在该区块中，或证明无效。

**内部流程：**
1. 重建区块的 `MerkleTree`。
2. 调用 `get_proof(tx_id)` 生成 Merkle 证明。
3. 调用 `MerkleTree::verify_proof()` 验证证明与 `merkle_root` 是否匹配。

```rust
let tx_id = "abc123def456...";
let tx_index = 2; // 该交易在区块中的位置

if block.verify_transaction_inclusion(tx_id, tx_index) {
    println!("交易已确认包含在第 {} 个区块中", block.index);
} else {
    println!("交易不在此区块中");
}
```

---

### `Block::mine_block`

工作量证明（Proof of Work）挖矿。不断递增 `nonce` 并重新计算哈希，直到哈希前缀满足难度要求（即以 `difficulty` 个 `'0'` 开头）。

```rust
pub fn mine_block(&mut self, difficulty: usize)
```

**参数：**
- `difficulty` — 挖矿难度，即哈希前缀需要的 `'0'` 个数。

**副作用：** 修改 `self.nonce` 和 `self.hash`，直到找到有效哈希。

```rust
let mut block = Block::new(1, transactions, prev_hash);
println!("开始挖矿，难度: 4");
block.mine_block(4); // 哈希必须以 "0000" 开头
println!("挖矿完成: {}", block.hash);
println!("使用 nonce: {}", block.nonce);
// 输出示例: 0000a3f7c2...
```

> **关于难度：** 比特币主网当前难度约等效于哈希前缀约 20 个 `'0'`（需要约 2^80 次哈希计算）。本项目使用较小难度值（如 2-4）以便演示。

---

## 完整使用示例

```rust
use simplebtc::block::Block;
use simplebtc::transaction::Transaction;
use simplebtc::wallet::Wallet;

fn main() {
    // 1. 创建矿工钱包
    let miner = Wallet::new();

    // 2. 创建 Coinbase 交易（矿工奖励）
    let coinbase = Transaction::new_coinbase(&miner.address, 3_125_000);

    // 3. 创建普通转账交易
    let alice = Wallet::new();
    let bob = Wallet::new();
    let transfer = Transaction::new(&alice, &bob.address, 50_000, 500);

    // 4. 打包区块（假设父块哈希已知）
    let prev_hash = "0000abc123...".to_string();
    let mut block = Block::new(1, vec![coinbase, transfer], prev_hash);

    // 5. 挖矿（工作量证明）
    block.mine_block(3); // 难度 3：哈希以 "000" 开头

    // 6. 验证区块
    assert!(block.validate_transactions(), "区块交易无效");
    println!("区块哈希: {}", block.hash);
    println!("Merkle 根: {}", block.merkle_root);
    println!("Nonce: {}", block.nonce);

    // 7. SPV 验证：某交易是否在此区块中
    let included = block.verify_transaction_inclusion(&block.transactions[0].id.clone(), 0);
    println!("Coinbase 交易已包含: {}", included);
}
```

---

## 不可篡改性原理

```
攻击者尝试修改区块 1 的某笔交易：

  修改交易
      ↓
  交易哈希改变
      ↓
  Merkle Root 改变
      ↓
  区块 1 的 Hash 改变
      ↓
  区块 2 的 previous_hash 不匹配
      ↓
  区块 2、3、4... 的 Hash 全部失效
      ↓
  攻击者需要重新挖所有后续区块（计算上不可行）
```

这就是区块链"不可篡改性"的数学保证。

---

## 相关模块

- [`MerkleTree`](merkle.md) — Merkle 树实现，用于计算 `merkle_root` 和生成 SPV 证明。
- [`Transaction`](transaction.md) — 交易结构体，Block 的核心数据。
- [`Blockchain`](blockchain.md) — 管理区块链，调用 `mine_block()` 并维护链状态。
