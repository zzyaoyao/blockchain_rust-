use crate::block::Block;
use crate::transaction::Transaction;
use crate::transaction::TXOutputs;
use anyhow::{anyhow, Result};
use bincode::{deserialize, serialize};
use sled::Db;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::block::TARGET_BITS;

const GENESIS_COINBASE_DATA: &str = "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

pub struct Blockchain {
    tip: Mutex<String>,
    db: Db,
}

impl Clone for Blockchain {
    fn clone(&self) -> Self {
        let db = self.db.clone();
        let tip = self.tip.lock().unwrap().clone();
        Blockchain {
            tip: Mutex::new(tip),
            db,
        }
    }
}

impl Blockchain {
    pub fn create_blockchain(address: &str) -> Result<Self> {
        let db = sled::open("data/blocks")?;
        
        if db.contains_key("l")? {
            return Err(anyhow!("Blockchain already exists"));
        }
        
        let cbtx = Transaction::new_coinbase(
            address.to_string(),
            GENESIS_COINBASE_DATA.to_string(),
        )?;
        
        let genesis = Block::new_block(
            vec![cbtx],
            String::new(),
            0,
            TARGET_BITS,
        )?;
        
        let genesis_hash = genesis.hash.clone();
        db.insert(genesis_hash.as_bytes(), serialize(&genesis)?)?;
        db.insert("l", genesis_hash.as_bytes())?;
        
        let bc = Blockchain {
            tip: Mutex::new(genesis_hash),
            db,
        };
        
        Ok(bc)
    }
    
    pub fn open() -> Result<Self> {
        let db = sled::open("data/blocks")?;
        let tip = match db.get("l")? {
            Some(t) => String::from_utf8(t.to_vec())?,
            None => return Err(anyhow!("Blockchain not found. Create one first")),
        };
        
        Ok(Blockchain {
            tip: Mutex::new(tip),
            db,
        })
    }
    
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        let last_hash = self.tip.lock().unwrap().clone();
        let new_height = self.get_best_height()? + 1;
        
        let block = Block::new_block(transactions, last_hash, new_height, TARGET_BITS)?;
        let block_hash = block.hash.clone();
        
        self.db.insert(block_hash.as_bytes(), serialize(&block)?)?;
        self.db.insert("l", block_hash.as_bytes())?;
        *self.tip.lock().unwrap() = block_hash;
        
        Ok(())
    }
    
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        self.add_block(transactions)
    }
    
    pub fn get_best_height(&self) -> Result<i32> {
        let tip = self.tip.lock().unwrap().clone();
        let block = self.get_block(&tip)?;
        Ok(block.height)
    }
    
    pub fn get_block_count(&self) -> Result<usize> {
        Ok(self.db.len() - 1)
    }
    
    pub fn get_block(&self, hash: &str) -> Result<Block> {
        let data = self.db.get(hash)?
            .ok_or_else(|| anyhow!("Block not found"))?;
        let block = deserialize(&data)?;
        Ok(block)
    }
    
    pub fn iter(&self) -> BlockchainIter {
        BlockchainIter {
            current_hash: self.tip.lock().unwrap().clone(),
            bc: self,
        }
    }
    
    pub fn get_block_hashes(&self) -> Result<Vec<String>> {
        let mut hashes = Vec::new();
        for block in self.iter() {
            let block = block?;
            hashes.push(block.hash.clone());
        }
        Ok(hashes)
    }
    
    pub fn find_utxo(&self) -> Result<HashMap<String, TXOutputs>> {
        let mut utxos = HashMap::new();
        let mut spent_outputs = HashMap::new();
    
        for block in self.iter() {
            let block = block?;
            for tx in block.transactions {
                // 收集所有输出
                for (_vout_idx, output) in tx.vout.iter().enumerate() {
                    let txid = tx.id.clone();
                    let outputs = utxos.entry(txid).or_insert(TXOutputs { outputs: vec![] });
                    outputs.outputs.push(output.clone());
                }
    
                // 如果不是 coinbase 交易，标记已花费的输出
                if !tx.is_coinbase() {
                    for input in &tx.vin {
                        spent_outputs
                            .entry(input.txid.clone())
                            .or_insert(vec![])
                            .push(input.vout);
                    }
                }
            }
        }
    
        // 移除已花费的输出
        for (txid, spent_indices) in spent_outputs {
            if let Some(outputs) = utxos.get_mut(&txid) {
                for spent_idx in spent_indices {
                    if (spent_idx as usize) < outputs.outputs.len() {
                        outputs.outputs[spent_idx as usize].value = -1; // 标记为已花费
                    }
                }
            }
        }
    
        // 清理已花费的输出
        for outputs in utxos.values_mut() {
            outputs.outputs.retain(|output| output.value != -1);
        }
    
        Ok(utxos)
    }
}

pub struct BlockchainIter<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl<'a> Iterator for BlockchainIter<'a> {
    type Item = Result<Block>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_hash.is_empty() {
            return None;
        }
        
        match self.bc.get_block(&self.current_hash) {
            Ok(block) => {
                self.current_hash = block.prev_block_hash.clone();
                Some(Ok(block))
            }
            Err(e) => Some(Err(e)),
        }
    }
}