#![feature(async_closure)]

use http::{header::HeaderValue, status::StatusCode};
use serde::{Deserialize, Serialize};

use juniper::graphql_object;
use juniper::http::graphiql::graphiql_source;
use std::sync::{atomic, Arc};

use std::env;
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

#[derive(Clone, Default)]
struct State(Arc<atomic::AtomicIsize>);

impl juniper::Context for State {}

struct Query;


graphql_object!(Query: State |&self| {
    // GraphQL integers are signed and 32 bits long.
    field accumulator(&executor) -> i32 as "Current value of the accumulator" {
        executor.context().0.load(atomic::Ordering::Relaxed) as i32
    }
});

// Here is `Mutation` unit struct. GraphQL mutations will refer to this struct. This is similar to
// `Query`, but it provides the way to "mutate" the accumulator state.
struct Mutation;

graphql_object!(Mutation: State |&self| {
    field add(&executor, by: i32) -> i32 as "Add given value to the accumulator." {
        executor.context().0.fetch_add(by as isize, atomic::Ordering::Relaxed) as i32 + by
    }
});

type Schema = juniper::RootNode<'static, Query, Mutation>;

fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| 8186)
}

async fn handle_graphiql(_cx: Context<State>) -> EndpointResult {
    // let address = SocketAddr::from(([127, 0, 0, 1], get_server_port()));
    let address = &format!("127.0.0.1:{}", get_server_port());
    // let html = graphiql_source("http://127.0.0.1:8186/graphql");
    let html = graphiql_source(address);
    let resp = http::Response::builder()
        // .header(http::header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
        .status(http::StatusCode::OK)
        .body(html.as_bytes().into())
        .expect("Failed to build response");
    Ok(resp)
}

async fn handle_graphql(mut cx: Context<State>) -> EndpointResult {
    let query: juniper::http::GraphQLRequest = cx.body_json().await.client_err()?;
    let schema = Schema::new(Query, Mutation);
    let response = query.execute(&schema, cx.state());
    let status = if response.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    let mut resp = response::json(response);
    *resp.status_mut() = status;
    Ok(resp)
}

// async fn echo_json(mut cx: Context<()>) -> EndpointResult {
//     let msg: Message = cx.body_json().await.client_err()?;
//     println!("JSON: {:?}", msg);
//     Ok(response::json(msg))
// }

fn main() {
    let mut app = App::with_state(State::default());
    let address = format!("127.0.0.1:{}", get_server_port());
    // let address = SocketAddr::from(([127, 0, 0, 1], get_server_port()));

    app.middleware(
        CorsMiddleware::new()
            .allow_origin(CorsOrigin::from("*"))
            .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS")),
    );

    app.at("/").get(async move |_| "hello world");
    app.at("/graphql").post(handle_graphql);
    app.at("/graphiql").get(handle_graphiql);

    app.run(address).unwrap();
}
