use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bitcoin_simulation::{blockchain::Blockchain, persistence::StorageManager, wallet::Wallet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

// Embedded Web UI (single-binary deployment)
const INDEX_HTML: &str = include_str!("../../static/index.html");

// API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Blockchain info
#[derive(Serialize)]
struct BlockchainInfo {
    height: usize,
    difficulty: usize,
    pending_transactions: usize,
    mining_reward: u64,
    genesis_address: String,
}

// Balance info
#[derive(Serialize)]
struct BalanceInfo {
    address: String,
    balance: u64,
}

// Transfer request
#[derive(Deserialize)]
struct TransferRequest {
    from_address: String,
    to_address: String,
    amount: u64,
    fee: u64,
}

// Mine request
#[derive(Deserialize)]
struct MineRequest {
    miner_address: String,
}

// Wallet info
#[derive(Serialize)]
struct WalletInfo {
    address: String,
    public_key: String,
}

// App state
#[derive(Clone)]
struct AppState {
    blockchain: Arc<Mutex<Blockchain>>,
    /// 钱包存储：地址 → 钱包（含真实私钥）
    wallets: Arc<Mutex<HashMap<String, Wallet>>>,
    #[allow(dead_code)]
    storage: Arc<StorageManager>,
}

#[tokio::main]
async fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let storage = Arc::new(StorageManager::new("./data".to_string()));

    // 初始化钱包存储，预置创世钱包
    let mut wallets = HashMap::new();
    let genesis = Wallet::genesis();
    wallets.insert(genesis.address.clone(), genesis);

    let app_state = AppState {
        blockchain,
        wallets: Arc::new(Mutex::new(wallets)),
        storage,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Web UI
        .route("/", get(serve_ui))
        // API endpoints
        .route("/api/blockchain/info", get(get_blockchain_info))
        .route("/api/blockchain/chain", get(get_chain))
        .route("/api/blockchain/validate", get(validate_chain))
        .route("/api/wallet/create", post(create_wallet))
        .route("/api/wallet/balance/:address", get(get_balance))
        .route("/api/transaction/create", post(create_transaction))
        .route("/api/mine", post(mine_block))
        .layer(cors)
        .with_state(app_state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let genesis_addr = Wallet::genesis().address;
    println!();
    println!("  SimpleBTC Server v1.0");
    println!("  =====================");
    println!();
    println!("  Web UI:   http://localhost:3000");
    println!("  API:      http://localhost:3000/api/blockchain/info");
    println!();
    println!("  Genesis:  {} (pre-funded with 100 BTC)", genesis_addr);
    println!();
    println!("  Crypto:   secp256k1 ECDSA (real Bitcoin signatures)");
    println!();
    println!("  Endpoints:");
    println!("    GET  /                              Web UI");
    println!("    GET  /api/blockchain/info            Chain info");
    println!("    GET  /api/blockchain/chain           Full chain");
    println!("    GET  /api/blockchain/validate        Validate chain");
    println!("    POST /api/wallet/create              Create wallet");
    println!("    GET  /api/wallet/balance/:address    Query balance");
    println!("    POST /api/transaction/create         Create transaction");
    println!("    POST /api/mine                       Mine block");
    println!();

    axum::serve(listener, app).await.unwrap();
}

// Serve embedded Web UI
async fn serve_ui() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        INDEX_HTML,
    )
}

// Get blockchain info
async fn get_blockchain_info(State(state): State<AppState>) -> Json<ApiResponse<BlockchainInfo>> {
    let blockchain = state.blockchain.lock().unwrap();

    Json(ApiResponse {
        success: true,
        data: Some(BlockchainInfo {
            height: blockchain.chain.len(),
            difficulty: blockchain.difficulty,
            pending_transactions: blockchain.mempool.len(),
            mining_reward: blockchain.mining_reward,
            genesis_address: Wallet::genesis().address,
        }),
        error: None,
    })
}

// Get full chain
async fn get_chain(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let blockchain = state.blockchain.lock().unwrap();
    let chain_json = serde_json::to_value(&blockchain.chain).unwrap();

    Json(ApiResponse {
        success: true,
        data: Some(chain_json),
        error: None,
    })
}

// Create wallet (stored server-side for signing)
async fn create_wallet(State(state): State<AppState>) -> Json<ApiResponse<WalletInfo>> {
    let wallet = Wallet::new();
    let info = WalletInfo {
        address: wallet.address.clone(),
        public_key: wallet.public_key.clone(),
    };

    // 存储钱包以便后续签名交易
    state
        .wallets
        .lock()
        .unwrap()
        .insert(wallet.address.clone(), wallet);

    Json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

// Query balance
async fn get_balance(
    State(state): State<AppState>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> Json<ApiResponse<BalanceInfo>> {
    let blockchain = state.blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&address);

    Json(ApiResponse {
        success: true,
        data: Some(BalanceInfo { address, balance }),
        error: None,
    })
}

// Create transaction (uses stored wallet for real ECDSA signing)
async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    // 从钱包存储中查找发送方钱包
    let wallets = state.wallets.lock().unwrap();
    let from_wallet = match wallets.get(&req.from_address) {
        Some(w) => w.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!(
                        "钱包未找到: {}。请先通过 /api/wallet/create 创建钱包。",
                        req.from_address
                    )),
                }),
            );
        }
    };
    drop(wallets); // 释放锁

    let mut blockchain = state.blockchain.lock().unwrap();
    match blockchain.create_transaction(&from_wallet, req.to_address.clone(), req.amount, req.fee) {
        Ok(tx) => {
            let tx_id = tx.id.clone();
            match blockchain.add_transaction(tx) {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        data: Some(format!("Transaction created: {}", tx_id)),
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

// Mine block
async fn mine_block(
    State(state): State<AppState>,
    Json(req): Json<MineRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let mut blockchain = state.blockchain.lock().unwrap();

    match blockchain.mine_pending_transactions(req.miner_address) {
        Ok(_) => {
            let height = blockchain.chain.len();
            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    data: Some(format!("Block mined! Height: {}", height)),
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

// Validate chain
async fn validate_chain(State(state): State<AppState>) -> Json<ApiResponse<String>> {
    let blockchain = state.blockchain.lock().unwrap();
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
