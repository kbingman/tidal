#![feature(async_closure)]

use http::header::HeaderValue;
use serde::{Deserialize, Serialize};

use std::{env, net::SocketAddr};
use tide::{
    error::ResultExt,
    middleware::{CorsMiddleware, CorsOrigin},
    response, App, Context, EndpointResult,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    author: Option<String>,
    contents: String,
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| 8186)
}

async fn echo_json(mut cx: Context<()>) -> EndpointResult {
    let msg: Message = cx.body_json().await.client_err()?;
    println!("JSON: {:?}", msg);
    Ok(response::json(msg))
}

fn main() {
    let mut app = App::new();
    let address = SocketAddr::from(([127, 0, 0, 1], get_server_port()));

    app.middleware(
        CorsMiddleware::new()
            .allow_origin(CorsOrigin::from("*"))
            .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS")),
    );

    app.at("/").get(async move |_| "hello world");
    app.at("/json").get(echo_json);
    app.run(address).unwrap();
}
