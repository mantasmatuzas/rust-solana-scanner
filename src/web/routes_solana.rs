use crate::crypto::{Solana, SolanaBlock};
use crate::ctx::Ctx;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

pub fn routes(solana: Solana) -> Router {
    Router::new()
        .route("/solana/{slot}", get(get_solana_block_by_slot))
        .with_state(solana)
}

async fn get_solana_block_by_slot(
    State(solana): State<Solana>,
    ctx: Ctx,
    Path(slot): Path<u64>,
) -> crate::Result<Json<SolanaBlock>> {
    println!("->> {:<12} - get_solana_block_by_slot - {slot:?}", "HANDLER");

    let mut block = solana.get_block(slot).await?;

    Ok(Json(block))
}
