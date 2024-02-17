#![feature(cow_is_borrowed)]
#![feature(let_chains)]
use crate::db::DBService;
use crate::external::fs::FsService;
use crate::external::mail::MailService;
use crate::internal::log;
use crate::internal::config::ConfigService;
use crate::providers::auth::service::RBACService;
use crate::types::service;

mod internal;
mod providers;
mod utils;
mod db;
mod middlewares;
mod external;
mod types;
#[ntex::main]
async fn main()  {
    log::init_log();
    #[cfg(not(debug_assertions))]
    tracing::info!("Rustle Blog {}({}), compiled on {}",
        env!("BUILD_VERSION"), env!("GIT_HASH"), env!("BUILD_TIME"));
    tracing::info!("running on: {}", os_info::get());
    if !service::init_services!(
        ConfigService,
        DBService,
        RBACService,
        MailService,
        FsService
    ) {
        return;
    }
    let _ =providers::run().await.inspect_err(|e| tracing::error!("http server: {e}"));
}
