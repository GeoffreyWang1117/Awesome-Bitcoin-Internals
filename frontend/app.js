const API_BASE = 'http://127.0.0.1:3000/api';

// 存储钱包列表
let wallets = [];
let demoWallets = {};

// 页面加载时初始化
window.addEventListener('DOMContentLoaded', async () => {
    await checkAPIConnection();
    await loadBlockchainInfo();
    setInterval(loadBlockchainInfo, 5000); // 每5秒更新一次
});

// 检查API连接
async function checkAPIConnection() {
    try {
        const response = await fetch('http://127.0.0.1:3000/');
        if (response.ok) {
            document.getElementById('apiStatus').innerHTML = '🟢 API已连接';
            setStatus('成功连接到SimpleBTC API服务器', 'success');
        }
    } catch (error) {
        document.getElementById('apiStatus').innerHTML = '🔴 API未连接';
        setStatus('无法连接到API服务器，请确保服务器正在运行', 'error');
    }
}

// 加载区块链信息
async function loadBlockchainInfo() {
    try {
        const response = await fetch(`${API_BASE}/blockchain/info`);
        const data = await response.json();

        if (data.success) {
            document.getElementById('blockHeight').textContent = data.data.height;
            document.getElementById('difficulty').textContent = data.data.difficulty;
            document.getElementById('pendingTxs').textContent = data.data.pending_transactions;
        }
    } catch (error) {
        console.error('加载区块链信息失败:', error);
    }
}

// 创建新钱包
async function createWallet() {
    try {
        const response = await fetch(`${API_BASE}/wallet/create`, {
            method: 'POST'
        });
        const data = await response.json();

        if (data.success) {
            const wallet = data.data;
            wallets.push(wallet);
            renderWallets();
            setStatus(`钱包创建成功: ${wallet.address}`, 'success');

            // 自动填充到发送地址（如果没有）
            const fromAddress = document.getElementById('fromAddress');
            if (!fromAddress.value) {
                fromAddress.value = wallet.address;
            }

            // 自动填充到矿工地址（如果没有）
            const minerAddress = document.getElementById('minerAddress');
            if (!minerAddress.value) {
                minerAddress.value = wallet.address;
            }
        }
    } catch (error) {
        setStatus('创建钱包失败: ' + error.message, 'error');
    }
}

// 渲染钱包列表
async function renderWallets() {
    const walletList = document.getElementById('walletList');
    walletList.innerHTML = '';

    for (const wallet of wallets) {
        // 获取余额
        const balance = await getBalance(wallet.address);

        const walletDiv = document.createElement('div');
        walletDiv.className = 'wallet-item';
        walletDiv.innerHTML = `
            <div class="address" title="${wallet.address}">${wallet.address.substring(0, 20)}...</div>
            <div class="balance">${balance} BTC</div>
        `;
        walletDiv.onclick = () => {
            document.getElementById('fromAddress').value = wallet.address;
        };
        walletList.appendChild(walletDiv);
    }
}

// 获取余额
async function getBalance(address) {
    try {
        const response = await fetch(`${API_BASE}/wallet/balance/${address}`);
        const data = await response.json();
        return data.success ? data.data.balance : 0;
    } catch (error) {
        return 0;
    }
}

// 创建交易
async function createTransaction(event) {
    event.preventDefault();

    const fromAddress = document.getElementById('fromAddress').value;
    const toAddress = document.getElementById('toAddress').value;
    const amount = parseInt(document.getElementById('amount').value);
    const fee = parseInt(document.getElementById('fee').value);

    try {
        const response = await fetch(`${API_BASE}/transaction/create`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                from_address: fromAddress,
                to_address: toAddress,
                amount: amount,
                fee: fee
            })
        });

        const data = await response.json();

        if (data.success) {
            setStatus(data.data, 'success');
            document.getElementById('transferForm').reset();
            await loadBlockchainInfo();
        } else {
            setStatus('交易失败: ' + data.error, 'error');
        }
    } catch (error) {
        setStatus('创建交易失败: ' + error.message, 'error');
    }
}

// 挖矿
async function mineBlock() {
    const minerAddress = document.getElementById('minerAddress').value;

    if (!minerAddress) {
        setStatus('请输入矿工地址', 'error');
        return;
    }

    const statusDiv = document.getElementById('miningStatus');
    statusDiv.textContent = '⛏️ 挖矿中...';
    statusDiv.className = 'status-message info';

    try {
        const response = await fetch(`${API_BASE}/mine`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                miner_address: minerAddress
            })
        });

        const data = await response.json();

        if (data.success) {
            statusDiv.textContent = '✅ ' + data.data;
            statusDiv.className = 'status-message success';
            setStatus(data.data, 'success');
            await loadBlockchainInfo();
            await loadBlockchain();
            await renderWallets();
        } else {
            statusDiv.textContent = '❌ ' + data.error;
            statusDiv.className = 'status-message error';
            setStatus('挖矿失败: ' + data.error, 'error');
        }
    } catch (error) {
        statusDiv.textContent = '❌ 挖矿失败';
        statusDiv.className = 'status-message error';
        setStatus('挖矿失败: ' + error.message, 'error');
    }
}

// 加载区块链
async function loadBlockchain() {
    try {
        const response = await fetch(`${API_BASE}/blockchain/chain`);
        const data = await response.json();

        if (data.success) {
            renderBlockchain(data.data);
        }
    } catch (error) {
        setStatus('加载区块链失败: ' + error.message, 'error');
    }
}

