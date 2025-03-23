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
        .merge(routes_hello())
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

// region:      --- Routes Hello
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/{name}", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// e.g., `hello?name=Mantas`
async fn handler_hello(Query(params): Query<HelloParams>) -> impl axum::response::IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("<h1>Hello {name}</h1>"))
}

// e.g., `/hello2/Mantas`
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("<h1>Hello {name}</h1>"))
}

// endregion:   --- Handler Hello

// #[allow(dead_code)]
// fn get_block() -> Result<crypto::SolanaBlock, crypto::Error> {
//     const SOLANA_API_ENDPOINT: &str = "https://api.devnet.solana.com";
//     println!("Connecting to Solana {}", SOLANA_API_ENDPOINT);
//
//     crypto::Solana::connect_to_api_url(SOLANA_API_ENDPOINT)
//         .get_block(354587721)
//         .inspect(|block| {
//             println!(
//                 "Transaction count in slot {} is {}",
//                 block.slot, block.transaction_count
//             );
//         })
// }
