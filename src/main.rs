

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use salvo::prelude::*;
use tracing::error;
use core::config::read_config;
use middlewares::session as sessionMiddleware;
#[macro_use]
extern crate rbatis;

mod core;
mod providers;
mod utils;
mod db;
mod middlewares;
mod communication;
mod fs;
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
    let router = Router::new().hoop(sessionMiddleware::new_middleware())
    .push(providers::init());
    let http_config = &read_config().http;
    let acceptor = TcpListener::new(
        format!("{}:{}", http_config.host, http_config.port)).bind().await;
    Server::new(acceptor).serve(router).await;
}
