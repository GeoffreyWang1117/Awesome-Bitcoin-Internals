use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bitcoin_simulation::{
    blockchain::Blockchain,
    wallet::Wallet,
    persistence::StorageManager,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

// API响应结构
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// 区块链状态信息
#[derive(Serialize)]
struct BlockchainInfo {
    height: usize,
    difficulty: usize,
    pending_transactions: usize,
    mining_reward: u64,
}

// 余额信息
#[derive(Serialize)]
struct BalanceInfo {
    address: String,
    balance: u64,
}

// 交易请求
#[derive(Deserialize)]
struct TransferRequest {
    from_address: String,
    to_address: String,
    amount: u64,
    fee: u64,
}

// 挖矿请求
#[derive(Deserialize)]
struct MineRequest {
    miner_address: String,
}

// 创建钱包响应
#[derive(Serialize)]
struct WalletInfo {
    address: String,
    public_key: String,
}

// 应用状态
#[derive(Clone)]
struct AppState {
    blockchain: Arc<Mutex<Blockchain>>,
    storage: Arc<StorageManager>,
}

#[tokio::main]
async fn main() {
    // 初始化区块链
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let storage = Arc::new(StorageManager::new("./data".to_string()));

    let app_state = AppState {
        blockchain,
        storage,
    };

    // 配置CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 配置路由
    let app = Router::new()
        .route("/", get(root))
        .route("/api/blockchain/info", get(get_blockchain_info))
        .route("/api/blockchain/chain", get(get_chain))
        .route("/api/wallet/create", post(create_wallet))
        .route("/api/wallet/balance/:address", get(get_balance))
        .route("/api/transaction/create", post(create_transaction))
        .route("/api/mine", post(mine_block))
        .route("/api/blockchain/validate", get(validate_chain))
        .layer(cors)
        .with_state(app_state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("========================================");
    println!("   SimpleBTC API Server");
    println!("========================================");
    println!("🚀 服务器运行在: http://127.0.0.1:3000");
    println!();
    println!("API 端点:");
    println!("  GET  /api/blockchain/info           - 获取区块链信息");
    println!("  GET  /api/blockchain/chain          - 获取完整区块链");
    println!("  POST /api/wallet/create             - 创建新钱包");
    println!("  GET  /api/wallet/balance/:address   - 查询余额");
    println!("  POST /api/transaction/create        - 创建交易");
    println!("  POST /api/mine                      - 挖矿");
    println!("  GET  /api/blockchain/validate       - 验证区块链");
    println!("========================================\n");

    axum::serve(listener, app).await.unwrap();
}

// 根路径
async fn root() -> &'static str {
    "SimpleBTC API Server v1.0 - Bitcoin Banking System"
}

// 获取区块链信息
async fn get_blockchain_info(
    State(state): State<AppState>,
) -> Json<ApiResponse<BlockchainInfo>> {
    let blockchain = state.blockchain.lock().unwrap();

    let info = BlockchainInfo {
        height: blockchain.chain.len(),
        difficulty: blockchain.difficulty,
        pending_transactions: blockchain.pending_transactions.len(),
        mining_reward: blockchain.mining_reward,
    };

    Json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

// 获取完整区块链
async fn get_chain(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let blockchain = state.blockchain.lock().unwrap();

    let chain_json = serde_json::to_value(&blockchain.chain).unwrap();

    Json(ApiResponse {
        success: true,
        data: Some(chain_json),
        error: None,
    })
}

// 创建新钱包
async fn create_wallet() -> Json<ApiResponse<WalletInfo>> {
    let wallet = Wallet::new();

    let info = WalletInfo {
        address: wallet.address.clone(),
        public_key: wallet.public_key.clone(),
    };

    Json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

// 查询余额
async fn get_balance(
    State(state): State<AppState>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> Json<ApiResponse<BalanceInfo>> {
    let blockchain = state.blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&address);

    let info = BalanceInfo { address, balance };

    Json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

// 创建交易
async fn create_transaction(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let mut blockchain = state.blockchain.lock().unwrap();

    // 创建临时钱包（实际应该从存储中加载）
    let from_wallet = Wallet::from_address(req.from_address.clone());

    match blockchain.create_transaction(
        &from_wallet,
        req.to_address.clone(),
        req.amount,
        req.fee,
    ) {
        Ok(tx) => {
            let tx_id = tx.id.clone();
            match blockchain.add_transaction(tx) {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        data: Some(format!("交易已创建: {}", tx_id)),
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

// 挖矿
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
                    data: Some(format!("区块已挖出，当前高度: {}", height)),
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

// 验证区块链
async fn validate_chain(
    State(state): State<AppState>,
) -> Json<ApiResponse<String>> {
    let blockchain = state.blockchain.lock().unwrap();
    let is_valid = blockchain.is_valid();

    Json(ApiResponse {
        success: is_valid,
        data: Some(if is_valid {
            "区块链有效".to_string()
        } else {
            "区块链无效".to_string()
        }),
        error: None,
    })
}
