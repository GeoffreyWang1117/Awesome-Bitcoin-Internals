use sha2::{Digest, Sha256};

/// Merkle树节点
///
/// Merkle树（也称哈希树）是比特币的关键数据结构之一。
///
/// 节点类型：
/// - 叶子节点：包含交易哈希，无子节点
/// - 内部节点：包含子节点哈希的哈希，有左右子节点
/// - 根节点：树的顶部节点，其哈希存储在区块头中
///
/// 构建过程（自底向上）：
/// 1. 叶子层：每笔交易的哈希作为叶子节点
/// 2. 上层：两两配对，哈希(左哈希 + 右哈希)作为父节点
/// 3. 重复步骤2直到只剩一个节点（根节点）
/// 4. 如果某层节点数为奇数，复制最后一个节点配对
///
/// 示例（4笔交易）：
///         Root
///        /    \
///      H12    H34
///     /  \   /  \
///    H1  H2 H3  H4
///    tx1 tx2 tx3 tx4
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,                    // 节点的哈希值
    pub left: Option<Box<MerkleNode>>,   // 左子节点（内部节点才有）
    pub right: Option<Box<MerkleNode>>,  // 右子节点（内部节点才有）
}

impl MerkleNode {
    /// 创建叶子节点
    pub fn new_leaf(data: &str) -> Self {
        let hash = MerkleNode::hash_data(data);
        MerkleNode {
            hash,
            left: None,
            right: None,
        }
    }

    /// 创建内部节点
    pub fn new_internal(left: MerkleNode, right: MerkleNode) -> Self {
        let combined = format!("{}{}", left.hash, right.hash);
        let hash = MerkleNode::hash_data(&combined);
        MerkleNode {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    /// 计算数据哈希
    fn hash_data(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Merkle树
///
/// Merkle树的核心功能和优势：
///
/// 1. 高效验证（SPV - Simplified Payment Verification）：
///    - 轻钱包无需下载完整区块链（约500GB）
///    - 只需要区块头（约80字节/区块）和Merkle证明
///    - 可以验证某笔交易是否在某个区块中
///    - 验证复杂度：O(log n)，n是交易数量
///
/// 2. 数据完整性：
///    - 任何交易的修改都会改变Merkle根
///    - 根哈希存储在区块头中，受PoW保护
///    - 不可能在不改变区块哈希的情况下篡改交易
///
/// 3. 存储优化：
///    - 区块头只需存储32字节的Merkle根
///    - 而不是存储所有交易的哈希列表
///    - 减少了区块头大小，提高了传播速度
///
/// 4. SPV证明示例：
///    验证tx1在区块中：
///    - 需要：H2, H34
///    - 计算：H12 = hash(H1 + H2)
///    - 计算：Root = hash(H12 + H34)
///    - 比较Root与区块头中的merkle_root
///    - 只需3个哈希（log₂4 + 1），而不是所有4个交易
///
/// 实际应用：
/// - SPV钱包（手机钱包、轻客户端）
/// - 跨链验证
/// - Git版本控制（类似原理）
/// - IPFS内容寻址
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>,  // 树根节点
    pub leaves: Vec<String>,       // 叶子节点数据（交易哈希列表）
}

impl MerkleTree {
    /// 从交易列表构建Merkle树
    pub fn new(transactions: &[String]) -> Self {
        if transactions.is_empty() {
            return MerkleTree {
                root: None,
                leaves: vec![],
            };
        }

        let mut leaves: Vec<String> = transactions.to_vec();

        // 如果交易数量是奇数，复制最后一个
        if leaves.len() % 2 != 0 {
            leaves.push(leaves.last().unwrap().clone());
        }

        // 构建叶子节点
        let mut nodes: Vec<MerkleNode> = leaves
            .iter()
            .map(|tx| MerkleNode::new_leaf(tx))
            .collect();

        // 自底向上构建树
        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..nodes.len()).step_by(2) {
                let left = nodes[i].clone();
                let right = if i + 1 < nodes.len() {
                    nodes[i + 1].clone()
                } else {
                    // 奇数个节点，复制最后一个
                    left.clone()
                };
                next_level.push(MerkleNode::new_internal(left, right));
            }

            nodes = next_level;
        }

        MerkleTree {
            root: Some(nodes[0].clone()),
            leaves: transactions.to_vec(),
        }
    }

    /// 获取Merkle根哈希
    pub fn get_root_hash(&self) -> String {
        match &self.root {
            Some(node) => node.hash.clone(),
            None => String::new(),
        }
    }

    /// 生成Merkle证明（用于SPV验证）
    pub fn get_proof(&self, tx_hash: &str) -> Option<Vec<String>> {
        let index = self.leaves.iter().position(|tx| tx == tx_hash)?;
        let mut proof = Vec::new();
        let mut current_index = index;
        let mut level_size = self.leaves.len();

        // 确保是偶数
        let mut leaves = self.leaves.clone();
        if leaves.len() % 2 != 0 {
            leaves.push(leaves.last().unwrap().clone());
        }

        let mut current_level = leaves;

        while current_level.len() > 1 {
            // 找到兄弟节点的索引
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index].clone());
            }

