#!/bin/bash
# 开发环境快速启动脚本

set -e

echo "🚀 SimpleBTC开发环境启动..."

# 检查Rust是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ 未安装Rust，请访问 https://rustup.rs/ 安装"
    exit 1
fi

# 检查依赖
echo "📦 检查依赖..."
cargo check || {
    echo "正在下载依赖..."
    cargo fetch
}

# 清理旧数据
if [ -d "./data" ]; then
    echo "🧹 清理旧数据..."
    rm -rf ./data
fi

# 创建数据目录
mkdir -p ./data/wallets

# 启动API服务器（后台）
echo "🌐 启动API服务器..."
cargo run --bin btc-server &
SERVER_PID=$!

# 等待服务器启动
echo "⏳ 等待服务器启动..."
sleep 3

# 检查服务器是否运行
if curl -s http://127.0.0.1:3000/ > /dev/null; then
    echo "✅ API服务器已启动: http://127.0.0.1:3000"
else
    echo "❌ API服务器启动失败"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

echo ""
echo "📖 可用的API端点:"
echo "  GET  /api/blockchain/info"
echo "  GET  /api/blockchain/chain"
echo "  POST /api/wallet/create"
echo "  GET  /api/wallet/balance/:address"
echo "  POST /api/transaction/create"
echo "  POST /api/mine"
echo "  GET  /api/blockchain/validate"
echo ""
echo "按 Ctrl+C 停止服务器"

# 等待用户中断
trap "kill $SERVER_PID 2>/dev/null; echo '👋 服务器已停止'; exit 0" INT TERM

wait $SERVER_PID
