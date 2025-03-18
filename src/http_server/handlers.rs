use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Body, Bytes, Frame, Incoming};
use hyper::{header, Error, Method, Request, Response, StatusCode};
use std::collections::HashMap;

pub async fn handle_http_request(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // println!("Got a request: {:?}", request.uri());
    // Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))

    match (request.method(), request.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("Try POSTing data to /echo"))),
        (&Method::POST, "/echo") => handle_echo(request),
        (&Method::POST, "/echo/uppercase") => handle_echo_uppercase(request),
        (&Method::POST, "/echo/reversed") => handle_echo_reversed(request).await,
        (&Method::GET, "/solana/block") => handle_solana_block(request),
        _ => handle_not_found(),
    }
}

fn handle_solana_block(
    request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    let params: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(|| HashMap::new());

    let slot = params.get("slot").map(|s| s.as_str()).unwrap_or("");
    println!("Slot: {}", slot);

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(empty())
        .unwrap())
}

async fn handle_echo_reversed(
    request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    let upper = request.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 1024 * 64 {
        let mut resp = Response::new(full("Body too big"));
        *resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(resp);
    }

    let whole_body = request.collect().await?.to_bytes();
    let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();

    Ok(Response::new(full(reversed_body)))
}

fn handle_echo(request: Request<Incoming>) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    Ok(Response::new(request.into_body().boxed()))
}

fn handle_not_found() -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    let mut not_found = Response::new(empty());
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    Ok(not_found)
}

fn handle_echo_uppercase(
    request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    let frame_stream = request.into_body().map_frame(|frame| {
        let frame = if let Ok(data) = frame.into_data() {
            data.iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Bytes>()
        } else {
            Bytes::new()
        };

        Frame::data(frame)
    });

    Ok(Response::new(frame_stream.boxed()))
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
