use crate::ctx::Ctx;
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};
use axum::RequestPartsExt;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::Request;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use std::future::ready;
use tower_cookies::{Cookie, Cookies};

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => {
            // TODO: token components validations
            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e),
    };

    // Remove the cookie if something went wrong other than NoAuthTokenCookie
    if result_ctx.is_err() && !matches!(&result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    // Store the result_ctx in the request extension
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// region:      --- Ctx Extractor
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self>> + Send {
        async move {
            println!("->> {:<12} - Ctx", "EXTRACTOR");

            parts
                .extensions
                .get::<Result<Ctx>>()
                .ok_or(Error::AuthFailCtxNotInRequestExtension)
                .unwrap()
                .clone()
        }
    }
}
// endregion:   --- Ctx Extractor

/// Parse a token of format `user-[user-id].[expiration].[signature]`
/// Returns (user_id, expiration, signature)
fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token,)
        .ok_or(Error::AuthFailTokenInvalidFormat)?;

    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenInvalidFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}
