use clap::Parser;
use log::info;
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
    SimpleLogger::new().init()?;
    info!("Starting blockchain node");

    let cli = Cli::parse();
    cli.run()?;

    Ok(())
}