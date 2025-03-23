#![allow(dead_code)]

use crate::{Error, Result};
use std::sync::Arc;

use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status_client_types::{TransactionDetails, UiTransactionEncoding};

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize)]
pub struct SolanaBlock {
    pub slot: u64,
    pub transaction_count: usize,
}

#[derive(Clone)]
pub struct Solana {
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
}

impl Solana {
    pub fn connect_to_api_url(api_url: &str) -> Self {
        println!("Connecting to {}", api_url);
        Self {
            rpc_client: Arc::new(solana_client::rpc_client::RpcClient::new_with_commitment(
                api_url,
                CommitmentConfig::confirmed(),
            )),
        }
    }

    pub async fn get_block(&self, slot: u64) -> Result<SolanaBlock> {
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
            .map_err(|_| Error::SolanaBlockNotFound { slot })
    }
}
