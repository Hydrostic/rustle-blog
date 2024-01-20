

use salvo::hyper::StatusCode;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions, Row};
use once_cell::sync::OnceCell;
use tracing::{error, info};
use crate::{core::config::SETTINGS, utils::error_handling::AppError};



pub mod verification;
pub mod user;
pub mod article;
pub mod rbac;

pub static DB_POOL: OnceCell<MySqlPool> = OnceCell::new();
pub fn get_db_pool() -> &'static MySqlPool{
    DB_POOL.get().unwrap()
}
type DBResult<T> = Result<T, sqlx::Error>;
impl From<sqlx::Error> for AppError{
    fn from(e: sqlx::Error) -> Self {
        match e{
            sqlx::Error::Io(_) | sqlx::Error::PoolTimedOut => {
                AppError::UnexpectedError(StatusCode::SERVICE_UNAVAILABLE, String::from("error.db.connection"))
            },
            _other => {
                AppError::UnexpectedError(StatusCode::INTERNAL_SERVER_ERROR, String::from("error.db.unexpected"))
            }
        }
        
    }

}
pub async fn init_db() -> bool {
    let db_config = &SETTINGS.read().database;
    let address = format!("mysql://{}:{}@{}:{}/{}",
        db_config.user_name,
        db_config.password,
        db_config.host,
        db_config.port,
        db_config.db_name
    );
    let pool = match MySqlPoolOptions::new().connect(&address).await{
        Ok(t) => t,
        Err(e) => {
            error!("failed to estalish connection to database, {:?}", e);
            return false;
        }
    };
    let version = match sqlx::query("select version()").fetch_one(&pool).await{
        Ok(t) => t.get("version()"),
        Err(_e) => String::from("unknown")
    };
    DB_POOL.set(pool).unwrap();
    info!("connected. Mysql version: {}", version);
    true
}