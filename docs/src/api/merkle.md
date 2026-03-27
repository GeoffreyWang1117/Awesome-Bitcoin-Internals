# Merkle API

Merkle 树（哈希树）实现在 `src/merkle.rs` 中，是区块链数据完整性验证的核心数据结构。SimpleBTC 使用 SHA256 构建二叉 Merkle 树，将区块内所有交易汇聚为单个根哈希（`merkle_root`），存储在区块头中。

---

## 数据结构

### MerkleNode 结构体

Merkle 树的单个节点，可以是叶子节点（对应一笔交易）或内部节点（对应子节点哈希的哈希）。

```rust
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,                   // 节点的 SHA256 哈希值（64 字符十六进制）
    pub left: Option<Box<MerkleNode>>,  // 左子节点（叶子节点为 None）
    pub right: Option<Box<MerkleNode>>, // 右子节点（叶子节点为 None）
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `hash` | `String` | 节点哈希。叶子节点为 `SHA256(交易ID)`；内部节点为 `SHA256(左哈希 + 右哈希)`。 |
| `left` | `Option<Box<MerkleNode>>` | 左子节点。叶子节点为 `None`。 |
| `right` | `Option<Box<MerkleNode>>` | 右子节点。叶子节点为 `None`。 |

#### MerkleNode 方法

```rust
// 从原始数据创建叶子节点（计算 SHA256 哈希）
pub fn new_leaf(data: &str) -> Self

// 从两个子节点创建内部节点（哈希 = SHA256(左哈希 + 右哈希)）
pub fn new_internal(left: MerkleNode, right: MerkleNode) -> Self
```

---

### MerkleTree 结构体

完整的 Merkle 树，持有根节点和原始叶子数据列表。

```rust
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>, // 树根节点（空交易列表时为 None）
    pub leaves: Vec<String>,      // 原始叶子数据列表（交易 ID 列表）
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `root` | `Option<MerkleNode>` | 树的根节点。输入为空时为 `None`。 |
| `leaves` | `Vec<String>` | 构建时传入的原始交易 ID 列表（未哈希）。 |

---

## 树的结构示意

以 4 笔交易为例：

```
              Root
             /    \
           H12    H34
          /  \   /  \
        H1  H2  H3  H4
        │    │   │   │
       tx1  tx2 tx3 tx4

其中：
  H1  = SHA256(tx1)
  H2  = SHA256(tx2)
  H12 = SHA256(H1 + H2)
  H34 = SHA256(H3 + H4)
  Root = SHA256(H12 + H34)
```

**奇数交易处理：** 若某层节点数为奇数，最后一个节点被复制配对（例如 3 笔交易时，tx3 被复制为 tx3'）。

---

## 方法

### `MerkleTree::new`

从交易 ID 列表构建完整的 Merkle 树。采用自底向上的方式逐层构建，时间复杂度 O(n)。

```rust
pub fn new(transactions: &[String]) -> Self
```

**参数：**
- `transactions` — 交易 ID（或任意字符串）的切片。可以为空，此时 `root` 为 `None`。

**返回值：** 构建完成的 `MerkleTree` 实例。

```rust
use simplebtc::merkle::MerkleTree;

// 从交易 ID 列表构建树
let tx_ids = vec![
    "tx_hash_1".to_string(),
    "tx_hash_2".to_string(),
    "tx_hash_3".to_string(),
    "tx_hash_4".to_string(),
];
let tree = MerkleTree::new(&tx_ids);

// 处理空交易列表
let empty_tree = MerkleTree::new(&[]);
assert!(empty_tree.root.is_none());
```

---

### `MerkleTree::get_root_hash`

获取 Merkle 树的根哈希字符串。这个值存储在区块头的 `merkle_root` 字段中。

```rust
pub fn get_root_hash(&self) -> String
```

**返回值：**
- 64 字符的小写十六进制 SHA256 哈希字符串（树非空时）。
- 空字符串 `""` （树为空时，即 `root` 为 `None`）。