            // 计算下一层
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    left
                };
                let combined = format!("{}{}",
                    MerkleNode::hash_data(left),
                    MerkleNode::hash_data(right)
                );
                next_level.push(MerkleNode::hash_data(&combined));
            }

            current_index /= 2;
            current_level = next_level;
        }

        Some(proof)
    }

    /// 验证Merkle证明（SPV核心功能）
    ///
    /// 这是SPV（轻量级支付验证）的核心函数。
    ///
    /// 验证过程：
    /// 1. 从交易哈希开始（叶子节点）
    /// 2. 使用证明中的兄弟哈希，逐层向上计算
    /// 3. 根据节点索引确定左右顺序
    /// 4. 最终计算出的根哈希应该与区块头中的merkle_root匹配
    ///
    /// 示例（验证tx1，index=0）：
    /// - 已知：tx1_hash, 证明=[H2, H34]
    /// - 步骤1：H12 = hash(tx1_hash + H2)  // index=0，tx1在左边
    /// - 步骤2：Root = hash(H12 + H34)     // index=0，H12在左边
    /// - 验证：计算的Root == 区块的merkle_root
    ///
    /// 为什么安全：
    /// - 攻击者无法伪造证明，因为：
    ///   1. 需要找到哈希碰撞（SHA256碰撞，计算上不可行）
    ///   2. 或者需要重新挖矿（改变merkle_root会改变区块哈希）
    /// - 轻客户端可以信任POW最长的链
    ///
    /// 效率对比（验证1笔交易）：
    /// - 全节点：下载整个区块（1-2MB），验证所有交易
    /// - SPV：下载证明（几KB），O(log n)次哈希计算
    ///
    /// # 参数
    /// * `tx_hash` - 要验证的交易哈希
    /// * `proof` - Merkle证明（兄弟节点哈希列表）
    /// * `root_hash` - 区块头中的Merkle根哈希
    /// * `index` - 交易在区块中的索引位置
    ///
    /// # 返回值
    /// true - 交易确实在该区块中
    /// false - 交易不在该区块中，或证明无效
    pub fn verify_proof(
        tx_hash: &str,
        proof: &[String],
        root_hash: &str,
        index: usize,
    ) -> bool {
        let mut current_hash = MerkleNode::hash_data(tx_hash);
        let mut current_index = index;

        for sibling_hash in proof {
            let combined = if current_index % 2 == 0 {
                format!("{}{}", current_hash, sibling_hash)
            } else {
                format!("{}{}", sibling_hash, current_hash)
            };
            current_hash = MerkleNode::hash_data(&combined);
            current_index /= 2;
        }

        current_hash == root_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let txs = vec![
            "tx1".to_string(),
            "tx2".to_string(),
            "tx3".to_string(),
            "tx4".to_string(),
        ];

        let tree = MerkleTree::new(&txs);
        assert!(tree.root.is_some());

        let root_hash = tree.get_root_hash();
        assert!(!root_hash.is_empty());

        // 测试证明
        let proof = tree.get_proof("tx1").unwrap();
        assert!(MerkleTree::verify_proof("tx1", &proof, &root_hash, 0));
    }
}
