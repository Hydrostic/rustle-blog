

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use salvo::prelude::*;
use tracing::error;
use core::config::SETTINGS;
#[macro_use]
extern crate rbatis;
mod core;
mod routes;
mod utils;
mod db;

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
    core::config::load_config();
    if let Err(e) = db::init_db().await{ 
        error!("failed to test connection with mysql, did it configured right?\n {:?}", e);
        return; 
    };
    let mut router = Router::new();
    router = routes::init(router);
    let http_config = &SETTINGS.read().unwrap().http;
    let acceptor = TcpListener::new(
        format!("{}:{}", http_config.host, http_config.port)).bind().await;
    Server::new(acceptor).serve(router).await;
}
