use bincode::{deserialize, serialize};
use ring::signature::Ed25519KeyPair;
use ring::signature::KeyPair;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::path::Path;
use anyhow::Result;

const ADDRESS_VERSION: u8 = 0x00;
const WALLET_FILE: &str = "data/wallets";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pkcs8: Vec<u8>,
}

impl Wallet {
    pub fn new() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Failed to generate key pair")
            .as_ref()
            .to_vec();
        
        Wallet { pkcs8 }
    }

    fn key_pair(&self) -> Ed25519KeyPair {
        Ed25519KeyPair::from_pkcs8(&self.pkcs8)
            .expect("Invalid key pair")
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.key_pair().public_key().as_ref().to_vec()
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.key_pair().sign(message).as_ref().to_vec()
    }

    pub fn get_address(&self) -> String {
        let pubkey = self.public_key();
        let pubkey_hash = hash_pub_key(&pubkey);
        
        let mut payload = vec![ADDRESS_VERSION];
        payload.extend_from_slice(&pubkey_hash);
        
        let checksum = double_sha256(&payload);
        payload.extend_from_slice(&checksum[0..4]);
        
        bs58::encode(payload)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_string()
    }
}

pub fn hash_pub_key(pubkey: &[u8]) -> Vec<u8> {
    let sha_hash = Sha256::digest(pubkey);
    let mut hasher = Ripemd160::new();
    hasher.update(sha_hash);
    hasher.finalize().to_vec()
}

pub fn double_sha256(data: &[u8]) -> Vec<u8> {
    let first = Sha256::digest(data);
    Sha256::digest(&first).to_vec()
}

#[derive(Default)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Result<Self> {
        let mut wlt = Wallets {
            wallets: HashMap::new(),
        };
        
        wlt.load_from_file()?;
        Ok(wlt)
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        address
    }

    pub fn get_all_addresses(&self) -> Vec<String> {
        self.wallets.keys().cloned().collect()
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    fn load_from_file(&mut self) -> Result<()> {
        if !Path::new(WALLET_FILE).exists() {
            return Ok(());
        }
        
        let data = std::fs::read(WALLET_FILE)?;
        self.wallets = deserialize(&data)?;
        Ok(())
    }

    pub fn save_all(&self) -> Result<()> {
        std::fs::create_dir_all("data")?;
        let data = serialize(&self.wallets)?;
        std::fs::write(WALLET_FILE, data)?;
        Ok(())
    }
}