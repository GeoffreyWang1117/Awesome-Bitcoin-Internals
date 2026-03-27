//! SimpleBTC 全节点
//!
//! 集成P2P网络 + HTTP API的完整节点。
//!
//! 用法:
//!   btc-node --port 8333                          # 启动独立节点
//!   btc-node --port 8334 --seed 127.0.0.1:8333    # 连接到种子节点
//!   btc-node --port 8335 --seed 127.0.0.1:8333 --seed 127.0.0.1:8334
//!
//! 每个节点同时运行:
//! - P2P服务 (TCP端口: --port)
//! - HTTP API (端口: --port + 1000, 即 8333 → 9333)

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bitcoin_simulation::{
    blockchain::Blockchain,
    network::{NetworkStats, P2PNode},
    wallet::Wallet,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

// Embedded Web UI
const INDEX_HTML: &str = include_str!("../../static/index.html");

// ============ API types ============

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct NodeInfo {
    height: usize,
    difficulty: usize,
    pending_transactions: usize,
    mining_reward: u64,
    genesis_address: String,
    // P2P info
    p2p_addr: String,
    peer_count: usize,
    peers: Vec<String>,
}

#[derive(Serialize)]
struct BalanceInfo {
    address: String,
    balance: u64,
}

#[derive(Serialize)]
struct WalletInfo {
    address: String,
    public_key: String,
}

#[derive(Deserialize)]
struct TransferRequest {
    from_address: String,
    to_address: String,
    amount: u64,
    fee: u64,
}

#[derive(Deserialize)]
struct MineRequest {
    miner_address: String,
}

// ============ App state ============

#[derive(Clone)]
struct AppState {
    blockchain: Arc<Mutex<Blockchain>>,
    wallets: Arc<Mutex<HashMap<String, Wallet>>>,
    p2p: Arc<P2PNode>,
}

// ============ CLI args ============

struct Args {
    p2p_port: u16,
    seeds: Vec<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 8333u16;
    let mut seeds = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(8333);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--seed" | "-s" => {
                if i + 1 < args.len() {
                    seeds.push(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("SimpleBTC Full Node");
                println!();
                println!("Usage: btc-node [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --port <PORT>    P2P listen port (default: 8333)");
                println!("  -s, --seed <ADDR>    Seed node address (can be repeated)");
                println!("  -h, --help           Show this help");
                println!();
                println!("Examples:");
                println!("  btc-node --port 8333");
                println!("  btc-node --port 8334 --seed 127.0.0.1:8333");
                std::process::exit(0);
            }
            _ => {
                i += 1;
            }
        }
    }

    Args {
        p2p_port: port,
        seeds,
    }
}

// ============ Main ============

#[tokio::main]
async fn main() {
    let args = parse_args();
    let http_port = args.p2p_port + 1000;
    let p2p_addr = format!("0.0.0.0:{}", args.p2p_port);
    let http_addr = format!("0.0.0.0:{}", http_port);

    // 创建共享区块链
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // 创建P2P节点
    let p2p = Arc::new(P2PNode::new(
        p2p_addr.clone(),
        args.seeds.clone(),
        Arc::clone(&blockchain),
    ));

    // 初始化钱包存储
    let mut wallets = HashMap::new();
    let genesis = Wallet::genesis();
    wallets.insert(genesis.address.clone(), genesis);

    let app_state = AppState {
        blockchain,
        wallets: Arc::new(Mutex::new(wallets)),
        p2p: Arc::clone(&p2p),
    };

    // 启动P2P网络
    let p2p_clone = Arc::clone(&p2p);
    tokio::spawn(async move {
        if let Err(e) = p2p_clone.start().await {
            eprintln!("P2P error: {}", e);
        }
    });

    // 启动HTTP API
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/api/blockchain/info", get(get_node_info))
        .route("/api/blockchain/chain", get(get_chain))
        .route("/api/blockchain/validate", get(validate_chain))
        .route("/api/wallet/create", post(create_wallet))
        .route("/api/wallet/balance/:address", get(get_balance))
        .route("/api/transaction/create", post(create_transaction))
        .route("/api/mine", post(mine_block))
        .route("/api/network/stats", get(get_network_stats))
        .route("/api/network/peers", get(get_peers))
        .layer(cors)
        .with_state(app_state);

    let genesis_addr = Wallet::genesis().address;
    println!();
    println!("  SimpleBTC Full Node v1.1");
    println!("  ========================");
    println!();
    println!("  P2P:      tcp://{}", p2p_addr);
    println!("  HTTP API: http://localhost:{}", http_port);
    println!("  Web UI:   http://localhost:{}", http_port);
    println!();
    println!("  Genesis:  {} (pre-funded)", genesis_addr);
    if !args.seeds.is_empty() {
        println!("  Seeds:    {:?}", args.seeds);
    }
    println!();
    println!("  Endpoints:");
    println!("    GET  /                              Web UI");
    println!("    GET  /api/blockchain/info            Node info + P2P status");
    println!("    GET  /api/blockchain/chain           Full chain");
    println!("    GET  /api/blockchain/validate        Validate chain");
    println!("    POST /api/wallet/create              Create wallet");
    println!("    GET  /api/wallet/balance/:address    Query balance");
    println!("    POST /api/transaction/create         Create & broadcast tx");
    println!("    POST /api/mine                       Mine & broadcast block");
    println!("    GET  /api/network/stats              P2P network stats");
    println!("    GET  /api/network/peers              Connected peers");
    println!();

    let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ============ Handlers ============

async fn serve_ui() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        INDEX_HTML,
    )
}

