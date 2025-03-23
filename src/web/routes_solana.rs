use crate::crypto::{Solana, SolanaBlock};
use crate::ctx::Ctx;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

pub fn routes(solana: Solana) -> Router {
    Router::new()
        .route("/solana/{slot}", get(get_block))
        .with_state(solana)
}

async fn get_block(
    State(solana): State<Solana>,
    ctx: Ctx,
    Path(slot): Path<u64>,
) -> crate::Result<Json<SolanaBlock>> {
    println!("->> {:<12} - list_tickets", "HANDLER");

    let mut block = solana.get_block(slot).await?;

    Ok(Json(block))
}
