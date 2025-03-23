#![allow(unused)]

pub use self::error::{Error, Result};

use crate::ticket_model::ModelController;
use axum::extract::{Path, Query};
use axum::http::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, get_service};
use axum::{Router, middleware};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

mod crypto;
mod ctx;
mod error;
mod ticket_model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize ModelController
    let mc = ModelController::new().await?;

    const SOLANA_API_ENDPOINT: &str = "https://api.devnet.solana.com";
    let solana = crypto::Solana::connect_to_api_url(SOLANA_API_ENDPOINT);

    let routes_apis = web::routes_tickets::routes(mc.clone())
        .merge(web::routes_solana::routes(solana.clone()))
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(get_service(ServeDir::new("./")));

    // region:      --- Start Server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    let addr = listener.local_addr().unwrap();
    println!("->> LISTENING on {}\n", addr);

    axum::serve(listener, routes_all).await.unwrap();
    // endregion:   --- Start Server

    Ok(())
}

async fn main_response_mapper<T>(res: Response<T>) -> Response<T> {
    println!("->> {:<12} - main_response_mapper", "HANDLER");

    println!();
    res
}
