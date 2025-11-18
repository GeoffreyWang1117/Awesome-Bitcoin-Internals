# UTXO API

UTXO模块管理所有未花费的交易输出，是比特币账户模型的核心。

## 数据结构

### `UTXOSet`

```rust
pub struct UTXOSet {
    // key: txid (交易ID)
    // value: Vec<(vout_index, TxOutput)>
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
}
```

**内部结构**:
```
HashMap {
    "tx1": [(0, Output{value: 100, addr: "alice"}),
            (1, Output{value: 50, addr: "bob"})],
    "tx2": [(0, Output{value: 200, addr: "charlie"})],
}
```

---

## 核心概念

### UTXO模型 vs 账户模型

#### 账户模型（以太坊）
```
账户余额:
  Alice: 100 BTC
  Bob: 50 BTC

转账: Alice → Bob (30 BTC)
  Alice: 70 BTC  (-30)
  Bob: 80 BTC    (+30)
```

#### UTXO模型（比特币）
```
UTXO集合:
  tx1:0 → 100 BTC (Alice)
  tx1:1 → 50 BTC (Bob)

转账: Alice → Bob (30 BTC)
  消费: tx1:0 (100 BTC)
  创建:
    tx2:0 → 30 BTC (Bob)
    tx2:1 → 70 BTC (Alice, 找零)

新UTXO集合:
  tx1:1 → 50 BTC (Bob)
  tx2:0 → 30 BTC (Bob)
  tx2:1 → 70 BTC (Alice)
```

### UTXO的优势

1. **更好的隐私**
   - 每次可用新地址
   - 难以追踪资金流向

2. **并行处理**
   - 不同UTXO可并发验证
   - 无账户锁定问题

3. **简化验证**
   - 只需检查UTXO存在
   - 无需账户历史

---

## 方法

### 初始化

#### `new`

```rust
pub fn new() -> Self
```

创建空的UTXO集合。

**示例**:
```rust
use bitcoin_simulation::utxo::UTXOSet;

let mut utxo_set = UTXOSet::new();
```

---

### UTXO管理

#### `add_transaction`

```rust
pub fn add_transaction(&mut self, tx: &Transaction)
```

将交易的所有输出添加到UTXO集合。

**过程**:
1. 遍历交易的所有输出
2. 将每个输出标记为未花费
3. 添加到UTXO集合

**参数**:
- `tx` - 要添加的交易

**注意**: 只添加输出，不处理输入

**示例**:
```rust
let tx = Transaction::new(...);
utxo_set.add_transaction(&tx);

// 现在tx的所有输出都可以被花费
```

---

#### `remove_utxo`

```rust
pub fn remove_utxo(&mut self, txid: &str, vout: usize)
```

移除已花费的UTXO。

**双花防护**:
- UTXO只能花费一次
- 花费后立即从集合移除
- 二次引用会失败

**参数**:
- `txid` - 交易ID
- `vout` - 输出索引

**示例**:
```rust
// 花费UTXO
utxo_set.remove_utxo("tx1", 0);

// 再次尝试花费（失败）
// UTXO不存在
```

---

#### `process_transaction`

```rust
pub fn process_transaction(&mut self, tx: &Transaction) -> bool
```

完整处理交易（移除输入，添加输出）。

**ACID特性**:

**Atomicity（原子性）**:
```rust
// 要么完全成功，要么完全失败
if !tx.verify() {
    return false;  // 不进行任何修改
}
// 全部处理
```

**Consistency（一致性）**:
```rust
// 处理前后，输入总额 = 输出总额 + 手续费
assert_eq!(input_sum, output_sum + fee);
```

**步骤**:
1. 验证交易有效性
2. 移除输入引用的UTXO
3. 添加新创建的输出

**参数**:
- `tx` - 要处理的交易

**返回值**:
- `true` - 处理成功
- `false` - 交易无效

