#!/bin/bash
# Git pre-commit hook
# 在提交前自动运行代码检查

set -e

echo "🔍 运行pre-commit检查..."

# 1. 代码格式检查
echo "📝 检查代码格式..."
cargo fmt --all -- --check || {
    echo "❌ 代码格式不符合规范，请运行: cargo fmt"
    exit 1
}

# 2. Clippy检查
echo "🔧 运行Clippy..."
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "❌ Clippy检查失败，请修复警告"
    exit 1
}

# 3. 运行测试
echo "🧪 运行测试..."
cargo test --lib || {
    echo "❌ 测试失败"
    exit 1
}

# 4. 构建检查
echo "🏗️  检查构建..."
cargo check --all-features || {
    echo "❌ 构建检查失败"
    exit 1
}

echo "✅ 所有检查通过！"
exit 0