```rust
let tree = MerkleTree::new(&tx_ids);
let root_hash = tree.get_root_hash();
println!("Merkle 根: {}", root_hash);
// 输出: a3f7c2e1b4d9...（64 字符十六进制）

// 与区块中存储的值比较
assert_eq!(root_hash, block.merkle_root);
```

---

### `MerkleTree::get_proof`

为指定交易生成 Merkle 证明（SPV 证明）。证明是一组兄弟节点哈希，SPV 客户端利用这些哈希从叶子逐层向上重建根哈希，无需访问完整区块。

```rust
pub fn get_proof(&self, tx_hash: &str) -> Option<Vec<String>>
```

**参数：**
- `tx_hash` — 要生成证明的交易 ID（必须存在于 `self.leaves` 中）。

**返回值：**
- `Some(Vec<String>)` — 证明所需的兄弟节点哈希列表，按从叶子层到根层的顺序排列。
- `None` — 交易 ID 不存在于该 Merkle 树中。

**证明大小：** 对于包含 n 笔交易的区块，证明包含 `ceil(log2(n))` 个哈希，每个 32 字节。例如 2000 笔交易的区块，证明仅约 352 字节（11 个哈希）。

```rust
let tree = MerkleTree::new(&tx_ids);

match tree.get_proof("tx_hash_1") {
    Some(proof) => {
        println!("证明包含 {} 个兄弟哈希", proof.len());
        for (i, hash) in proof.iter().enumerate() {
            println!("  层 {}: {}", i, &hash[..16]);
        }
    }
    None => println!("交易不存在于此 Merkle 树"),
}
```

---

### `MerkleTree::verify_proof`

验证 Merkle 证明（静态方法）。这是 SPV 轻量级验证的核心函数：利用证明中的兄弟哈希，从叶子节点逐层向上计算，验证最终结果是否与区块头中的 `merkle_root` 匹配。

```rust
pub fn verify_proof(
    tx_hash: &str,
    proof: &[String],
    root_hash: &str,
    index: usize,
) -> bool
```

**参数：**
- `tx_hash` — 要验证的交易 ID 字符串（原始值，非哈希）。
- `proof` — 由 `get_proof()` 生成的兄弟节点哈希列表。
- `root_hash` — 区块头中存储的 `merkle_root` 值。
- `index` — 交易在区块交易列表中的位置索引（从 0 开始），用于确定左右合并顺序。

**返回值：**
- `true` — 证明有效，该交易确实包含在对应区块中。
- `false` — 证明无效，交易不在该区块中，或数据被篡改。

**验证算法：**

```
输入: tx_hash, proof = [sibling_0, sibling_1, ...], root_hash, index

步骤:
  current = SHA256(tx_hash)
  对于 proof 中的每个 sibling_hash：
    如果 index 为偶数（当前节点在左）：
      current = SHA256(current + sibling_hash)
    如果 index 为奇数（当前节点在右）：
      current = SHA256(sibling_hash + current)
    index = index / 2

最终: current == root_hash → 验证通过
```

```rust
use simplebtc::merkle::MerkleTree;

let tx_ids = vec![
    "tx1".to_string(),
    "tx2".to_string(),
    "tx3".to_string(),
    "tx4".to_string(),
];

let tree = MerkleTree::new(&tx_ids);
let root = tree.get_root_hash();

// 生成证明
let proof = tree.get_proof("tx1").expect("交易存在");

// 验证证明（index=0，tx1 是第一笔交易）
let is_valid = MerkleTree::verify_proof("tx1", &proof, &root, 0);
assert!(is_valid, "SPV 证明验证失败");
println!("交易 tx1 已确认包含在区块中");

// 篡改测试：修改交易内容后证明失效
let tampered = MerkleTree::verify_proof("tx1_TAMPERED", &proof, &root, 0);
assert!(!tampered, "篡改后证明应当失效");
```

---

## 完整使用示例

### 示例一：与区块集成