async fn get_node_info(State(state): State<AppState>) -> Json<ApiResponse<NodeInfo>> {
    let blockchain = state.blockchain.lock().await;
    let stats = state.p2p.get_stats().await;

    Json(ApiResponse {
        success: true,
        data: Some(NodeInfo {
            height: blockchain.chain.len(),
            difficulty: blockchain.difficulty,
            pending_transactions: blockchain.mempool.len(),
            mining_reward: blockchain.mining_reward,
            genesis_address: Wallet::genesis().address,
            p2p_addr: stats.listen_addr,
            peer_count: stats.peer_count,
            peers: stats.peers,
        }),
        error: None,
    })
}

async fn get_chain(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let blockchain = state.blockchain.lock().await;
    let chain_json = serde_json::to_value(&blockchain.chain).unwrap();

    Json(ApiResponse {
        success: true,
        data: Some(chain_json),
        error: None,
    })
}

async fn validate_chain(State(state): State<AppState>) -> Json<ApiResponse<String>> {
    let blockchain = state.blockchain.lock().await;
    let is_valid = blockchain.is_valid();

    Json(ApiResponse {
        success: is_valid,
        data: Some(if is_valid {
            "Blockchain is valid".to_string()
        } else {
            "Blockchain is invalid".to_string()
        }),
        error: None,
    })
}

async fn create_wallet(State(state): State<AppState>) -> Json<ApiResponse<WalletInfo>> {
    let wallet = Wallet::new();
    let info = WalletInfo {
        address: wallet.address.clone(),
        public_key: wallet.public_key.clone(),
    };

    state
        .wallets
        .lock()
        .await
        .insert(wallet.address.clone(), wallet);

    Json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

async fn get_balance(
    State(state): State<AppState>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> Json<ApiResponse<BalanceInfo>> {
    let blockchain = state.blockchain.lock().await;
    let balance = blockchain.get_balance(&address);

    Json(ApiResponse {
        success: true,
        data: Some(BalanceInfo { address, balance }),
        error: None,
    })
}

async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let wallets = state.wallets.lock().await;
    let from_wallet = match wallets.get(&req.from_address) {
        Some(w) => w.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Wallet not found: {}", req.from_address)),
                }),
            );
        }
    };
    drop(wallets);

    let mut blockchain = state.blockchain.lock().await;
    match blockchain.create_transaction(&from_wallet, req.to_address, req.amount, req.fee) {
        Ok(tx) => {
            let tx_id = tx.id.clone();

            // 广播交易到P2P网络
            state.p2p.broadcast_transaction(&tx).await;

            match blockchain.add_transaction(tx) {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        data: Some(format!("Transaction created & broadcast: {}", tx_id)),
                        error: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        error: Some(e),
                    }),
                ),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e),
            }),
        ),
    }
}

async fn mine_block(
    State(state): State<AppState>,
    Json(req): Json<MineRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let mut blockchain = state.blockchain.lock().await;

    match blockchain.mine_pending_transactions(req.miner_address) {
        Ok(_) => {
            let height = blockchain.chain.len();
            let new_block = blockchain.chain.last().unwrap();

            // 广播新区块到P2P网络
            state.p2p.broadcast_block(new_block).await;

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    data: Some(format!("Block mined & broadcast! Height: {}", height)),
                    error: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e),
            }),
        ),
    }
}

async fn get_network_stats(State(state): State<AppState>) -> Json<ApiResponse<NetworkStats>> {
    let stats = state.p2p.get_stats().await;

    Json(ApiResponse {
        success: true,
        data: Some(stats),
        error: None,
    })
}

async fn get_peers(State(state): State<AppState>) -> Json<ApiResponse<Vec<String>>> {
    let peers = state.p2p.peers().await;

    Json(ApiResponse {
        success: true,
        data: Some(peers),
        error: None,
    })
}
