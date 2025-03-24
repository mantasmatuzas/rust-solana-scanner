#![allow(unused)]

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
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

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAILED,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response
        response.extensions_mut().insert(self);

        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAILED),

            // -- Auth
            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenInvalidFormat
            | Self::AuthFailCtxNotInRequestExtension => {
                (StatusCode::FORBIDDEN, ClientError::NO_AUTH)
            }

            // -- Model
            Self::TicketDeleteFailIdNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // -- Fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}
