//! P2P网络层
//!
//! 实现比特币节点间的点对点通信，包括：
//! - **节点发现**: 通过种子节点连接网络
//! - **区块广播**: 新区块挖出后广播给所有节点
//! - **交易广播**: 新交易创建后广播给所有节点
//! - **链同步**: 新节点加入时同步完整区块链
//!
//! # 协议设计
//!
//! 使用基于TCP的JSON消息协议，每条消息以换行符分隔。
//! 消息类型：
//! - `Version`: 握手消息，交换节点信息
//! - `VerAck`: 握手确认
//! - `GetBlocks`: 请求区块（从指定高度开始）
//! - `Blocks`: 返回区块数据
//! - `NewBlock`: 广播新区块
//! - `NewTransaction`: 广播新交易
//! - `Ping`/`Pong`: 心跳检测
//!
//! # 示例
//!
//! ```no_run
//! use bitcoin_simulation::network::P2PNode;
//! use bitcoin_simulation::blockchain::Blockchain;
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! #[tokio::main]
//! async fn main() {
//!     let blockchain = Arc::new(Mutex::new(Blockchain::new()));
//!     let node = P2PNode::new("0.0.0.0:8333".to_string(), vec![], blockchain);
//!     // node.start().await;
//! }
//! ```

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex, RwLock};

/// 网络协议版本
const PROTOCOL_VERSION: u32 = 1;

/// 最大消息大小 (10 MB)
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// 心跳间隔（秒）
const PING_INTERVAL_SECS: u64 = 30;

/// 连接超时（秒）
const CONNECT_TIMEOUT_SECS: u64 = 5;

// ============ 网络消息类型 ============

/// P2P网络消息
///
/// 比特币P2P协议的简化版本。真实比特币使用二进制协议，
/// 这里使用JSON以保持可读性和教育性。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    /// 版本握手 — 连接时交换节点信息
    Version {
        version: u32,
        height: usize,
        addr_from: String,
    },

    /// 版本确认
    VerAck,

    /// 请求区块（从指定高度开始）
    GetBlocks { from_height: usize },

    /// 返回区块数据
    Blocks { blocks: Vec<Block> },

    /// 广播新区块（刚挖出的）
    NewBlock { block: Block },

    /// 广播新交易
    NewTransaction { transaction: Transaction },

    /// 心跳请求
    Ping { nonce: u64 },

    /// 心跳响应
    Pong { nonce: u64 },

    /// 请求已知节点列表
    GetPeers,

    /// 返回已知节点列表
    Peers { addresses: Vec<String> },
}

/// 节点信息
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub address: String,
    pub height: usize,
    pub version: u32,
}

/// 节点事件（用于通知上层）
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// 收到新区块
    NewBlock(Block),
    /// 收到新交易
    NewTransaction(Transaction),
    /// 新节点连接
    PeerConnected(String),
    /// 节点断开
    PeerDisconnected(String),
    /// 链同步完成
    SyncComplete,
}

// ============ P2P节点 ============

/// P2P网络节点
///
/// 管理与其他节点的TCP连接，处理消息收发和区块链同步。
///
/// 架构：
/// ```text
/// ┌─────────────┐
/// │   P2PNode   │
/// │ ┌─────────┐ │     TCP      ┌─────────┐
/// │ │Listener │◄├────────────►│  Peer 1  │
/// │ └─────────┘ │              └─────────┘
/// │ ┌─────────┐ │     TCP      ┌─────────┐
/// │ │Broadcast│◄├────────────►│  Peer 2  │
/// │ └─────────┘ │              └─────────┘
/// │ ┌─────────┐ │     TCP      ┌─────────┐
/// │ │  Chain  │◄├────────────►│  Peer N  │
/// │ └─────────┘ │              └─────────┘
/// └─────────────┘
/// ```
pub struct P2PNode {
    /// 监听地址
    listen_addr: String,

    /// 种子节点
    seed_peers: Vec<String>,

    /// 区块链实例（共享）
    blockchain: Arc<Mutex<Blockchain>>,

    /// 已连接的节点地址
    connected_peers: Arc<RwLock<HashSet<String>>>,

    /// 广播通道（向所有节点广播消息）
    broadcast_tx: broadcast::Sender<NetMessage>,

    /// 事件通道
    event_tx: broadcast::Sender<NodeEvent>,

    /// 已知的交易ID（防重复广播）
    known_txids: Arc<RwLock<HashSet<String>>>,

    /// 已知的区块哈希（防重复广播）
    known_blocks: Arc<RwLock<HashSet<String>>>,
}

