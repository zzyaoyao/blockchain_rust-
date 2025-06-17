# Rust Blockchain Implementation


A complete blockchain implementation in Rust, featuring:

- Proof-of-Work mining
- UTXO (Unspent Transaction Output) model
- Wallet creation and transaction signing
- Simple CLI interface

## Features

- ✅ Create wallets and addresses
- ✅ Mine blocks with custom difficulty
- ✅ Send transactions between addresses
- ✅ Blockchain persistence using sled DB
- ✅ UTXO set for efficient balance checking

## Getting Started

### Prerequisites

- Rust (>= 1.65.0)
- Cargo

### Installation

```bash
git clone https://github.com/YOUR_USERNAME/blockchain_rust.git
cd blockchain_rust
cargo build --release