```rust
use simplebtc::block::Block;
use simplebtc::merkle::MerkleTree;
use simplebtc::transaction::Transaction;
use simplebtc::wallet::Wallet;

fn main() {
    // 模拟打包 4 笔交易
    let miner = Wallet::new();
    let alice = Wallet::new();
    let bob = Wallet::new();

    let transactions = vec![
        Transaction::new_coinbase(&miner.address, 3_125_000),
        Transaction::new(&alice, &bob.address, 100_000, 1_000),
        Transaction::new(&alice, &miner.address, 50_000, 500),
        Transaction::new(&bob, &alice.address, 20_000, 200),
    ];

    // Block::new 内部自动构建 MerkleTree 并计算 merkle_root
    let block = Block::new(1, transactions, "000000abc...".to_string());
    println!("Merkle 根: {}", block.merkle_root);

    // SPV 验证：tx[2] 是否在此区块中
    let tx_id = block.transactions[2].id.clone();
    let included = block.verify_transaction_inclusion(&tx_id, 2);
    println!("交易已包含在区块中: {}", included);
}
```

### 示例二：独立使用 MerkleTree

```rust
use simplebtc::merkle::MerkleTree;

fn spv_demo() {
    // 全节点构建完整 Merkle 树
    let tx_ids: Vec<String> = (1..=8)
        .map(|i| format!("transaction_{:04}", i))
        .collect();

    let tree = MerkleTree::new(&tx_ids);
    let root = tree.get_root_hash();
    println!("8 笔交易的 Merkle 根: {}", root);

    // 为 tx #5 (index=4) 生成 SPV 证明
    let target_tx = "transaction_0005";
    let proof = tree.get_proof(target_tx).expect("交易存在");
    println!("证明大小: {} 个哈希（log2(8)=3 层）", proof.len());

    // SPV 客户端验证（仅需 root + proof，不需要完整交易列表）
    let verified = MerkleTree::verify_proof(target_tx, &proof, &root, 4);
    println!("SPV 验证结果: {}", verified);
}
```

### 示例三：检测数据篡改

```rust
use simplebtc::merkle::MerkleTree;

fn tamper_detection() {
    let original = vec!["tx_a".to_string(), "tx_b".to_string(), "tx_c".to_string()];
    let tree = MerkleTree::new(&original);
    let original_root = tree.get_root_hash();

    // 模拟攻击者修改了 tx_b
    let mut tampered = original.clone();
    tampered[1] = "tx_b_MALICIOUS".to_string();
    let tampered_tree = MerkleTree::new(&tampered);
    let tampered_root = tampered_tree.get_root_hash();

    // Merkle 根完全不同，篡改立即被检测到
    assert_ne!(original_root, tampered_root);
    println!("原始根:   {}", &original_root[..16]);
    println!("篡改后根: {}", &tampered_root[..16]);
    println!("篡改检测成功：根哈希已改变");
}
```

---

## 安全性说明

**为什么 Merkle 证明可以信任？**

攻击者若要伪造一个合法的 Merkle 证明，需要：
1. 找到一个 SHA256 哈希碰撞（计算复杂度约 2^128，当前技术无法实现）；或
2. 重新挖矿（改变 `merkle_root` 会改变区块哈希，需要重新完成工作量证明）。

因此，只要 SPV 客户端能获取到由诚实的工作量证明保护的区块头，Merkle 证明的安全性就与完整节点等价。

**区块时间误差：** 矿工的区块时间戳允许有约 2 小时的误差，但这不影响 Merkle 验证的安全性（Merkle 树不依赖时间戳）。

---

## 相关模块

- [`Block`](block.md) — `Block::new()` 内部调用 `MerkleTree::new()` 计算 `merkle_root`；`Block::verify_transaction_inclusion()` 使用 `get_proof()` 和 `verify_proof()`。
- [高级模块](advanced.md) — SPV 模块 (`src/spv.rs`) 基于 Merkle API 实现轻量级客户端。
