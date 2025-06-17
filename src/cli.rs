use crate::blockchain::Blockchain;
use crate::server::{start_full_node, start_miner_node};
use crate::transaction::Transaction;
use crate::utxoset::UTXOSet;
use crate::wallets::Wallets;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new wallet
    CreateWallet,
    /// Get balance for an address
    GetBalance {
        address: String,
    },
    /// Create a new blockchain
    CreateBlockchain {
        address: String,
    },
    /// Print blockchain info
    Info,
    /// Start a node
    StartNode {
        port: u16,
        miner_address: Option<String>,
    },
    /// Send coins from one address to another
    Send {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: i32,
        #[arg(long, action = clap::ArgAction::SetTrue)] 
        mine: bool,
    },
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::CreateWallet => self.cmd_create_wallet(),
            Command::GetBalance { ref address } => self.cmd_get_balance(address),
            Command::CreateBlockchain { ref address } => self.cmd_create_blockchain(address),
            Command::Info => self.cmd_info(),
            Command::StartNode { port, ref miner_address } => self.cmd_start_node(*port, miner_address),
            Command::Send { ref from, ref to, amount, mine } => self.cmd_send(from, to, *amount, *mine),
        }
    }

    fn cmd_create_wallet(&self) -> Result<()> {
        let mut wallets = Wallets::new()?;
        let address = wallets.create_wallet();
        wallets.save_all()?;
        println!("Wallet created");
        println!("Address: {}", address);
        Ok(())
    }

    fn cmd_get_balance(&self, address: &str) -> Result<()> {
        let bc = Blockchain::open()?;
        let utxo_set = UTXOSet { blockchain: bc };
        let balance = utxo_set.get_balance(address)?;
        println!("Balance of '{}': {}", address, balance);
        Ok(())
    }

    fn cmd_create_blockchain(&self, address: &str) -> Result<()> {
        let bc = Blockchain::create_blockchain(address)?;
        let utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;
        println!("Blockchain created");
        Ok(())
    }

    fn cmd_info(&self) -> Result<()> {
        let bc = Blockchain::open()?;
        let utxo_set = UTXOSet { blockchain: bc };
        let best_height = utxo_set.blockchain.get_best_height()?;
        let block_count = utxo_set.blockchain.get_block_count()?;
        let utxo_count = utxo_set.count_transactions()?;
        
        let wallets = Wallets::new()?;
        let addresses = wallets.get_all_addresses();
        
        println!("Blockchain Info:");
        println!("{}", "=".repeat(40));
        println!("Blocks:         {}", block_count);
        println!("Best Height:    {}", best_height);
        println!("UTXO Count:     {}", utxo_count);
        println!("Wallet Count:   {}", addresses.len());
        
        if !addresses.is_empty() {
            println!();
            println!("Wallet Balances:");
            println!("{}", "-".repeat(30));
            
            for addr in &addresses {
                let balance = utxo_set.get_balance(addr)?;
                println!("{:<34} : {:>8} BTC", addr, balance);
            }
        }
        
        Ok(())
    }

    fn cmd_start_node(&self, port: u16, miner_address: &Option<String>) -> Result<()> {
        let bc = Blockchain::open()?;
        let utxo_set = UTXOSet { blockchain: bc };
        
        if let Some(addr) = miner_address {
            println!("Starting miner node on port {}", port);
            start_miner_node(port, addr, utxo_set)?;
        } else {
            println!("Starting full node on port {}", port);
            start_full_node(port, utxo_set)?;
        }
        
        Ok(())
    }

    fn cmd_send(&self, from: &str, to: &str, amount: i32, mine: bool) -> Result<()> {
        let bc = Blockchain::open()?;
        let utxo_set = UTXOSet { blockchain: bc.clone() };
        
        let wallets = Wallets::new()?;
        let wallet = wallets.get_wallet(from)
            .ok_or_else(|| anyhow!("Wallet not found"))?;
        
        let tx = Transaction::new_utxo(wallet, to, amount, &utxo_set)?;
        
        if mine {
            let mut bc_mine = bc.clone();
            bc_mine.mine_block(vec![tx])?;
        } else {
            // TODO: 发送交易到网络
        }
        
        println!("Success!");
        Ok(())
    }
}