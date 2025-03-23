#![allow(unused)]

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    LoginFail,

    // -- Auth errors
    AuthFailNoAuthTokenCookie,
    AuthFailTokenInvalidFormat,
    AuthFailCtxNotInRequestExtension,

    // -- Model errors
    TicketDeleteFailIdNotFound { id: u64 },

    // -- Solana Errors
    SolanaBlockNotFound { slot: u64 },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}
