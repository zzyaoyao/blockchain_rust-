use crate::transaction::Transaction;
use anyhow::{anyhow, Result};
use bincode::serialize;
use log::{error, info, debug};
use merkle_cbt::merkle_tree::Merge;
use merkle_cbt::merkle_tree::CBMT;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TARGET_BITS: u32 = 16; // 保持较低难度

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub prev_block_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub height: i32,
    pub difficulty: u32,
}

#[allow(dead_code)]
impl Block {
    pub fn new_block(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
        difficulty: u32,
    ) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash: prev_block_hash.clone(),
            hash: String::new(),
            nonce: 0,
            height,
            difficulty,
        };

        block.run_proof_of_work()?;

        info!(
            "Created new block: height={}, prev_hash={}, hash={}, nonce={}, txs={}",
            height,
            prev_block_hash,
            block.hash,
            block.nonce,
            block.transactions.len()
        );
        
        Ok(block)
    }

    fn run_proof_of_work(&mut self) -> Result<()> {
        info!(
            "Mining block: height={}, difficulty={}",
            self.height, self.difficulty
        );
        
        let start_time = std::time::Instant::now();
        let mut attempts = 0;
        let mut last_log_time = start_time;
        
        loop {
            let data = match self.prepare_hash_data() {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to prepare hash data: {}", e);
                    return Err(e);
                }
            };
            
            let hash_bytes = Sha256::digest(&data);
            let hash_hex = hex::encode(hash_bytes);
            
            // 添加进度日志
            let now = std::time::Instant::now();
            if now.duration_since(last_log_time).as_secs() >= 5 {
                info!(
                    "Mining progress: attempts={}, nonce={}, current_hash={}",
                    attempts, self.nonce, hash_hex
                );
                last_log_time = now;
            }
            
            if self.is_valid_proof(&hash_hex) {
                self.hash = hash_hex;
                let elapsed = start_time.elapsed();
                info!(
                    "Block mined: hash={}, nonce={}, attempts={}, time={:.2}s",
                    self.hash,
                    self.nonce,
                    attempts,
                    elapsed.as_secs_f32()
                );
                return Ok(());
            }
            
            // 处理 nonce 溢出
            if self.nonce == u64::MAX {
                error!("Nonce overflow! Resetting to 0");
                self.nonce = 0;
            } else {
                self.nonce += 1;
            }
            
            attempts += 1;
            
            // 安全措施：防止无限循环
            if attempts % 10_000_000 == 0 {
                debug!("Safety check: {} attempts made", attempts);
            }
        }
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let header_data = (
            &self.prev_block_hash,
            self.hash_transactions()?,
            self.timestamp,
            self.difficulty,
            self.nonce,
        );
        
        serialize(&header_data).map_err(|e| anyhow!("Serialize error: {}", e))
    }

    fn hash_transactions(&self) -> Result<Vec<u8>> {
        if self.transactions.is_empty() {
            return Ok(vec![]);
        }
        
        let mut tx_hashes = Vec::new();
        for tx in &self.transactions {
            tx_hashes.push(tx.id.as_bytes().to_vec());
        }
        
        struct MergeSha256;
        impl Merge for MergeSha256 {
            type Item = Vec<u8>;
            fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
                let mut hasher = Sha256::new();
                hasher.update(left);
                hasher.update(right);
                hasher.finalize().to_vec()
            }
        }
        
        let tree = CBMT::<Vec<u8>, MergeSha256>::build_merkle_tree(&tx_hashes);
        Ok(tree.root())
    }

    fn is_valid_proof(&self, hash: &str) -> bool {
        let hash_bytes = match hex::decode(hash) {
            Ok(bytes) => bytes,
            Err(_) => {
                error!("Invalid hex hash: {}", hash);
                return false;
            }
        };
        
        let target = self.calculate_target();
        let is_valid = hash_bytes.as_slice() < target.as_slice();
        
        // 添加调试输出
        if self.nonce % 1_000_000 == 0 {
            info!(
                "Validation check: nonce={}, hash={}, target={}, valid={}",
                self.nonce,
                hash,
                hex::encode(&target),
                is_valid
            );
        }
        
        is_valid
    }
    
    fn calculate_target(&self) -> Vec<u8> {
        let mut target = vec![0xFFu8; 32]; // 初始化为全FF
        let zero_bytes = (self.difficulty / 8) as usize;
        let zero_bits = self.difficulty % 8;
        
        // 设置前导零字节
        for i in 0..zero_bytes.min(32) {
            target[i] = 0;
        }
        
        // 设置部分零的字节
        if zero_bits > 0 && zero_bytes < 32 {
            // 清除高位 zero_bits 位
            target[zero_bytes] &= 0xFFu8 >> zero_bits;
        }
        
        target
    }
    
    pub fn validate(&self, prev_block: &Block) -> Result<bool> {
        let calculated_hash = self.calculate_hash()?;
        if calculated_hash != self.hash {
            return Err(anyhow!("Block hash mismatch"));
        }
        
        if !self.is_valid_proof(&self.hash) {
            return Err(anyhow!("Invalid proof-of-work"));
        }
        
        if self.prev_block_hash != prev_block.hash {
            return Err(anyhow!("Previous block hash mismatch"));
        }
        
        if self.height != prev_block.height + 1 {
            return Err(anyhow!("Block height mismatch"));
        }
        
        if self.timestamp <= prev_block.timestamp {
            return Err(anyhow!("Invalid block timestamp"));
        }
        
        Ok(true)
    }
    
    pub fn calculate_hash(&self) -> Result<String> {
        let data = self.prepare_hash_data()?;
        let hash_bytes = Sha256::digest(&data);
        Ok(hex::encode(hash_bytes))
    }
}