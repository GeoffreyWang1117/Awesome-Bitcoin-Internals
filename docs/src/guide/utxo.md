# UTXO管理

UTXO（Unspent Transaction Output，未花费的交易输出）是比特币账本模型的核心。理解UTXO是掌握比特币工作原理的关键一步。本章介绍SimpleBTC中`UTXOSet`的设计与使用。

---

## 什么是UTXO？

在传统银行体系（以及以太坊账户模型）中，系统直接记录每个账户的余额数字。比特币选择了一种截然不同的方式：**不存储余额，只存储"未花费的输出"**。

每一笔比特币交易都会：
1. **消费**若干个此前已存在的UTXO（作为输入）
2. **创建**若干个新的UTXO（作为输出）

把比特币想象成现金纸币：你手里有一张100元和一张50元。你去买一件80元的商品，你需要把100元纸币整张交出去，找回20元。你"花费"了100元这个UTXO，同时"创建"了两个新UTXO——一个价值80元给商家，一个价值20元的找零给自己。

### UTXO的生命周期

```
创建                  存在                   花费                  销毁
 |                     |                      |                     |
 v                     v                      v                     v
交易输出 ──────► UTXO集合 ──────────────► 被新交易引用 ──────► 从集合移除
(打包进区块)      (可被查询和花费)          (作为输入)           (不可再用)
```

---

## UTXO模型 vs 账户模型

| 特性 | UTXO模型（比特币） | 账户模型（以太坊） |
|------|-------------------|-------------------|
| 状态存储 | 记录所有未花费输出 | 记录每个账户的余额 |
| 余额计算 | 扫描所有属于该地址的UTXO求和 | 直接读取账户字段 |
| 隐私性 | 较好（每笔交易可换地址） | 较弱（地址固定） |
| 并行处理 | 天然支持（不同UTXO互不依赖） | 需要额外的并发控制 |
| 双花防止 | UTXO只能被消费一次 | 通过nonce序号控制 |
| 复杂合约 | 较难实现 | 原生支持 |
| 直观性 | 需要理解UTXO概念 | 类似银行账户，直观 |

源码注释（`src/utxo.rs`第8–19行）对此有精炼的总结：

```rust
// 账户模型（以太坊等）：
// - 记录每个账户的余额
// - 转账：A账户-100，B账户+100
// - 简单直观，但难以并行处理
//
// UTXO模型（比特币）：
// - 没有账户余额概念
// - 只记录未花费的交易输出
// - 转账：消费A的UTXO，创建给B的新UTXO
// - 更好的隐私性和并行性
```

---

## UTXOSet数据结构

SimpleBTC使用`UTXOSet`来管理整个区块链的未花费输出集合。

```rust
#[derive(Debug, Clone)]
pub struct UTXOSet {
    // key: txid（交易ID）
    // value: 该交易所有未花费的输出列表 [(输出索引, 输出详情)]
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
}
```

- **键（key）**：交易ID（txid），是一个十六进制哈希字符串
- **值（value）**：该交易下还未被花费的输出列表，每项包含输出在交易中的索引（`vout`）和输出的详细信息（`TxOutput`）

这种结构设计使得按txid查找某笔交易的所有可用输出非常高效（O(1)哈希查找），同时也能方便地移除指定的单个输出。

---

## 核心API详解

### 创建UTXO集合

```rust
let mut utxo_set = UTXOSet::new();
```

初始化一个空的UTXO集合。区块链启动时从创世区块开始，逐块处理所有交易来填充它。

---

### 添加交易输出：`add_transaction`

```rust
pub fn add_transaction(&mut self, tx: &Transaction)
```

当一笔新交易被打包进区块并确认时，调用此方法将该交易的**所有输出**加入UTXO集合。

```rust
// 示例：挖到创世区块，coinbase奖励进入UTXO集合
let coinbase_tx = Transaction::new_coinbase("miner_address".to_string(), 50, 0, 0);
utxo_set.add_transaction(&coinbase_tx);
// 现在 coinbase_tx.id -> [(0, TxOutput { value: 50, ... })] 在集合中
```