**示例**:
```rust
let tx = blockchain.create_transaction(&alice, bob.address, 1000, 10)?;

if utxo_set.process_transaction(&tx) {
    println!("✓ UTXO更新成功");
} else {
    println!("✗ 交易无效");
}
```

---

### 查询操作

#### `find_utxos`

```rust
pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)>
```

查找地址的所有UTXO。

**返回格式**: `Vec<(txid, vout, value)>`

**参数**:
- `address` - 要查询的地址

**返回值**: UTXO列表

**示例**:
```rust
let utxos = utxo_set.find_utxos(&alice.address);

println!("Alice的UTXO:");
for (txid, vout, value) in utxos {
    println!("  {}:{} → {} sat", &txid[..8], vout, value);
}

// 输出:
// Alice的UTXO:
//   tx1:0 → 5000 sat
//   tx2:1 → 3000 sat
//   tx5:0 → 2000 sat
```

---

#### `find_spendable_outputs`

```rust
pub fn find_spendable_outputs(
    &self,
    address: &str,
    amount: u64
) -> Option<(u64, Vec<(String, usize)>)>
```

查找可用于支付的UTXO组合。

**UTXO选择策略**:

1. **贪心算法**（当前实现）:
   ```rust
   accumulated = 0
   for utxo in utxos:
       accumulated += utxo.value
       if accumulated >= amount:
           return utxos
   ```

2. **最优匹配**（可优化）:
   - 选择总额最接近目标的组合
   - 减少找零，节省手续费

3. **最小UTXO优先**:
   - 优先使用小额UTXO
   - 避免UTXO碎片化

**参数**:
- `address` - 发送者地址
- `amount` - 需要的金额（包括手续费）

**返回值**:
- `Some((accumulated, utxo_list))` - 找到足够UTXO
  - `accumulated`: 累积金额
  - `utxo_list`: 选中的UTXO列表
- `None` - 余额不足

**示例**:
```rust
// 需要1000 sat（含手续费）
let result = utxo_set.find_spendable_outputs(&alice.address, 1000);

match result {
    Some((accumulated, utxos)) => {
        println!("✓ 找到足够的UTXO");
        println!("  累积金额: {} sat", accumulated);
        println!("  使用UTXO数: {}", utxos.len());
        println!("  找零: {} sat", accumulated - 1000);
    }
    None => {
        println!("✗ 余额不足");
    }
}
```

---

#### `get_balance`

```rust
pub fn get_balance(&self, address: &str) -> u64
```

计算地址的总余额。

**计算方式**:
```rust
balance = sum(all_utxos.value)
```

**参数**:
- `address` - 要查询的地址

**返回值**: 余额（satoshi）

**示例**:
```rust
let balance = utxo_set.get_balance(&alice.address);
println!("余额: {} satoshi", balance);
println!("余额: {:.8} BTC", balance as f64 / 100_000_000.0);

// 批量查询
let addresses = vec![alice.address, bob.address, charlie.address];
for addr in addresses {
    let bal = utxo_set.get_balance(&addr);
    println!("{}: {} sat", &addr[..10], bal);
}
```

---

## 使用场景

### 场景1: 创建交易时选择UTXO

```rust
fn create_payment(
    utxo_set: &UTXOSet,
    from: &Wallet,
    to: &str,
    amount: u64,
    fee: u64
) -> Result<Transaction, String> {
    let total_needed = amount + fee;

    // 1. 查找可用UTXO
    let result = utxo_set.find_spendable_outputs(&from.address, total_needed);

    let (accumulated, utxo_refs) = result.ok_or("余额不足")?;

    // 2. 构建输入
    let mut inputs = Vec::new();
    for (txid, vout) in utxo_refs {
        let signature = from.sign(&format!("{}{}", txid, vout));
        inputs.push(TxInput::new(txid, vout, signature, from.public_key.clone()));
    }

    // 3. 构建输出
    let mut outputs = vec![
        TxOutput::new(amount, to.to_string()),  // 给接收者
    ];

    // 4. 找零
    if accumulated > total_needed {
        outputs.push(TxOutput::new(
            accumulated - total_needed,
            from.address.clone()
        ));
    }

    // 5. 创建交易
    Ok(Transaction::new(inputs, outputs, current_timestamp(), fee))
}
```

