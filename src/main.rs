

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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(fmt::layer())
        .init();
    #[cfg(not(debug_assertions))]
    tracing::info!("Rustle Blog {}({}), compiled on {}", 
        env!("BUILD_VERSION"), env!("GIT_HASH"), env!("BUILD_TIME"));
    let os_info = os_info::get();
    tracing::info!("running on: {} {}/{}", 
    os_info.os_type(),os_info.version(), os_info.architecture().unwrap_or("unknown arch"));
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