// 渲染区块链
function renderBlockchain(chain) {
    const container = document.getElementById('blockchainContainer');
    container.innerHTML = '';

    // 倒序显示（最新的在前）
    for (let i = chain.length - 1; i >= 0; i--) {
        const block = chain[i];
        const blockDiv = document.createElement('div');
        blockDiv.className = 'block';

        const transactions = block.transactions.map((tx, idx) => `
            <div class="transaction">
                <div class="tx-id">TX #${idx}: ${tx.id.substring(0, 30)}...</div>
                <div class="tx-details">
                    ${tx.fee > 0 ? `手续费: ${tx.fee} satoshi` : 'Coinbase交易'}
                </div>
            </div>
        `).join('');

        blockDiv.innerHTML = `
            <div class="block-header">
                <div class="block-index">区块 #${block.index}</div>
            </div>
            <div class="block-hash">哈希: ${block.hash}</div>
            <div class="block-hash">前一哈希: ${block.previous_hash}</div>
            <div class="block-info">
                时间戳: ${new Date(block.timestamp * 1000).toLocaleString()} |
                Nonce: ${block.nonce} |
                交易数: ${block.transactions.length}
            </div>
            <div class="transactions">
                <strong>交易:</strong>
                ${transactions}
            </div>
        `;

        container.appendChild(blockDiv);
    }
}

// 验证区块链
async function validateBlockchain() {
    try {
        const response = await fetch(`${API_BASE}/blockchain/validate`);
        const data = await response.json();

        if (data.success) {
            setStatus('✅ ' + data.data, 'success');
        } else {
            setStatus('❌ ' + data.data, 'error');
        }
    } catch (error) {
        setStatus('验证失败: ' + error.message, 'error');
    }
}

// 设置状态消息
function setStatus(message, type = 'info') {
    const statusEl = document.getElementById('statusMessage');
    statusEl.textContent = message;
    statusEl.className = `status-message ${type}`;
    console.log(`[${type.toUpperCase()}] ${message}`);
}

// 延迟函数
function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

// 运行演示
async function runDemo() {
    setStatus('🎬 开始运行演示...', 'info');

    try {
        // 步骤1: 创建三个钱包
        setStatus('步骤 1/6: 创建演示钱包...', 'info');
        const walletAlice = await createDemoWallet();
        await delay(500);
        const walletBob = await createDemoWallet();
        await delay(500);
        const walletCharlie = await createDemoWallet();
        await delay(1000);

        demoWallets = { alice: walletAlice, bob: walletBob, charlie: walletCharlie };

        // 步骤2: 获取创世地址并给Alice发放初始余额
        setStatus('步骤 2/6: 为Alice发放初始余额...', 'info');
        const infoResp = await fetch(`${API_BASE}/blockchain/info`);
        const infoData = await infoResp.json();
        const genesisAddr = infoData.data.genesis_address;
        await createDemoTransaction(genesisAddr, walletAlice.address, 100, 0);
        await delay(1000);

        // 步骤3: 挖矿
        setStatus('步骤 3/6: 挖矿打包交易...', 'info');
        await mineBlockDemo(walletAlice.address);
        await delay(2000);

        // 步骤4: Alice转账给Bob
        setStatus('步骤 4/6: Alice向Bob转账30 BTC...', 'info');
        await createDemoTransaction(walletAlice.address, walletBob.address, 30, 2);
        await delay(1000);

        // 步骤5: Bob转账给Charlie（更高手续费）
        setStatus('步骤 5/6: Bob向Charlie转账20 BTC（高手续费优先）...', 'info');
        await createDemoTransaction(walletBob.address, walletCharlie.address, 20, 5);
        await delay(1000);

        // 步骤6: 最后一次挖矿
        setStatus('步骤 6/6: 挖矿确认交易...', 'info');
        await mineBlockDemo(walletAlice.address);
        await delay(2000);

        // 完成
        setStatus('✅ 演示完成！查看右侧区块链和钱包余额', 'success');
        await loadBlockchain();
        await renderWallets();

    } catch (error) {
        setStatus('演示失败: ' + error.message, 'error');
    }
}

// 创建演示钱包
async function createDemoWallet() {
    const response = await fetch(`${API_BASE}/wallet/create`, { method: 'POST' });
    const data = await response.json();
    if (data.success) {
        wallets.push(data.data);
        await renderWallets();
        return data.data;
    }
    throw new Error('创建钱包失败');
}

// 创建演示交易
async function createDemoTransaction(from, to, amount, fee) {
    const response = await fetch(`${API_BASE}/transaction/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            from_address: from,
            to_address: to,
            amount: amount,
            fee: fee
        })
    });
    const data = await response.json();
    if (!data.success) {
        // 如果失败是正常的（比如余额不足），不抛出错误
        console.log('交易失败（预期）:', data.error);
    }
}

// 演示挖矿
async function mineBlockDemo(address) {
    const response = await fetch(`${API_BASE}/mine`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ miner_address: address })
    });
    const data = await response.json();
    if (data.success) {
        await loadBlockchainInfo();
    }
}

// 重置演示
function resetDemo() {
    if (confirm('确定要重置演示吗？这将刷新页面。')) {
        location.reload();
    }
}
