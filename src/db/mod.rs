use crate::db::DBInternalError::{DBConnection, DBUnknown};
use crate::types::service::AppService;
use crate::{internal::config::SETTINGS, types::err::AppError};
use once_cell::sync::OnceCell;
use rustle_derive::ErrorHelper;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool, Row};
use tracing::{error, info};
use crate::types::err::EmptyErrResult;

pub mod article;
pub mod rbac;
pub mod user;
pub mod verification;
pub mod fs;

pub static DB_POOL: OnceCell<MySqlPool> = OnceCell::new();
pub fn get_db_pool() -> &'static MySqlPool {
    DB_POOL.get().unwrap()
}
type DBResult<T> = Result<T, sqlx::Error>;
#[derive(ErrorHelper)]
#[err(internal)]
pub enum DBInternalError {
    #[err(msg = "error.db.connection")]
    DBConnection,
    #[err(msg = "error.db.unknown")]
    DBUnknown,
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Io(_) | sqlx::Error::PoolTimedOut => DBConnection.into(),
            _other => DBUnknown.into(),
        }
    }
}
pub struct DBService;
impl AppService for DBService {
    async fn initialize() -> EmptyErrResult<()> {
        let db_config = &SETTINGS.read().database;
        let address = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_config.user_name,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.db_name
        );
        let pool = MySqlPoolOptions::new().connect(&address).await.map_err(|e|{
            error!("failed to establish connection to database, {:?}", e);
            ()
        })?;
        let version = match sqlx::query("select version()").fetch_one(&pool).await {
            Ok(t) => t.get("version()"),
            Err(_e) => String::from("unknown"),
        };
        DB_POOL.set(pool).unwrap();
        info!("connected. Mysql version: {}", version);
        Ok(())
    }
    fn name() -> &'static str {
        "DBService"
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}