impl P2PNode {
    /// 创建新的P2P节点
    pub fn new(
        listen_addr: String,
        seed_peers: Vec<String>,
        blockchain: Arc<Mutex<Blockchain>>,
    ) -> Self {
        let (broadcast_tx, _) = broadcast::channel(256);
        let (event_tx, _) = broadcast::channel(256);

        Self {
            listen_addr,
            seed_peers,
            blockchain,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            broadcast_tx,
            event_tx,
            known_txids: Arc::new(RwLock::new(HashSet::new())),
            known_blocks: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// 获取事件接收器
    pub fn subscribe_events(&self) -> broadcast::Receiver<NodeEvent> {
        self.event_tx.subscribe()
    }

    /// 获取已连接节点数
    pub async fn peer_count(&self) -> usize {
        self.connected_peers.read().await.len()
    }

    /// 获取已连接节点列表
    pub async fn peers(&self) -> Vec<String> {
        self.connected_peers.read().await.iter().cloned().collect()
    }

    /// 启动P2P节点
    ///
    /// 1. 启动TCP监听器，接受入站连接
    /// 2. 连接到种子节点
    /// 3. 启动心跳定时器
    pub async fn start(&self) -> crate::error::Result<()> {
        println!("[P2P] Starting node on {}", self.listen_addr);

        // 启动TCP监听
        let listener = TcpListener::bind(&self.listen_addr).await.map_err(|e| {
            crate::error::BitcoinError::NetworkError {
                reason: format!("Failed to bind {}: {}", self.listen_addr, e),
            }
        })?;

        println!("[P2P] Listening on {}", self.listen_addr);

        // 连接种子节点（异步）
        for seed in &self.seed_peers {
            let seed = seed.clone();
            let blockchain = Arc::clone(&self.blockchain);
            let listen_addr = self.listen_addr.clone();
            let node = self.clone_handles();
            tokio::spawn(async move {
                println!("[P2P] Connecting to seed {}...", seed);
                let stream = tokio::time::timeout(
                    std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS),
                    TcpStream::connect(&seed),
                )
                .await;

                match stream {
                    Ok(Ok(stream)) => {
                        let height = blockchain.lock().await.chain.len();
                        let version_msg = NetMessage::Version {
                            version: PROTOCOL_VERSION,
                            height,
                            addr_from: listen_addr,
                        };
                        if let Err(e) = node
                            .handle_connection_with_handshake(
                                stream,
                                seed.clone(),
                                Some(version_msg),
                            )
                            .await
                        {
                            eprintln!("[P2P] Error with seed {}: {}", seed, e);
                        }
                    }
                    _ => {
                        eprintln!("[P2P] Failed to connect to seed {}", seed);
                    }
                }
            });
        }

        // 启动心跳定时器
        let broadcast_tx = self.broadcast_tx.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(PING_INTERVAL_SECS));
            loop {
                interval.tick().await;
                let nonce = rand::random::<u64>();
                let _ = broadcast_tx.send(NetMessage::Ping { nonce });
            }
        });

        // 接受入站连接
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let peer_addr = addr.to_string();
                    println!("[P2P] Incoming connection from {}", peer_addr);
                    let node = self.clone_handles();
                    tokio::spawn(async move {
                        if let Err(e) = node.handle_connection(stream, peer_addr.clone()).await {
                            eprintln!("[P2P] Connection error with {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[P2P] Accept error: {}", e);
                }
            }
        }
    }

    /// 广播新区块给所有节点
    pub async fn broadcast_block(&self, block: &Block) {
        let hash = block.hash.clone();
        {
            let mut known = self.known_blocks.write().await;
            if known.contains(&hash) {
                return; // 已知区块，不重复广播
            }
            known.insert(hash);
        }

        let msg = NetMessage::NewBlock {
            block: block.clone(),
        };
        let _ = self.broadcast_tx.send(msg);
    }

    /// 广播新交易给所有节点
    pub async fn broadcast_transaction(&self, tx: &Transaction) {
        let txid = tx.id.clone();
        {
            let mut known = self.known_txids.write().await;
            if known.contains(&txid) {
                return;
            }
            known.insert(txid);
        }

        let msg = NetMessage::NewTransaction {
            transaction: tx.clone(),
        };
        let _ = self.broadcast_tx.send(msg);
    }

    /// 创建共享句柄（用于spawn到新任务）
    fn clone_handles(&self) -> P2PNodeHandle {
        P2PNodeHandle {
            listen_addr: self.listen_addr.clone(),
            blockchain: Arc::clone(&self.blockchain),
            connected_peers: Arc::clone(&self.connected_peers),
            broadcast_tx: self.broadcast_tx.clone(),
            event_tx: self.event_tx.clone(),
            known_txids: Arc::clone(&self.known_txids),
            known_blocks: Arc::clone(&self.known_blocks),
        }
    }
}

