use sha2::{Digest, Sha256};

/// Merkle树节点
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
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
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>,
    pub leaves: Vec<String>,
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

    /// 验证Merkle证明
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