> **注意**：`add_transaction`只添加输出，不处理输入（不移除被花费的UTXO）。完整的交易处理应使用`process_transaction`。

---

### 移除已花费输出：`remove_utxo`

```rust
pub fn remove_utxo(&mut self, txid: &str, vout: usize)
```

当一个UTXO被某笔交易的输入引用（即被花费）时，必须将其从集合中移除。这是防止双重花费的核心机制。

```rust
// 用户花费了 txid="abc123" 的第0号输出
utxo_set.remove_utxo("abc123", 0);
// 之后再试图花费同一个UTXO，因为它已不在集合中，验证会失败
```

实现上，`remove_utxo`使用`retain`保留其他未受影响的输出，如果某笔交易的所有输出都被花费了，则将整个txid条目一并删除：

```rust
pub fn remove_utxo(&mut self, txid: &str, vout: usize) {
    if let Some(outputs) = self.utxos.get_mut(txid) {
        outputs.retain(|(index, _)| *index != vout);
        if outputs.is_empty() {
            self.utxos.remove(txid);
        }
    }
}
```

---

### 查询地址的所有UTXO：`find_utxos`

```rust
pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)>
```

遍历整个UTXO集合，返回属于指定地址的所有未花费输出，结果格式为`(txid, vout, value)`。

```rust
let utxos = utxo_set.find_utxos("alice_address");
for (txid, vout, value) in &utxos {
    println!("UTXO: {}:{} = {} satoshis", txid, vout, value);
}
```

---

### 查找可用UTXO：`find_spendable_outputs`

```rust
pub fn find_spendable_outputs(
    &self,
    address: &str,
    amount: u64,
) -> Option<(u64, Vec<(String, usize)>)>
```

这是**创建新交易时最重要的API**。它使用贪心算法（Greedy Coin Selection），从该地址的UTXO中逐个累加，直到总额满足`amount`为止。

```rust
// 要支付 30 satoshis（含手续费）
match utxo_set.find_spendable_outputs("alice", 30) {
    Some((accumulated, inputs)) => {
        // accumulated: 实际选出的总金额（可能 > 30，差额作为找零）
        // inputs: 选中的 UTXO 列表，每项为 (txid, vout)
        let change = accumulated - 30;
        println!("选中 {} 个UTXO，找零: {} satoshis", inputs.len(), change);
    }
    None => {
        println!("余额不足");
    }
}
```

**找零机制**：若`accumulated > amount`，差额需要作为找零输出返回给发送者。例如，要支付3 BTC，选了5 BTC的UTXO，需要创建一个2 BTC的找零输出（手续费从中扣除）。

**UTXO选择策略对比**：

| 策略 | 说明 | 本实现 |
|------|------|--------|
| 贪心算法 | 顺序累加直到满足金额 | ✓ 使用此策略 |
| 最优匹配 | 最接近目标金额的组合 | 减少找零 |
| 最小UTXO优先 | 优先用小额UTXO | 减少碎片化 |
| 最大UTXO优先 | 优先用大额UTXO | 减少输入数量 |

---

### 排除已待确认UTXO：`find_spendable_outputs_excluding`

```rust
pub fn find_spendable_outputs_excluding(
    &self,
    address: &str,
    amount: u64,
    excluded: &HashSet<String>,
) -> Option<(u64, Vec<(String, usize)>)>
```

这是`find_spendable_outputs`的扩展版本。当同一个钱包在短时间内连续发起多笔交易时，先前交易已选用的UTXO尚未被确认（仍在内存池中），但已不可再用。通过传入`excluded`集合（格式为`"txid:vout"`字符串），可以跳过这些已被待确认交易占用的UTXO。

```rust
// 第一笔交易选用了 "abc:0"
let mut pending_spent: HashSet<String> = HashSet::new();
pending_spent.insert("abc:0".to_string());

// 第二笔交易自动跳过 "abc:0"
let result = utxo_set.find_spendable_outputs_excluding("alice", 20, &pending_spent);
```