// ============ 节点句柄（轻量级，可clone到异步任务） ============

/// P2P节点句柄
///
/// 包含处理单个连接所需的所有共享状态。
#[derive(Clone)]
struct P2PNodeHandle {
    listen_addr: String,
    blockchain: Arc<Mutex<Blockchain>>,
    connected_peers: Arc<RwLock<HashSet<String>>>,
    broadcast_tx: broadcast::Sender<NetMessage>,
    event_tx: broadcast::Sender<NodeEvent>,
    known_txids: Arc<RwLock<HashSet<String>>>,
    known_blocks: Arc<RwLock<HashSet<String>>>,
}

impl P2PNodeHandle {
    /// 处理入站连接（等待对方握手）
    async fn handle_connection(
        &self,
        stream: TcpStream,
        peer_addr: String,
    ) -> crate::error::Result<()> {
        self.handle_connection_with_handshake(stream, peer_addr, None)
            .await
    }

    /// 处理连接（可选发送初始握手）
    async fn handle_connection_with_handshake(
        &self,
        stream: TcpStream,
        peer_addr: String,
        initial_msg: Option<NetMessage>,
    ) -> crate::error::Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // 发送初始消息（出站连接时发送Version）
        if let Some(msg) = initial_msg {
            let data = serde_json::to_string(&msg).map_err(|e| {
                crate::error::BitcoinError::NetworkError {
                    reason: format!("Serialize error: {}", e),
                }
            })?;
            writer.write_all(data.as_bytes()).await.map_err(|e| {
                crate::error::BitcoinError::NetworkError {
                    reason: format!("Write error: {}", e),
                }
            })?;
            writer.write_all(b"\n").await.map_err(|e| {
                crate::error::BitcoinError::NetworkError {
                    reason: format!("Write error: {}", e),
                }
            })?;
        }

        // 注册节点
        self.connected_peers.write().await.insert(peer_addr.clone());
        let _ = self
            .event_tx
            .send(NodeEvent::PeerConnected(peer_addr.clone()));
        println!(
            "[P2P] Peer connected: {} (total: {})",
            peer_addr,
            self.connected_peers.read().await.len()
        );

        // 订阅广播消息（转发给此节点）
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        let mut line_buf = String::new();

        loop {
            tokio::select! {
                // 读取来自节点的消息
                result = reader.read_line(&mut line_buf) => {
                    match result {
                        Ok(0) => {
                            // 连接关闭
                            break;
                        }
                        Ok(n) => {
                            if n > MAX_MESSAGE_SIZE {
                                eprintln!("[P2P] Message too large from {}", peer_addr);
                                line_buf.clear();
                                continue;
                            }

                            let trimmed = line_buf.trim();
                            if trimmed.is_empty() {
                                line_buf.clear();
                                continue;
                            }

                            match serde_json::from_str::<NetMessage>(trimmed) {
                                Ok(msg) => {
                                    if let Some(response) = self.handle_message(msg, &peer_addr).await {
                                        let data = serde_json::to_string(&response).unwrap();
                                        if writer.write_all(data.as_bytes()).await.is_err() {
                                            break;
                                        }
                                        if writer.write_all(b"\n").await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[P2P] Invalid message from {}: {}", peer_addr, e);
                                }
                            }
                            line_buf.clear();
                        }
                        Err(e) => {
                            eprintln!("[P2P] Read error from {}: {}", peer_addr, e);
                            break;
                        }
                    }
                }
                // 转发广播消息给此节点
                result = broadcast_rx.recv() => {
                    if let Ok(msg) = result {
                        let data = serde_json::to_string(&msg).unwrap();
                        if writer.write_all(data.as_bytes()).await.is_err() {
                            break;
                        }
                        if writer.write_all(b"\n").await.is_err() {
                            break;
                        }
                    }
                }
            }
        }

        // 清理
        self.connected_peers.write().await.remove(&peer_addr);
        let _ = self
            .event_tx
            .send(NodeEvent::PeerDisconnected(peer_addr.clone()));
        println!(
            "[P2P] Peer disconnected: {} (total: {})",
            peer_addr,
            self.connected_peers.read().await.len()
        );

        Ok(())
    }

