use config::Config;
use sync_cow::SyncCow;
use serde::{Deserialize, Serialize};
use tracing::error;
use std::{env, fs::File, io::Write, sync::Arc};
use lazy_static::lazy_static;
use crate::communication::mail;
use crate::utils::{password_salt, hmac};
use crate::middlewares::session;
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct DatabaseConfig{
    pub db_name: String,
    pub user_name: String,
    pub password: String,
    pub host: String,
    pub port: u32
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct MailConfig{
    #[serde(skip)]
    pub mail_enabled: bool,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub smtp_user_name: String,
    #[serde(default)]
    pub smtp_password: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u32,
    #[serde(default = "max_queue_capacity_default")]
    pub max_queue_capacity: u32
}
fn max_queue_capacity_default() -> u32{
    100
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct HttpConfig{
    pub host: String,
    pub port: u32
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct SecurityConfig{
    #[serde(default)]
    pub password_salt: String,
    #[serde(default)]
    pub session_secret: String,
    #[serde(default)]
    pub credential_secret: String
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct InfoConfig{
    pub name: String,
    pub link: String,
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct BaseConfig{
    pub database: DatabaseConfig,
    pub http: HttpConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub mail: MailConfig,
    #[serde(default)]
    pub info: InfoConfig
}
lazy_static! {
    pub static ref DEBUG_MODE: bool = !env::var("DEBUG_ENABLED").is_err();
    pub static ref SETTINGS: Arc<SyncCow<BaseConfig>> = Arc::new(SyncCow::new(BaseConfig{..Default::default()}.to_owned()));
}
pub fn generate_config(c: &mut BaseConfig) -> Result<(),anyhow::Error>{
    let mut need_rewrite = false;
    need_rewrite = password_salt::init_config(c)? || need_rewrite;
    need_rewrite = session::init_config(c)? || need_rewrite;
    need_rewrite = hmac::init_config(c)? || need_rewrite;
    need_rewrite = mail::init_config(c)? || need_rewrite;

    if need_rewrite{
        let mut buf = File::create("./config.toml").unwrap();
        if let Err(e) = buf.write_all(toml::to_string(c).unwrap().as_bytes()){
            error!("couldn't write to config file. Is permission right?\n {:?}", e);
            return Err(e.into())
        }
    }
    Ok(())
}
pub fn read_config() -> Arc<BaseConfig>{
    let cow = &*SETTINGS.clone();
    cow.read()
}
pub fn load_config(){
    let settings = Config::builder()
    .add_source(config::File::with_name("./config.toml"))
    .build()
    .unwrap();
    
    let mut config = settings.try_deserialize::<BaseConfig>().unwrap();
    generate_config(&mut config).expect("exit due to config error");
    let write_arc = SETTINGS.clone();
    (&*write_arc).edit(|x|{
        *x = config;
    });
}