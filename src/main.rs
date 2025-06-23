use clap::Parser;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

mod block;
mod blockchain;
mod cli;
mod server;
mod transaction;
mod utxoset;
mod wallets;

use crate::cli::Cli;

fn main() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("sled", LevelFilter::Warn)
        .init()?;
    info!("Starting blockchain node");

    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}