    /// 处理收到的消息，返回可选的响应
    async fn handle_message(&self, msg: NetMessage, from: &str) -> Option<NetMessage> {
        match msg {
            NetMessage::Version {
                version,
                height,
                addr_from,
            } => {
                println!(
                    "[P2P] Version from {} (v{}, height: {})",
                    addr_from, version, height
                );

                // 如果对方链更长，请求同步
                let our_height = {
                    let bc = self.blockchain.lock().await;
                    bc.chain.len()
                };

                if height > our_height {
                    println!(
                        "[P2P] Peer {} has longer chain ({} > {}), requesting sync...",
                        from, height, our_height
                    );
                    // 返回VerAck + 随后请求区块（通过广播通道）
                    let _ = self.broadcast_tx.send(NetMessage::GetBlocks {
                        from_height: our_height,
                    });
                }

                Some(NetMessage::VerAck)
            }

            NetMessage::VerAck => {
                println!("[P2P] Handshake complete with {}", from);
                None
            }

            NetMessage::GetBlocks { from_height } => {
                println!("[P2P] {} requests blocks from height {}", from, from_height);
                let bc = self.blockchain.lock().await;
                let blocks: Vec<Block> = bc.chain[from_height..].to_vec();
                Some(NetMessage::Blocks { blocks })
            }

            NetMessage::Blocks { blocks } => {
                if blocks.is_empty() {
                    return None;
                }

                println!("[P2P] Received {} blocks from {}", blocks.len(), from);

                let mut bc = self.blockchain.lock().await;
                let our_height = bc.chain.len();

                // 验证并追加区块
                let mut added = 0;
                for block in blocks {
                    // 只接受比我们链更高的区块
                    if block.index as usize >= our_height + added {
                        // 基本验证
                        if block.validate_transactions() {
                            // 更新UTXO
                            for tx in &block.transactions {
                                bc.utxo_set.process_transaction(tx);
                            }
                            bc.indexer.index_block(&block);
                            bc.chain.push(block);
                            added += 1;
                        }
                    }
                }

                if added > 0 {
                    println!(
                        "[P2P] Synced {} blocks, new height: {}",
                        added,
                        bc.chain.len()
                    );
                    let _ = self.event_tx.send(NodeEvent::SyncComplete);
                }

                None
            }

            NetMessage::NewBlock { block } => {
                let hash = block.hash.clone();

                // 检查是否已知
                {
                    let mut known = self.known_blocks.write().await;
                    if known.contains(&hash) {
                        return None;
                    }
                    known.insert(hash.clone());
                }

                println!(
                    "[P2P] New block from {}: #{} ({})",
                    from,
                    block.index,
                    &block.hash[..16]
                );

                // 验证并追加
                let mut bc = self.blockchain.lock().await;
                let expected_index = bc.chain.len() as u32;

                if block.index == expected_index && block.validate_transactions() {
                    let expected_prev = &bc.chain.last().unwrap().hash;
                    if block.previous_hash == *expected_prev {
                        // 更新UTXO
                        for tx in &block.transactions {
                            bc.utxo_set.process_transaction(tx);
                        }

                        // 从内存池移除已确认的交易
                        for tx in &block.transactions {
                            if !tx.is_coinbase() {
                                let _ = bc.mempool.remove_transaction(&tx.id);
                            }
                        }

                        bc.indexer.index_block(&block);
                        let _ = self.event_tx.send(NodeEvent::NewBlock(block.clone()));
                        bc.chain.push(block);

                        println!("[P2P] Accepted block, height: {}", bc.chain.len());
                    }
                }

                None
            }

            NetMessage::NewTransaction { transaction } => {
                let txid = transaction.id.clone();

                // 检查是否已知
                {
                    let mut known = self.known_txids.write().await;
                    if known.contains(&txid) {
                        return None;
                    }
                    known.insert(txid.clone());
                }

                // 验证并添加到内存池
                let mut bc = self.blockchain.lock().await;
                match bc.add_transaction(transaction.clone()) {
                    Ok(_) => {
                        println!("[P2P] Accepted transaction {} from {}", &txid[..16], from);
                        let _ = self.event_tx.send(NodeEvent::NewTransaction(transaction));
                    }
                    Err(e) => {
                        eprintln!(
                            "[P2P] Rejected transaction {} from {}: {}",
                            &txid[..16],
                            from,
                            e
                        );
                    }
                }

                None
            }

            NetMessage::Ping { nonce } => Some(NetMessage::Pong { nonce }),

            NetMessage::Pong { .. } => {
                // 心跳响应，可用于延迟计算
                None
            }

            NetMessage::GetPeers => {
                let peers: Vec<String> =
                    self.connected_peers.read().await.iter().cloned().collect();
                Some(NetMessage::Peers { addresses: peers })
            }

            NetMessage::Peers { addresses } => {
                // 记录新发现的节点地址（连接由上层逻辑处理）
                let known = self.connected_peers.read().await;
                let new_peers: Vec<String> = addresses
                    .into_iter()
                    .filter(|a| !known.contains(a) && *a != self.listen_addr)
                    .collect();
                if !new_peers.is_empty() {
                    println!("[P2P] Discovered {} new peers", new_peers.len());
                }
                None
            }
        }
    }
}

