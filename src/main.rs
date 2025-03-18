mod http_server;
mod solana_driver;

use crate::solana_driver::{Error, SolanaBlock, SolanaDriver};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    http_server::start().await
}

fn get_block() -> Result<SolanaBlock, Error> {
    const SOLANA_API_ENDPOINT: &str = "https://api.devnet.solana.com";
    println!("Connecting to Solana {}", SOLANA_API_ENDPOINT);

    SolanaDriver::new(SOLANA_API_ENDPOINT)
        .get_block(354587721)
        .inspect(|block| {
            println!(
                "Transaction count in slot {} is {}",
                block.slot, block.transaction_count
            );
        })
}
