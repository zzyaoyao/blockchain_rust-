use crate::utxoset::UTXOSet;
use anyhow::Result;
use log::info;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn start_miner_node(port: u16, miner_addr: &str, utxo_set: UTXOSet) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    
    let miner_addr = miner_addr.to_string();
    
    for stream in listener.incoming() {
        let stream = stream?;
        let utxo_set = utxo_set.clone();
        let miner_addr = miner_addr.clone();
        
        thread::spawn(move || {
            handle_connection(stream, &utxo_set, Some(miner_addr));
        });
    }
    
    Ok(())
}

pub fn start_full_node(port: u16, utxo_set: UTXOSet) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    
    for stream in listener.incoming() {
        let stream = stream?;
        let utxo_set = utxo_set.clone();
        
        thread::spawn(move || {
            handle_connection(stream, &utxo_set, None);
        });
    }
    
    Ok(())
}

fn handle_connection(stream: TcpStream, _utxo_set: &UTXOSet, _miner_addr: Option<String>) {
    info!("New connection from {:?}", stream.peer_addr());
}