/// P2P节点网络统计
#[derive(Debug, Clone, Serialize)]
pub struct NetworkStats {
    /// 监听地址
    pub listen_addr: String,
    /// 已连接节点数
    pub peer_count: usize,
    /// 已连接节点列表
    pub peers: Vec<String>,
    /// 已知交易数
    pub known_transactions: usize,
    /// 已知区块数
    pub known_blocks: usize,
}

impl P2PNode {
    /// 获取网络统计信息
    pub async fn get_stats(&self) -> NetworkStats {
        NetworkStats {
            listen_addr: self.listen_addr.clone(),
            peer_count: self.connected_peers.read().await.len(),
            peers: self.connected_peers.read().await.iter().cloned().collect(),
            known_transactions: self.known_txids.read().await.len(),
            known_blocks: self.known_blocks.read().await.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = NetMessage::Version {
            version: PROTOCOL_VERSION,
            height: 10,
            addr_from: "127.0.0.1:8333".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            NetMessage::Version {
                version,
                height,
                addr_from,
            } => {
                assert_eq!(version, PROTOCOL_VERSION);
                assert_eq!(height, 10);
                assert_eq!(addr_from, "127.0.0.1:8333");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_ping_pong_serialization() {
        let ping = NetMessage::Ping { nonce: 12345 };
        let json = serde_json::to_string(&ping).unwrap();
        let pong_expected = NetMessage::Pong { nonce: 12345 };

        let deserialized: NetMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            NetMessage::Ping { nonce } => assert_eq!(nonce, 12345),
            _ => panic!("Wrong type"),
        }

        let pong_json = serde_json::to_string(&pong_expected).unwrap();
        let pong: NetMessage = serde_json::from_str(&pong_json).unwrap();
        match pong {
            NetMessage::Pong { nonce } => assert_eq!(nonce, 12345),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_new_block_message() {
        use crate::transaction::Transaction;

        let tx = Transaction::new_coinbase("miner".to_string(), 50, 0, 0);
        let block = Block::new(1, vec![tx], "prev_hash".to_string());

        let msg = NetMessage::NewBlock {
            block: block.clone(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            NetMessage::NewBlock { block: b } => {
                assert_eq!(b.index, 1);
                assert_eq!(b.previous_hash, "prev_hash");
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_get_blocks_message() {
        let msg = NetMessage::GetBlocks { from_height: 5 };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: NetMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            NetMessage::GetBlocks { from_height } => {
                assert_eq!(from_height, 5);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[tokio::test]
    async fn test_p2p_node_creation() {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let node = P2PNode::new("127.0.0.1:0".to_string(), vec![], blockchain);

        assert_eq!(node.peer_count().await, 0);
        assert!(node.peers().await.is_empty());
    }

    #[tokio::test]
    async fn test_two_nodes_connect() {
        let bc1 = Arc::new(Mutex::new(Blockchain::new()));
        let bc2 = Arc::new(Mutex::new(Blockchain::new()));

        // 启动节点1
        let node1 = Arc::new(P2PNode::new("127.0.0.1:18333".to_string(), vec![], bc1));

        let node1_clone = Arc::clone(&node1);
        let handle1 = tokio::spawn(async move {
            let _ = node1_clone.start().await;
        });

        // 等待节点1启动
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // 启动节点2并连接到节点1
        let node2 = Arc::new(P2PNode::new(
            "127.0.0.1:18334".to_string(),
            vec!["127.0.0.1:18333".to_string()],
            bc2,
        ));

        let node2_clone = Arc::clone(&node2);
        let handle2 = tokio::spawn(async move {
            let _ = node2_clone.start().await;
        });

        // 等待连接建立
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // 验证节点1有1个连接
        assert!(
            node1.peer_count().await >= 1,
            "Node1 should have at least 1 peer"
        );

        // 清理
        handle1.abort();
        handle2.abort();
    }
}
