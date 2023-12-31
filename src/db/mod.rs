use rbdc_mysql::MysqlDriver;
use rbatis::RBatis;
use lazy_static::lazy_static;
use log::LevelFilter;
use rbatis::intercept::Intercept;
use rbatis::intercept_log::LogInterceptor;
use tracing::info;
use std::sync::{Arc, OnceLock};
use crate::core::config::read_config;

pub mod verification;
pub mod user;
pub static RB_LOG: OnceLock<Arc<LogInterceptor>> = OnceLock::new();
lazy_static!{
    pub static ref RB: RBatis = RBatis::new();
}

pub async fn init_db() -> Result<(), anyhow::Error> {
    // set log level
    let l = Arc::new(LogInterceptor::new(LevelFilter::Trace));
    RB.intercepts.clear();
    RB.intercepts.push(l.clone() as Arc<dyn Intercept>);
    _ = RB_LOG.set(l);
    let db_config = &read_config().database;
    let address = format!("mysql://{}:{}@{}:{}/{}",
        db_config.user_name,
        db_config.password,
        db_config.host,
        db_config.port,
        db_config.db_name
    );
    RB.init(MysqlDriver {}, &address).unwrap();
    let version: String = RB.query_decode("SELECT version()", vec![]).await?;
    info!("connected. Mysql version: {}", version);
    Ok(())
}