---

### 查询余额：`get_balance`

```rust
pub fn get_balance(&self, address: &str) -> u64
```

比特币的"余额"是一个**计算值**，而非存储值。此方法内部调用`find_utxos`，将所有属于该地址的UTXO金额求和。

```rust
let balance = utxo_set.get_balance("alice");
println!("Alice的余额: {} satoshis", balance);
```

> 重要认知：查询余额需要扫描整个UTXO集合（时间复杂度O(n)），比特币实际节点通过地址索引来优化此操作。

---

### 完整交易处理：`process_transaction`

```rust
pub fn process_transaction(&mut self, tx: &Transaction) -> bool
```

这是UTXO状态更新的**核心函数**，按顺序执行：
1. 调用`tx.verify()`验证交易签名
2. 若非coinbase交易，移除所有输入引用的UTXO
3. 将交易的所有输出加入UTXO集合

```rust
// 处理一笔普通交易
let success = utxo_set.process_transaction(&transfer_tx);
if !success {
    eprintln!("交易验证失败，UTXO集合未变更");
}
```

此函数具备原子性语义——验证失败时不会修改UTXO集合，保证了状态一致性。

---

## 双花防止机制

双重花费（Double Spend）是区块链需要解决的核心安全问题。UTXO模型天然防御双花：

```
攻击流程：
1. 攻击者有一个价值10 BTC的UTXO（txid="xyz", vout=0）
2. 创建交易A：花费 xyz:0，付给商家10 BTC
3. 商家接受，交易A进入内存池
4. 攻击者创建交易B：同样花费 xyz:0，付给自己10 BTC
5. 尝试广播交易B

防御结果：
- 交易A确认后，xyz:0 从UTXO集合删除
- 交易B验证时找不到 xyz:0，被节点拒绝
- 即使交易A未确认，内存池的双花检测也会拒绝交易B
```

在SimpleBTC的`Mempool`中，`utxo_index`（`HashMap<"txid:vout", spending_txid>`）记录了哪些UTXO已被内存池中的交易占用，从而在第3步就能检测到双花并拒绝交易B。

---

## 完整使用示例

```rust
use bitcoin_simulation::utxo::UTXOSet;
use bitcoin_simulation::transaction::Transaction;

fn main() {
    let mut utxo_set = UTXOSet::new();

    // 步骤1：挖矿，创建coinbase交易（凭空产生比特币）
    let coinbase = Transaction::new_coinbase("alice".to_string(), 50, 0, 0);
    utxo_set.process_transaction(&coinbase);

    // 步骤2：查询Alice的余额
    let alice_balance = utxo_set.get_balance("alice");
    println!("Alice余额: {} satoshis", alice_balance); // 输出: 50

    // 步骤3：Alice向Bob转账20 satoshis（手续费2 satoshis）
    let needed = 22; // 20给Bob + 2手续费
    if let Some((accumulated, inputs)) = utxo_set.find_spendable_outputs("alice", needed) {
        let change = accumulated - needed;
        println!("选用 {} 个UTXO，总额: {}，找零: {}", inputs.len(), accumulated, change);

        // 构建并广播交易（此处省略签名细节）
        // let tx = build_transaction(inputs, "bob", 20, "alice", change, 2);
        // utxo_set.process_transaction(&tx);
    }

    // 步骤4：查询Bob的余额
    // println!("Bob余额: {}", utxo_set.get_balance("bob"));
}
```

---

## 小结

UTXO模型是比特币架构的基石。`UTXOSet`通过以下机制保证账本安全：

- **`process_transaction`**：原子性地更新UTXO状态（先删输入，再增输出）
- **`remove_utxo`**：确保每个UTXO只能被消费一次，防止双花
- **`find_spendable_outputs_excluding`**：通过`pending_spent`跟踪，解决连续交易的UTXO冲突
- **`get_balance`**：余额是计算值，是所有属于该地址的UTXO的求和

下一章将介绍交易的具体结构和签名验证机制。
