#![allow(unused)]

pub use self::error::{Error, Result};

use crate::ctx::Ctx;
use crate::log::log_request;
use crate::ticket_model::ModelController;
use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{Json, Router, middleware};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod crypto;
mod ctx;
mod error;
mod log;
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

async fn main_response_mapper(
    ctx: Result<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new response
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string()
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            // Build the new response from client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1;
    let _ = log_request(uuid, req_method, uri, ctx, service_error, client_error).await;
    println!("    ->> server log line - {uuid} - Error: {service_error:?}");

    println!();
    error_response.unwrap_or(res)
}
