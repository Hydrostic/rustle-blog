use anyhow::anyhow;
use rbdc_mysql::MysqlDriver;
use rbatis::RBatis;
use log::LevelFilter;
use rbatis::intercept::Intercept;
use rbatis::intercept_log::LogInterceptor;
use tracing::info;
use std::sync::{Arc, OnceLock};
use crate::core::config::SETTINGS;
pub static RB_LOG: OnceLock<Arc<LogInterceptor>> = OnceLock::new();
pub async fn init_db() -> Result<(), anyhow::Error> {
    let rb = RBatis::new();
    // set log level
    let l = Arc::new(LogInterceptor::new(LevelFilter::Trace));
    rb.intercepts.clear();
    rb.intercepts.push(l.clone() as Arc<dyn Intercept>);
    _ = RB_LOG.set(l);
    let db_config = &(SETTINGS.read().map_err(|_|anyhow!("rwlock failure"))?).database;
    let address = format!("mysql://{}:{}@{}:{}/{}",
        db_config.user_name,
        db_config.password,
        db_config.host,
        db_config.port,
        db_config.db_name
    );
    rb.init(MysqlDriver {}, &address).unwrap();
    let version: String = rb.query_decode("SELECT version()", vec![]).await?;
    info!("connected. Mysql version: {}", version);
    Ok(())
}