### 场景2: 查询余额详情

```rust
fn balance_breakdown(utxo_set: &UTXOSet, address: &str) {
    let utxos = utxo_set.find_utxos(address);
    let total = utxo_set.get_balance(address);

    println!("=== 余额详情 ===");
    println!("地址: {}", &address[..20]);
    println!("总余额: {} sat ({:.8} BTC)", total, total as f64 / 1e8);
    println!("UTXO数量: {}", utxos.len());
    println!("\nUTXO列表:");

    for (i, (txid, vout, value)) in utxos.iter().enumerate() {
        println!("  #{}: {}:{} → {} sat",
            i + 1, &txid[..8], vout, value);
    }

    // 统计
    if !utxos.is_empty() {
        let avg = total / utxos.len() as u64;
        let max = utxos.iter().map(|(_, _, v)| v).max().unwrap();
        let min = utxos.iter().map(|(_, _, v)| v).min().unwrap();

        println!("\n统计:");
        println!("  平均: {} sat", avg);
        println!("  最大: {} sat", max);
        println!("  最小: {} sat", min);
    }
}
```

### 场景3: UTXO碎片整理

```rust
fn consolidate_utxos(
    blockchain: &mut Blockchain,
    wallet: &Wallet
) -> Result<(), String> {
    let utxos = blockchain.utxo_set.find_utxos(&wallet.address);

    // 如果UTXO太多（>50个），整理成1个
    if utxos.len() > 50 {
        println!("开始整理UTXO...");
        println!("  当前UTXO数: {}", utxos.len());

        // 创建自己给自己的交易，整理所有UTXO
        let total = blockchain.get_balance(&wallet.address);
        let fee = 100;  // 固定手续费

        let tx = blockchain.create_transaction(
            wallet,
            wallet.address.clone(),
            total - fee,
            fee,
        )?;

        blockchain.add_transaction(tx)?;
        blockchain.mine_pending_transactions(wallet.address.clone())?;

        let new_utxos = blockchain.utxo_set.find_utxos(&wallet.address);
        println!("✓ 整理完成");
        println!("  新UTXO数: {}", new_utxos.len());
    }

    Ok(())
}
```

### 场景4: UTXO审计

```rust
fn audit_utxo_set(utxo_set: &UTXOSet, blockchain: &Blockchain) -> bool {
    println!("=== UTXO审计 ===");

    // 1. 统计UTXO总数
    let total_utxos: usize = utxo_set.utxos.values()
        .map(|v| v.len())
        .sum();
    println!("总UTXO数: {}", total_utxos);

    // 2. 统计总价值
    let mut total_value = 0u64;
    for outputs in utxo_set.utxos.values() {
        for (_, output) in outputs {
            total_value += output.value;
        }
    }
    println!("总价值: {} sat", total_value);

    // 3. 验证每个UTXO
    let mut valid = true;
    for (txid, outputs) in &utxo_set.utxos {
        // 验证交易存在于区块链
        let tx_exists = blockchain.chain.iter()
            .any(|block| block.transactions.iter()
                .any(|tx| &tx.id == txid));

        if !tx_exists {
            println!("✗ 警告: UTXO引用不存在的交易 {}", txid);
            valid = false;
        }
    }

    if valid {
        println!("✓ UTXO集合完整");
    }

    valid
}
```

---

## 性能优化

### 1. 索引优化

