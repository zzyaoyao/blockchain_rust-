use crate::blockchain::Blockchain;
use crate::transaction::TXOutputs;
use anyhow::Result;
use bincode::deserialize;
use sled;
use std::collections::HashMap;
use bincode;

const UTXO_BUCKET: &str = "utxoset";

#[derive(Clone)]
pub struct UTXOSet {
    pub blockchain: Blockchain,
}

impl UTXOSet {
    pub fn reindex(&self) -> Result<usize> {
        std::fs::create_dir_all("data")?;
        let db = sled::open("data/utxoset")?;
        db.clear()?;
        
        let utxos = self.blockchain.find_utxo()?;
        let len = utxos.len();
        for (txid, outs) in utxos {
            db.insert(txid.as_bytes(), bincode::serialize(&outs)?)?;
        }
        
        Ok(len)
    }
    
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &Vec<u8>,
        amount: i32,
    ) -> Result<(i32, HashMap<String, Vec<i32>>)> {
        let db = sled::open("data/utxoset")?;
        let mut acc = 0;
        let mut outputs = HashMap::new();
        
        for item in db.iter() {
            let (key, value) = item?;
            let txid = String::from_utf8(key.to_vec())?;
            let outs: TXOutputs = deserialize(&value)?;
            
            for (idx, out) in outs.outputs.iter().enumerate() {
                if acc < amount && out.is_locked_with_key(pub_key_hash) {
                    acc += out.value;
                    outputs.entry(txid.clone())
                        .or_insert_with(Vec::new)
                        .push(idx as i32);
                }
            }
        }
        
        Ok((acc, outputs))
    }
    
    pub fn get_balance(&self, address: &str) -> Result<i32> {
        let pub_key_hash = bs58::decode(address)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()?;
        
        // 跳过版本字节和校验和
        let pub_key_hash = &pub_key_hash[1..pub_key_hash.len()-4];
        
        let mut balance = 0;
        let db = sled::open("data/utxoset")?;
        
        for item in db.iter() {
            let (_, value) = item?;
            let outs: TXOutputs = deserialize(&value)?;
            
            for out in outs.outputs {
                if out.is_locked_with_key(pub_key_hash) {
                    balance += out.value;
                }
            }
        }
        
        Ok(balance)
    }
    
    pub fn count_transactions(&self) -> Result<usize> {
        let db = sled::open("data/utxoset")?;
        Ok(db.len())
    }
}