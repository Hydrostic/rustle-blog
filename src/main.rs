#![feature(error_generic_member_access)]
#![feature(result_option_inspect)]

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use salvo::prelude::*;
mod core;
mod routes;
mod utils;
#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(fmt::layer())
        .init();
    tracing::info!("Rustle Blog v1.0.0({})", env!("GIT_HASH"));
    let mut router = Router::new();
    router = routes::init(router);
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
