use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status_client_types::{TransactionDetails, UiTransactionEncoding};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {}

pub struct SolanaBlock {
    pub slot: u64,
    pub transaction_count: usize,
}
pub struct SolanaDriver {
    rpc_client: solana_client::rpc_client::RpcClient,
}

impl SolanaDriver {
    pub fn new(api_url: &str) -> Self {
        Self {
            rpc_client: solana_client::rpc_client::RpcClient::new_with_commitment(
                api_url,
                CommitmentConfig::confirmed(),
            ),
        }
    }

    pub fn get_block(&self, slot: u64) -> Result<SolanaBlock, Error> {
        self.rpc_client
            .get_block_with_config(
                slot,
                RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    transaction_details: Some(TransactionDetails::Full),
                    rewards: Some(false),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
            .map(|block| match block.transactions {
                Some(txs) => SolanaBlock {
                    slot,
                    transaction_count: txs.len(),
                },
                None => SolanaBlock {
                    slot,
                    transaction_count: 0,
                },
            })
            .map_err(|error| Error {
                message: format!("{:?}", error),
            })
    }
}
