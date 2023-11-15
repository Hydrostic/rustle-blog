use config::Config;
use serde::{Deserialize, Serialize};
use std::{env, sync::RwLock};
use lazy_static::lazy_static;
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct DatabaseConfig{
    pub db_name: String,
    pub user_name: String,
    pub password: String,
    pub host: String,
    pub port: u32
}
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct HttpConfig{
    pub host: String,
    pub port: u32
}
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct BaseConfig{
    pub database: DatabaseConfig,
    pub http: HttpConfig
}
lazy_static! {
    pub static ref DEBUG_MODE: bool = !env::var("DEBUG_ENABLED").is_err();
    pub static ref SETTINGS: RwLock<BaseConfig> = RwLock::new(BaseConfig {
        ..Default::default()
    });
}
pub fn load_config(){
    let settings = Config::builder()
    .add_source(config::File::with_name("./config.toml"))
    .build()
    .unwrap();

    let config = settings.try_deserialize::<BaseConfig>().unwrap();
    let mut sett = SETTINGS.write().unwrap();
    *sett = config;
}