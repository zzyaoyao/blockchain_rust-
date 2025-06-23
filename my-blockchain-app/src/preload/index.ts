import { contextBridge, ipcRenderer } from 'electron'

const electronAPI = {
  createWallet: () => ipcRenderer.invoke('create-wallet'),
  createBlockchain: (address: string) => ipcRenderer.invoke('create-blockchain', address),
  getInfo: () => ipcRenderer.invoke('get-info'),
  sendTx: (args: { from: string; to: string; amount: number }) => ipcRenderer.invoke('send-tx', args),
  startNode: (port: number, minerAddress?: string) => ipcRenderer.invoke('start-node', { port, minerAddress }),
};

contextBridge.exposeInMainWorld('electron', electronAPI);

// 定义暴露给渲染进程的、类型安全的 API
export const api = {
  createWallet: (): Promise<{ success: boolean; data?: string; error?: string }> =>
    ipcRenderer.invoke('create-wallet'),

  getBalance: (address: string): Promise<{ success: boolean; data?: number; error?: string }> =>
    ipcRenderer.invoke('get-balance', address),

  getInfo: (): Promise<{ success: boolean; data?: string; error?: string }> =>
    ipcRenderer.invoke('get-info'),
    
  sendTx: (txInfo: { from: string; to: string; amount: number }): Promise<{ success: boolean; data?: string; error?: string }> => 
    ipcRenderer.invoke('send-tx', txInfo),

  createBlockchain: (address: string): Promise<{ success: boolean; data?: string; error?: string }> =>
    ipcRenderer.invoke('create-blockchain', address),
}

// 安全地将 API 挂载到 window 对象上
contextBridge.exposeInMainWorld('electronAPI', api) 