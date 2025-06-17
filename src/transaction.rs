use crate::utxoset::UTXOSet;
use crate::wallets::Wallet;
use anyhow::{anyhow, Result};
use bincode::serialize;
use log::info;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use bs58;

const SUBSIDY: i32 = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub pub_key_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    pub fn new_utxo(
        wallet: &Wallet,
        to: &str,
        amount: i32,
        utxo: &UTXOSet,
    ) -> Result<Transaction> {
        info!(
            "New UTXO Transaction from: {} to: {}",
            wallet.get_address(),
            to
        );
        
        let pub_key_hash = wallet.public_key();
        let pub_key_hash = crate::wallets::hash_pub_key(&pub_key_hash);
        
        let (acc_value, outputs) = utxo.find_spendable_outputs(&pub_key_hash, amount)?;
        
        if acc_value < amount {
            return Err(anyhow!(
                "Insufficient balance: current {}, required {}",
                acc_value,
                amount
            ));
        }
        
        let mut vin = Vec::new();
        for (txid, outs) in outputs {
            for out in outs {
                let input = TXInput {
                    txid: txid.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: wallet.public_key().to_vec(),
                };
                vin.push(input);
            }
        }
        
        let mut vout = vec![TXOutput::new(amount, to)?];
        if acc_value > amount {
            vout.push(TXOutput::new(acc_value - amount, &wallet.get_address())?);
        }
        
        let mut tx = Transaction { id: String::new(), vin, vout };
        tx.id = tx.hash()?;
        Ok(tx)
    }

    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        info!("New coinbase Transaction to: {}", to);
        
        let mut rand_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut rand_bytes);
        
        if data.is_empty() {
            data = format!("Reward to '{}'", to);
        }
        
        let mut pub_key = data.into_bytes();
        pub_key.extend_from_slice(&rand_bytes);
        
        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                signature: Vec::new(),
                pub_key,
            }],
            vout: vec![TXOutput::new(SUBSIDY, &to)?],
        };
        
        tx.id = tx.hash()?;
        Ok(tx)
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn hash(&self) -> Result<String> {
        let mut copy = self.clone();
        copy.id.clear();
        
        let serialized = serialize(&copy)?;
        let hash = Sha256::digest(&serialized);
        Ok(hex::encode(hash))
    }
}

impl TXOutput {
    pub fn new(value: i32, address: &str) -> Result<Self> {
        let mut txo = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        txo.lock(address)?;
        Ok(txo)
    }


    pub fn lock(&mut self, address: &str) -> Result<()> {
        let decoded = bs58::decode(address)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()?;
        
        if decoded.len() < 5 {
            return Err(anyhow!("Invalid address length"));
        }
        
        let version = decoded[0];
        let pub_key_hash = &decoded[1..decoded.len()-4];
        let checksum = &decoded[decoded.len()-4..];
        
        let mut payload = vec![version];
        payload.extend(pub_key_hash);
        let hash = crate::wallets::double_sha256(&payload);
        
        if &hash[0..4] != checksum {
            return Err(anyhow!("Invalid address checksum"));
        }
        
        self.pub_key_hash = pub_key_hash.to_vec();
        Ok(())
    }

        pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }
}