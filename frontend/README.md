# SimpleBTC Frontend - Electron GUI

SimpleBTC的图形用户界面，使用Electron构建。

## 安装

```bash
cd frontend
npm install
```

## 运行

### 1. 启动后端API服务器

在项目根目录：

```bash
cargo run --bin btc-server --release
```

服务器将在 `http://127.0.0.1:3000` 运行

### 2. 启动Electron GUI

在frontend目录：

```bash
npm start
```

## 功能

### 钱包管理
- 创建新钱包
- 查看钱包余额
- 点击钱包快速填充发送地址

### 转账功能
- 发起转账交易
- 设置交易手续费
- 手续费越高，交易越优先被打包

### 挖矿
- 输入矿工地址
- 点击挖矿按钮打包待处理交易
- 获得挖矿奖励和所有交易手续费

### 区块链浏览
- 实时查看区块链
- 查看每个区块的详细信息
- 查看交易详情
- 验证区块链完整性

### 演示模式
- 点击"运行演示"按钮自动演示完整流程
- 自动创建钱包、转账、挖矿
- 适合视频录制和演示

## 开发模式

```bash
npm run dev
```

开发模式会自动打开DevTools。
