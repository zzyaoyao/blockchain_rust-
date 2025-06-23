import { app, BrowserWindow, ipcMain } from 'electron'
import path from 'path'
import { execFile, execSync } from 'child_process'
import util from 'util'
import fs from 'fs'
import { spawn } from 'child_process'

// 将 execFile 转换为 Promise-based 函数
const execFilePromise = util.promisify(execFile);

// 确保应用程序资源目录存在
const appResourcesDir = path.join(app.getAppPath(), 'resources');
if (!fs.existsSync(appResourcesDir)) {
  fs.mkdirSync(appResourcesDir, { recursive: true });
}

// 定义本地可执行文件路径
const localRustBinaryPath = path.join(appResourcesDir, 'blockchain-demo.exe');

// 如果本地不存在可执行文件，尝试从原始位置复制
const originalRustBinaryPath = path.resolve(app.getAppPath(), '../target/release/blockchain-demo.exe');
if (!fs.existsSync(localRustBinaryPath) && fs.existsSync(originalRustBinaryPath)) {
  try {
    fs.copyFileSync(originalRustBinaryPath, localRustBinaryPath);
    console.log('Copied Rust binary to local resources directory');
  } catch (err) {
    console.error('Failed to copy Rust binary:', err);
  }
}

// 使用本地路径或打包后的资源路径
const rustBinaryPath = app.isPackaged
  ? path.join(process.resourcesPath, 'blockchain-demo.exe') 
  : localRustBinaryPath;

console.log('Rust binary path:', rustBinaryPath);
console.log('File exists:', fs.existsSync(rustBinaryPath) ? 'Yes' : 'No');

function createWindow() {
  const mainWindow = new BrowserWindow({
    width: 900,
    height: 670,
    show: false,
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, '../preload/index.js'),
      sandbox: false,
      nodeIntegration: true,
      contextIsolation: true,
    },
  });

  mainWindow.on('ready-to-show', () => {
    mainWindow.show();
  });

  // Vite dev server URL or index.html file
  if (process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
    // Open devtools
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
  }
}

// --- IPC Handlers ---

ipcMain.handle('get-info', async () => {
  try {
    console.log('Executing Rust binary for info:', rustBinaryPath);
    
    // 使用 execFile 执行命令并获取输出
    const { stdout } = await execFilePromise(rustBinaryPath, ['info']);
    
    // 只保留真正的区块链信息（过滤掉日志输出）
    const lines = stdout.split('\n');
    const filteredLines = lines.filter(line => 
      !line.includes('[blockchain_demo]') && 
      !line.includes('[sled::') &&
      !line.trim().startsWith('TRACE') && 
      !line.trim().startsWith('DEBUG') &&
      !line.trim().startsWith('INFO')
    );
    
    return { success: true, data: filteredLines.join('\n') };
  } catch (error: any) {
    console.error('Error executing Rust binary:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('create-wallet', async () => {
  try {
    console.log('Executing Rust binary for create-wallet:', rustBinaryPath);
    const { stdout } = await execFilePromise(rustBinaryPath, ['create-wallet']);
    // 从输出中提取地址
    const address = stdout.split(':').pop()?.trim();
    return { success: true, data: address };
  } catch (error: any) {
    console.error('Error executing Rust binary:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('create-blockchain', async (event, address) => {
  try {
    console.log('Executing Rust binary for create-blockchain:', rustBinaryPath);
    const { stdout } = await execFilePromise(rustBinaryPath, ['create-blockchain', address]);
    return { success: true, data: stdout };
  } catch (error: any) {
    console.error('Error executing Rust binary for create-blockchain:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('send-tx', async (event, { from, to, amount }) => {
    if (!from || !to || !amount) {
        return { success: false, error: "From, To, and Amount are required." };
    }
    try {
        console.log('Executing Rust binary for send-tx:', rustBinaryPath);
        const { stdout } = await execFilePromise(rustBinaryPath, [
            'send',
            '--from', from,
            '--to', to,
            '--amount', amount.toString()
        ]);
        // 过滤日志，只保留有用信息
        const lines = stdout.split('\n');
        const filteredLines = lines.filter(line => 
          !line.includes('[blockchain_demo]') && 
          !line.includes('[sled::') &&
          !line.trim().startsWith('TRACE') && 
          !line.trim().startsWith('DEBUG') &&
          !line.trim().startsWith('INFO')
        );
        return { success: true, data: filteredLines.join('\n') };
    } catch (error: any) {
        console.error('Error executing Rust binary:', error);
        // Rust程序的错误通常在stderr
        const errorMessage = error.stderr || error.message;
        return { success: false, error: errorMessage };
    }
});

ipcMain.handle('start-node', async (event, { port, minerAddress }) => {
  try {
    const args = ['start-node', port.toString()];
    if (minerAddress) {
      args.push('--miner_address', minerAddress);
    }
    const nodeProcess = spawn(rustBinaryPath, args, { stdio: 'ignore', detached: true });
    nodeProcess.unref(); // 让进程在主进程退出后依然运行
    return { success: true, data: '节点已启动' };
  } catch (error: any) {
    return { success: false, error: error.message };
  }
});

app.whenReady().then(() => {
  createWindow()
  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
}) 