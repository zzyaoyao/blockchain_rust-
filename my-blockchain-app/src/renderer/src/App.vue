<template>
  <div class="container">
    <h1>Rust Blockchain GUI</h1>

    <div class="card">
      <h2>Actions</h2>
      <div class="action-buttons">
        <button @click="createWallet">Create New Wallet</button>
        <button @click="createBlockchain" :disabled="!newWalletAddress">Create Blockchain</button>
      </div>
      <div v-if="newWalletAddress" class="result">
        New Wallet Address: <code>{{ newWalletAddress }}</code>
      </div>
      <div v-if="actionError" class="error">
        Error: {{ actionError }}
      </div>
      <div v-if="blockchainCreationResult" class="result">
        {{ blockchainCreationResult }}
      </div>
    </div>

    <div class="card">
      <h2>Start Node</h2>
      <input v-model="nodePort" type="number" placeholder="Port (如 6001)" />
      <input v-model="minerAddress" placeholder="Miner Address (可选)" />
      <button @click="handleStartNode">Start Node</button>
      <div v-if="nodeStatus" class="result">{{ nodeStatus }}</div>
    </div>

    <div class="card">
      <h2>Blockchain Info</h2>
      <button @click="fetchInfo">Refresh Info</button>
      <div v-if="infoError" class="error">
        Error fetching info: {{ infoError }}
      </div>
      <pre v-if="blockchainInfo">{{ blockchainInfo }}</pre>
    </div>
    
    <div class="card">
        <h2>Send Coins</h2>
        <input v-model="sendForm.from" placeholder="From Address" />
        <input v-model="sendForm.to" placeholder="To Address" />
        <input v-model.number="sendForm.amount" type="number" placeholder="Amount" />
        <button @click="handleSend">Send</button>
        <div v-if="sendStatus" class="result">{{ sendStatus }}</div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'

const newWalletAddress = ref('')
const blockchainInfo = ref('')
const infoError = ref('')
const actionError = ref('')
const blockchainCreationResult = ref('')
const sendStatus = ref('')

const nodePort = ref(6001)
const minerAddress = ref('')
const nodeStatus = ref('')

const sendForm = reactive({
    from: '',
    to: '',
    amount: 0
});

const createWallet = async () => {
  actionError.value = ''
  blockchainCreationResult.value = ''
  try {
    const result = await window.electron.createWallet()
    if (result.success) {
      newWalletAddress.value = result.data
    } else {
      actionError.value = result.error
    }
  } catch (error) {
    actionError.value = error.message
  }
}

const createBlockchain = async () => {
  if (!newWalletAddress.value) {
    actionError.value = 'Please create a wallet first.'
    return
  }
  actionError.value = ''
  blockchainCreationResult.value = ''
  try {
    const result = await window.electron.createBlockchain(newWalletAddress.value)
    if (result.success) {
      blockchainCreationResult.value = 'Blockchain created successfully!'
      fetchInfo() // Refresh info after creating blockchain
    } else {
      // Check for specific error message
      if (result.error && result.error.includes('Blockchain already exists')) {
        blockchainCreationResult.value = 'Blockchain already exists. No need to create again.'
      } else {
        actionError.value = result.error
      }
    }
  } catch (error) {
    actionError.value = error.message
  }
}

const fetchInfo = async () => {
  infoError.value = '';
  blockchainInfo.value = '';
  try {
    const result = await window.electron.getInfo();
    if (result.success) {
      blockchainInfo.value = result.data;
    } else {
      infoError.value = result.error;
    }
  } catch (error) {
    infoError.value = error.message;
  }
};

const handleSend = async () => {
    actionError.value = '';
    sendStatus.value = 'Sending...';
    try {
      const result = await window.electron.sendTx({
        from: sendForm.from,
        to: sendForm.to,
        amount: sendForm.amount
      });
      if (result.success) {
        sendStatus.value = `Transaction successful! ${result.data}`;
        fetchInfo(); // 使用 fetchInfo 替代 handleGetInfo
      } else {
        sendStatus.value = `Error: ${result.error}`;
      }
    } catch (error) {
      sendStatus.value = `Error: ${error.message}`;
    }
};

const handleStartNode = async () => {
  nodeStatus.value = '正在启动节点...';
  try {
    const result = await window.electron.startNode(Number(nodePort.value), minerAddress.value || undefined);
    if (result.success) {
      nodeStatus.value = result.data;
    } else {
      nodeStatus.value = `启动失败: ${result.error}`;
    }
  } catch (e: any) {
    nodeStatus.value = `启动失败: ${e.message}`;
  }
};

// 初始加载
fetchInfo();
</script>

<style scoped>
.container {
  padding: 2rem;
  font-family: sans-serif;
}
.card {
  background: #f4f4f4;
  padding: 1rem;
  margin-bottom: 1rem;
  border-radius: 8px;
}
button {
  margin-right: 10px;
  padding: 8px 15px;
  border: none;
  background-color: #42b883;
  color: white;
  border-radius: 4px;
  cursor: pointer;
}
button:hover {
  background-color: #35495e;
}
input {
    display: block;
    margin-bottom: 10px;
    padding: 8px;
    width: 300px;
}
.result {
  margin-top: 1rem;
  color: #35495e;
  font-weight: bold;
}
.result span {
  font-family: monospace;
  background: #e0e0e0;
  padding: 2px 4px;
  border-radius: 4px;
}
.error {
  margin-top: 1rem;
  color: red;
}
.info-box {
  background: #333;
  color: #eee;
  padding: 1rem;
  border-radius: 4px;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: monospace;
}
</style> 