```rust
// 为地址创建索引
pub struct IndexedUTXOSet {
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
    // 新增：地址索引
    address_index: HashMap<String, Vec<(String, usize)>>,
}

impl IndexedUTXOSet {
    pub fn find_utxos(&self, address: &str) -> Vec<(String, usize, u64)> {
        // O(1) 查找而不是 O(n)
        if let Some(refs) = self.address_index.get(address) {
            refs.iter()
                .filter_map(|(txid, vout)| {
                    self.utxos.get(txid)
                        .and_then(|outputs| outputs.iter()
                            .find(|(idx, _)| idx == vout)
                            .map(|(_, output)| (txid.clone(), *vout, output.value))
                        )
                })
                .collect()
        } else {
            vec![]
        }
    }
}
```

### 2. 批量操作

```rust
// 批量处理交易
pub fn process_transactions(&mut self, txs: &[Transaction]) -> bool {
    // 1. 验证所有交易
    for tx in txs {
        if !tx.verify() {
            return false;
        }
    }

    // 2. 批量更新UTXO
    for tx in txs {
        // 移除输入
        if !tx.is_coinbase() {
            for input in &tx.inputs {
                self.remove_utxo(&input.txid, input.vout);
            }
        }

        // 添加输出
        self.add_transaction(tx);
    }

    true
}
```

### 3. 缓存余额

```rust
pub struct CachedUTXOSet {
    utxos: HashMap<String, Vec<(usize, TxOutput)>>,
    balance_cache: HashMap<String, u64>,  // 余额缓存
}

impl CachedUTXOSet {
    pub fn get_balance(&mut self, address: &str) -> u64 {
        // 检查缓存
        if let Some(balance) = self.balance_cache.get(address) {
            return *balance;
        }

        // 计算并缓存
        let balance = self.calculate_balance(address);
        self.balance_cache.insert(address.to_string(), balance);
        balance
    }

    fn invalidate_cache(&mut self, address: &str) {
        self.balance_cache.remove(address);
    }
}
```

---

## 与以太坊账户模型对比

| 特性 | UTXO模型（比特币） | 账户模型（以太坊） |
|------|------------------|-------------------|
| 状态 | 无状态（只有UTXO集合） | 有状态（账户余额、nonce） |
| 余额 | 计算值（UTXO总和） | 存储值（直接存储） |
| 转账 | 消费UTXO，创建新UTXO | 账户余额增减 |
| 隐私 | 较好（可用新地址） | 较差（重复使用地址） |
| 并行 | 易于并行验证 | 需要顺序处理（nonce） |
| 复杂度 | 交易构建复杂 | 交易简单 |
| 智能合约 | 有限（Script） | 灵活（EVM） |

---

## 常见问题

### Q: 为什么要用UTXO模型？

**A:**
- ✅ 更好的隐私（每次用新地址）
- ✅ 并行验证（不同UTXO独立）
- ✅ 简化的验证逻辑
- ✅ 防双花机制天然

### Q: UTXO会越来越多吗？

**A:** 是的。解决方案：
- UTXO整理（将多个小UTXO合并）
- 提高交易费（限制垃圾UTXO）
- UTXO承诺（减少存储）

### Q: 如何防止UTXO碎片化？

**A:**
```rust
// 定期整理
if utxos.len() > threshold {
    consolidate_utxos();
}

// 优先使用小UTXO
utxos.sort_by_key(|u| u.value);  // 小的优先
```

### Q: UTXO丢失怎么办？

**A:** 只要有私钥，可以从区块链重建UTXO集合：
```rust
fn rebuild_utxo_set(blockchain: &Blockchain, address: &str) -> UTXOSet {
    let mut utxo_set = UTXOSet::new();

    for block in &blockchain.chain {
        for tx in &block.transactions {
            utxo_set.process_transaction(tx);
        }
    }

    utxo_set
}
```

---

## 参考

- [Transaction API](./transaction.md) - UTXO的创建和消费
- [Blockchain API](./blockchain.md) - UTXO集合管理
- [基本概念 - UTXO模型](../guide/concepts.md#utxo模型)
- [比特币白皮书](https://bitcoin.org/bitcoin.pdf)

---

[返回API目录](./core.md)
